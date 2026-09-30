'use strict';
// Source audit/prediction only; does not attach to the game or extend the GUI.
const fs = require('node:fs'), path = require('node:path'), assert = require('node:assert/strict');
const s = require('./source.cjs');
const d = require('../../OT2BattleEngine/tools/kismet_decoder.cjs');
const sources = [];
function table(name) {
  const t = s.table(name); sources.push({source:t.source,sha256:t.sha256}); return t.rows;
}
function blueprint(asset, names) {
  const file = path.join(s.assets.CONTENT, asset + '.uasset');
  const p = s.assets.loadPackage(file), q = d.loadPackage(file);
  const schemas = s.extract.extractPackage(p, file).schemas;
  sources.push({source:'/Game/'+asset,sha256:s.assets.hash(p.b)});
  return Object.fromEntries(names.map(name => {
    const ex = p.exports.find(e => e.name === name);
    const schema = schemas.find(e => e.exportIndex === ex.index);
    const decoded = d.decodeFrom(q, name, schema.scriptStart-ex.serialOffset, true);
    assert.equal(decoded.exprs.at(-1).name, 'EndOfScript');
    return [name, decoded.exprs.map(e => d.flatten(e).join('\n')).join('\n')];
  }));
}
const placements = table('Placement/Database/PlacementData');
const placement = placements.find(r => r.id === 'NPC_TOU_10_0500_D000');
assert.equal(placement.EventType_A, 'eFC_PURCHASE_ITEM');
assert.equal(placement.EventParam_A_1, 'ITM_TRE_TOU_10_0010');
assert.equal(placement.EventEndFlag_A, 8013);
assert.equal(placement.EventLabel_A, 'MS_TOU_10_1400');
const npc = table('Character/Database/NPCData').find(r => r.id === placement.ResourceLabel);
const purchase = table('Character/Database/NPCPurchaseData').find(r => r.id === npc.FCmd_Purchase_ID);
const shop = table('Shop/Database/ShopList').find(r => r.id === purchase.ShopID);
assert.equal(shop.LabelList.length, 1);
const item = table('Shop/Database/PurchaseItemTable').find(r => r.id === shop.LabelList[0]);
assert.equal(item.ItemLabel, placement.EventParam_A_1);
const params = table('GameParam/GameParamDefineTable');
const threshold = params.find(r => r.id === 'STEAL_PROBABILITY_LEVEL_OVER');
const probability = params.find(r => r.id === 'STEAL_PROBABILITY_OVER');
assert(2-item.ProperSteal >= Number(threshold.Init));
assert.equal(Number(probability.Init), 100);
const event = table('Event/Database/EventList').find(r => r.id === placement.EventLabel_A);
assert.equal(event.EncountReset, false);
assert.equal(event.MapID, '61');
const raw = fs.readFileSync(path.join(s.assets.CONTENT, 'Event/json', event.id));
sources.push({source:'/Game/Event/json/'+event.id,sha256:s.assets.hash(raw)});
const commands = Object.values(JSON.parse(raw));
assert.deepEqual(commands.map(c=>c.cmd), [40,40,71,70,8030,7100,10000]);
assert.equal(commands.find(c=>c.cmd===7100).target, 17);
const changedFlags = commands.filter(c=>[70,71].includes(c.cmd)).map(c=>c.target);
assert.equal(placements.filter(p=>p.PlacementType===1 &&
  (changedFlags.includes(p.SpawnStartFlag)||changedFlags.includes(p.SpawnEndFlag))).length, 0);
const voiceRows = table('Talk/Database/TalkVoice_EN');
const voices = commands.filter(c=>c.cmd===40).map(c=>voiceRows.find(v=>v.id===c.text));
assert(voices.every(v=>v && v.Voice.length===1));
const ui = blueprint('UserInterface/FieldCommand/BP/FieldCommandWidgetSteal', ['StealItemExec','UpdateStealConfirm']);
assert.equal((ui.StealItemExec.match(/CallMath .*:RandomIntegerInRange/g)||[]).length, 1);
const flow = blueprint('FieldCommand/BPv2/Flow/BP_FCv2_FieldCommandFlow_Steal', ['UpdateFlow','PreProcess']);
assert(flow.UpdateFlow.indexOf('GetPlayableEventLabelByGotItems') < flow.UpdateFlow.indexOf('DrawEventLabel'));
const base = blueprint('FieldCommand/BPv2/Flow/BP_FCv2_FieldCommandFlowBase', ['GetPlayableEventLabelByGotItems']);
// The menu timer was separately audited in docs/throne-ambush.md. The same
// common manager path precedes Steal's item UI and success-event dispatch.
let state = 0;
const next = () => state = (Math.imul(state,214013)+2531011)>>>0;
for(let n=0;n<5467;n++) next();
assert.equal(state, 0xb8e6f54d);
const start = state, menu = next(), stolen = next();
const result = {
  status:'Final endpoint 5469 confirmed by the user on 2026-09-28. Individual call attribution is source-derived, not a live per-call capture. One menu opening and one clothes theft, without intervening actions/time changes.',
  boundaries:[{label:'Before menu',advances:5467,state:start},{label:'Menu open',advances:5468,state:menu},{label:'Clothes stolen; 1400 dialogue and Save tutorial dismissed',advances:5469,state:stolen}],
  expected_draws:2,
  reasons:['Path-action action timer: one CRT rand even at zero timer variance.','StealItemExec always performs RandomIntegerInRange(1,100), including 100% success.','Item-specific story event 1400 bypasses the generic success-dialogue lottery. EncountReset=false, no new NPC, fixed voices and tutorial 17 add no further draws.'],
  placement, npc, purchase, item, threshold, probability, event, commands, voices,
  blueprint:{ui,flow,base},sources,
};
fs.writeFileSync(path.join(__dirname,'../data/source/throne-clothes-steal.json'),JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify({status:result.status,draws:result.expected_draws,boundaries:result.boundaries.map(b=>({label:b.label,advances:b.advances,hex:'0x'+b.state.toString(16).toUpperCase().padStart(8,'0')}))}));
