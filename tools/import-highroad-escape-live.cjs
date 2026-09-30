'use strict';
// Preserve the exact observed calls, not an endpoint-only offset fixture.
const fs = require('node:fs'), path = require('node:path'), assert = require('node:assert/strict');
const {createHash} = require('node:crypto');
const root = path.resolve(__dirname, '../..');
const source = process.argv[2];
assert(source, 'Pass the completed live capture JSON path relative to the workspace');
const bytes = fs.readFileSync(path.join(root, source)), capture = JSON.parse(bytes);
assert.equal(capture.finish, 'Detached with original debug registers restored');
assert.equal(capture.initialState, 0x5d64baea);
assert.equal(capture.finalState, 0x2e4773fc);
assert.equal(capture.records.length, 10);
let state = capture.initialState;
const actors = require('../data/town-loading.json').maps.Fld_Cty_1_3.environment_draws;
assert.equal(actors.length, 2);
const records = capture.records.map((r, i) => {
  assert.equal(r.kind, 'rand'); assert.equal(r.position, 6002+i); assert.equal(r.before, state);
  state = (Math.imul(state,214013)+2531011)>>>0;
  const node = r.blueprint.node;
  if(i<5) assert(node.endsWith('BP_BattleVoiceComponent_C.PlayBattleVoice'));
  else if(i===5) assert(node.endsWith('LibBattle_C.CheckEscapeSuccess'));
  else if(i<8) {
    assert(node.endsWith('BP_EnvObjMD_Canoe_A_C.ExecuteUbergraph_BP_EnvObjMD_Canoe_A'));
    assert(r.blueprint.obj.endsWith('.'+actors[i-6].actor));
  } else {
    assert(node.endsWith('EncountManagerBP_C.CalcEncountStepCount'));
    assert.equal(r.maximum,400);
  }
  return {position:r.position,before:r.before,after:state,returnRva:r.returnRva,
    blueprint:node,object:r.blueprint.obj,maximum:r.maximum};
});
assert.equal(state,capture.finalState);
fs.writeFileSync(path.join(__dirname,'../data/source/live-highroad-escape-6002.json'),JSON.stringify({
  source,sha256:createHash('sha256').update(bytes).digest('hex'),
  initial_advances:6002,initial_state:capture.initialState,
  final_advances:6012,final_state:capture.finalState,
  note:'Five 0→1→0 boost cycles; successful first Escape; wait for field control without moving. Two existing Highroad gondolas begin play again before counter B/A. Trace detached cleanly; all calls are continuous.',
  records
},null,2)+'\n');
console.log('Verified all 10 calls: five boosts, Escape, two gondolas, counter B/A.');
