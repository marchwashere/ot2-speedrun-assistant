'use strict';
// Static source audit only. Does not attach to the game or change predictions.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const s = require('./source.cjs');
const decoder = require('../../OT2BattleEngine/tools/kismet_decoder.cjs');
const { audit } = require('../../OT2BattleEngineRust/tools/audit-sequence.cjs');
const sources = [];
function table(name) {
  const t = s.table(name);
  sources.push({ source: t.source, sha256: t.sha256 });
  return t.rows;
}
function script(id) {
  const bytes = fs.readFileSync(path.join(s.assets.CONTENT, 'Event/json', id));
  sources.push({ source: '/Game/Event/json/' + id, sha256: s.assets.hash(bytes) });
  return Object.values(JSON.parse(bytes));
}
function blueprint(name, functions) {
  const file = path.join(s.assets.CONTENT, name + '.uasset');
  const p = s.assets.loadPackage(file), q = decoder.loadPackage(file);
  sources.push({ source: '/Game/' + name, sha256: s.assets.hash(p.b) });
  const schemas = s.extract.extractPackage(p, file).schemas;
  return functions.map(name => {
    const ex = p.exports.find(e => e.name === name);
    assert(ex, name);
    const schema = schemas.find(e => e.exportIndex === ex.index);
    const decoded = decoder.decodeFrom(q, name, schema.scriptStart - ex.serialOffset, true);
    assert.equal(decoded.exprs.at(-1).name, 'EndOfScript');
    return { name, body: decoded.exprs.map(e => decoder.flatten(e).join('\n')).join('\n') };
  });
}
const placement = table('Placement/Database/PlacementData');
const guard = placement.find(r => r.id === 'NPC_Dng_Cty_3_1_0100_N000');
assert.equal(guard.EventType_A, 'eFC_ASSASSINATE_SUCCESS');
assert.equal(guard.EventParam_A_1, 'None');
assert.equal(guard.EventStartFlag_A, 8003);
assert.equal(guard.EventEndFlag_A, 8004);
assert.equal(guard.EventLabel_A, 'MS_TOU_10_0410');
const npc = table('Character/Database/NPCData').find(r => r.id === guard.ResourceLabel);
const battle = table('Character/Database/NPCBattleData').find(r => r.id === npc.FCmd_Battle_ID);
assert.equal(battle.AssassinateNeedLevel, 1);
assert.equal(battle.CommandEnable, 4);
assert.equal(battle.CanSwoon, true);
assert(battle.SwooningFlags.every(n => n === 0));
const events = table('Event/Database/EventList');
const ambush = events.find(r => r.id === guard.EventLabel_A);
const rejoin = events.find(r => r.id === 'MS_TOU_10_0500');
assert(ambush.EncountReset && rejoin.EncountReset);
assert.equal(rejoin.EncountTrigger, 'ENT_DNG_CTY_3_1_LND_1');
const ambushScript = script(ambush.id), rejoinScript = script(rejoin.id);
assert.equal(ambushScript[0].text, 'eFC_ASSASINATE_PC_TALK_01_A');
assert(ambushScript.findIndex(c => c.cmd === 9990) < ambushScript.findIndex(c => c.cmd === 1000));
assert.equal(ambushScript.find(c => c.cmd === 1000).opt[0], rejoin.id);
assert(rejoinScript.some(c => c.cmd === 71 && c.target === 8004 && c.opt[0] === '1'));
assert(rejoinScript.some(c => c.cmd === 70 && c.target === 25304 && c.opt[0] === '0'));
assert.deepEqual(rejoinScript.filter(c => c.cmd === 3000).map(c => c.target), [15, 16]);
const fieldData = table('FieldCommand/Database/FieldCommandData').filter(r => /ASSAS/i.test(JSON.stringify(r)));
const voice = table('Talk/Database/TalkVoice_EN').find(r => r.id === ambushScript[0].text);
assert(voice, 'fixed English voice row');
const mapLabels = table('Placement/Database/PlacementList').find(r => r.id === 'Dng_Cty_3_1').LabelList;
const mapPlacements = mapLabels.map(id => placement.find(r => r.id === id));
assert.equal(mapPlacements.filter(r => r.PlacementType === 1 && r.SpawnStartFlag === 8004).length, 0);
const sequenceName = 'Event/RichEvent/01_Main/70_TOU/ep1/RI_MS_TOU_10_0500';
const sequence = audit(sequenceName);
sources.push({ source: '/Game/' + sequenceName,
  sha256: s.assets.hash(s.assets.loadPackage(path.join(s.assets.CONTENT, sequenceName + '.uasset')).b) });
const flow = blueprint('FieldCommand/BPv2/Flow/BP_FCv2_FieldCommandFlow_Assassinate',
  ['UpdateFlow', 'IsGreaterEqualRequiredLevel', 'SwoonNpcInFade']);
assert(flow[0].body.includes('LocalVirtualFunction DrawEventLabel'));
const base = blueprint('FieldCommand/BPv2/Flow/BP_FCv2_FieldCommandFlowBase',
  ['DrawEventLabel', 'SwoonNpc_Internal', 'PlayFieldCommandAction']);
assert(base[0].body.includes('RandomFloatInRange'));
const manager = blueprint('FieldCommand/BPv2/FCv2_FieldCommandManager',
  ['UpdatePreSequence', 'PlayFieldCommandActions']);
assert(manager[0].body.includes('K2_SetTimerDelegate'));
assert(manager[1].body.includes('PlayFieldCommandAction'));
const result = {
  status: 'Source-derived expectation; not live-validated',
  boundary: 'Confirm Ambush on first sewer guard after MS_TOU_10_0400; skip MS_TOU_10_0500; stop when field control returns',
  expected_crt_draws: 4,
  menu_opening: {
    expected_crt_draws: 1,
    expected_total_from_before_menu: 5,
    reason: 'UpdatePreSequence @541 starts FieldCommandActionTimerHandle once after PlayFieldCommandActions. K2_SetTimerDelegate uses CRT rand even with InitialStartDelayVariance=0.',
    cancel_and_reopen: 'Cancelling does not undo the consumed draw; reopening through the normal action-start path consumes another one.',
    scope: 'Normal path-action preview opening with a valid field character/action and world; not the passive nearby-NPC button prompt.'
  },
  expected_order: [
    { kind: 'ambush_level_check_and_story_dialogue', draws: 0 },
    { kind: 'MS_TOU_10_0410_counter_B', draws: 1 },
    { kind: 'MS_TOU_10_0410_counter_A', draws: 1 },
    { kind: 'MS_TOU_10_0500_counter_B', draws: 1 },
    { kind: 'MS_TOU_10_0500_counter_A', draws: 1 }
  ],
  distinction: 'Generic Ambush state 2 draws a weighted before-dialogue label once. This guard matches state 1 event type 20 / None and jumps to state 8, bypassing state 2.',
  limitations: 'Not a universal Ambush offset or exhaustive proof of every transitive callback. No live process capture was made by this audit.',
  guard, npc, battle, fieldData, voice,
  events: [{ row: ambush, commands: ambushScript }, { row: rejoin, commands: rejoinScript }],
  field_npc_changes: mapPlacements.filter(r => r.PlacementType === 1).map(r => ({
    id: r.id, start: r.SpawnStartFlag, end: r.SpawnEndFlag
  })),
  blueprint: { flow, base, manager },
  sequence: { event_keys: sequence.length,
    calls: [...new Set(sequence.flatMap(e => [...e.body.matchAll(/LocalVirtualFunction (\S+)/g)].map(m => m[1])))] },
  native_evidence: {
    random_float_in_range: 'FUN_1430d6fa0: one rand(); binary32 scale 3.051851e-05',
    fixed_voice: 'GetTalkVoice thunk FUN_14105c080 -> FUN_140f63560 -> FUN_140f63600: language-specific table lookup',
    gc_timer: 'FUN_141053880 -> FUN_140f53710: deterministic timer-type lookup; FUN_142faac30 updates GC timing, not a CRT roll',
    menu_action_timer: 'K2_SetTimerDelegate thunk FUN_1430f96c0 -> FUN_142c52d90: rand() before multiplying by InitialStartDelayVariance; zero variance still advances CRT once (return RVA 0x2C52E57)',
    finish_pre: 'FUN_140e8e740 guards +0x519 per event; EventManagerBP.FinishPre @3470 calls ResetEncountStep',
    counter: 'EncountManagerBP.CalcEncountStepCount @130 B then @167 A, two CRT calls per reset'
  },
  sources
};
fs.writeFileSync(path.join(__dirname, '../data/source/throne-ambush.json'), JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ status: result.status, expected_crt_draws: result.expected_crt_draws,
  fixed_voice: voice, sequence_keys: sequence.length, decoded_flow: flow.map(f => f.name) }));
