'use strict';
// Read-only game-source audit. This does not extend the predictor's route boundary.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const s = require('./source.cjs');
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
const events = table('Event/Database/EventList');
const event = events.find(r => r.id === 'MS_TOU_10_0400');
const commands = script(event.id);
const nextCommands = script('MS_TOU_10_0410');
const rejoinCommands = script('MS_TOU_10_0500');
assert.equal(event.MapID, '57');
assert.equal(event.EncountReset, true);
assert.equal(event.EncountTrigger, 'ENT_EMPTY_1');
const flagIndex = commands.findIndex(c => c.cmd === 71 && c.target === 8003 && c.opt[0] === '1');
const finishIndex = commands.findIndex(c => c.cmd === 9990);
const tutorialIndex = commands.findIndex(c => c.cmd === 7100);
assert(flagIndex >= 0 && finishIndex > flagIndex && tutorialIndex > finishIndex);
assert.equal(commands[tutorialIndex].target, 32);
assert(commands.some(c => c.cmd === 70 && c.target === 25304 && c.opt[0] === '1'));
assert(rejoinCommands.some(c => c.cmd === 70 && c.target === 25304 && c.opt[0] === '0'));
assert(nextCommands.some(c => c.cmd === 1000 && c.opt[0] === 'MS_TOU_10_0500'));
const placement = table('Placement/Database/PlacementData');
const mapLabels = table('Placement/Database/PlacementList').find(r => r.id === 'Dng_Cty_3_1').LabelList;
const newlyActive = mapLabels.map(id => placement.find(r => r.id === id))
  .filter(r => r.PlacementType === 1 && r.SpawnStartFlag === 8003 && r.SpawnEndFlag === 8004);
assert.equal(newlyActive.length, 2);
const npcs = table('Character/Database/NPCData');
const tutorial = table('Tutorial/Database/TutorialFlagPart').find(r => r.ID === 32);
assert.equal(tutorial.id, 'eFIELD_COMMAND_NIGHT_TOU');
const page = table('Tutorial/Database/TutorialPageTable').find(r => r.id === tutorial.PageDataLabel[0]);
const level = table('Level/Database/LevelTable').find(r => r.id === 'Dng_Cty_3_1');
const sequenceName = 'Event/RichEvent/01_Main/70_TOU/ep1/RI_MS_TOU_10_0400';
const sequence = audit(sequenceName);
const sequencePackage = s.assets.loadPackage(path.join(s.assets.CONTENT, sequenceName + '.uasset'));
sources.push({ source: '/Game/' + sequenceName, sha256: s.assets.hash(sequencePackage.b) });
const calls = [...new Set(sequence.flatMap(e => [...e.body.matchAll(/LocalVirtualFunction (\S+)/g)].map(m => m[1])))];
const result = {
  status: 'Source-derived expectation; no live checkpoint-to-tutorial capture yet',
  boundary: 'After MS_TOU_10_0300, reach MS_TOU_10_0400 without an encounter, skip it, dismiss Ambush tutorial; stop before Ambush',
  expected_crt_draws: 4,
  expected_order: [
    ...newlyActive.map(r => ({ kind: 'field_npc_start_frame', placement: r.id,
      resource: r.ResourceLabel, action: npcs.find(n => n.id === r.ResourceLabel).ActionID, draws: 1 })),
    { kind: 'counter_random_b', draws: 1 },
    { kind: 'counter_random_a', draws: 1 }
  ],
  tutorial_extra_draws: 0,
  counter: { min: level.EncountStepMin, range_b: level.RandomStepB, range_a: level.RandomStepA,
    first_ratio: level.FirstEncountStepRatio },
  event, commands, tutorial, page,
  new_npc_placements: newlyActive,
  sequence: { event_keys: sequence.length, calls },
  next_events: { ambush: nextCommands, rejoin: rejoinCommands },
  chapter_event_manifest: events.filter(r => /^MS_TOU_10_/.test(r.id)).map(r => ({
    event: r.id, map_id: r.MapID, skip: r.EventSkip, encounter_reset: r.EncountReset,
    encounter_trigger: r.EncountTrigger
  })),
  native_evidence: {
    finish_pre: 'FUN_140e8e740: guard/set +0x519; refresh placements through FUN_140ed58b0 before Blueprint reset',
    placements: 'FUN_140ed5040: iterate PlacementList; existing NPC -> refresh, missing eligible NPC -> SpawnPlacementObject',
    tutorial_open: 'command 7100 -> FUN_1410f7e30 -> OpenTutorial -> FUN_140f67460 -> FUN_140f67070',
    tutorial_close: 'OnTutorialFinish -> FUN_140f66900 -> FUN_140f6c860; no separate event/counter reset'
  },
  blueprint_evidence: [
    'BPC_LevelManager_Others.SpawnNPC @1677: RandomIntegerInRange(0,15), once per new field NPC',
    'EventManagerBP.FinishPre @66 native cleanup; @3342 EncountReset; @3470 ResetEncountStep',
    'EventManagerBP.FinishEvent @80: calls FinishPre only if ExecFinishPre is false',
    'EncountManagerBP.ResetEncountStep: current map parameters; CalcEncountStepCount @130 B, @167 A'
  ],
  sources
};
fs.writeFileSync(path.join(__dirname, '../data/source/throne-checkpoint.json'), JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ event: event.id, expected_crt_draws: result.expected_crt_draws,
  npc_actions: result.expected_order.filter(r => r.action).map(r => r.action), tutorial: tutorial.id,
  sequence_keys: sequence.length, status: result.status }));
