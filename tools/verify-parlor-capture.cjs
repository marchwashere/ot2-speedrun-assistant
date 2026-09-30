'use strict';
// Capture regression only. These counts are not runtime prediction inputs.
const fs = require('node:fs'), path = require('node:path'), assert = require('node:assert/strict');
const s = require('./source.cjs');
const file = path.join(__dirname, '../data/source/live-parlor-20260927.json');
const capture = JSON.parse(fs.readFileSync(file, 'utf8'));
const next = state => (Math.imul(state, 214013) + 2531011) >>> 0;
let state = 0;
for (let i = 0; i < 5374; i++) state = next(state);
assert.equal(capture.initialState, state);
assert.equal(capture.finish, 'Detached with original debug registers restored');
const categories = {counter: 0, level: 0, npc: 0, lottery: 0};
for (const [i, call] of capture.records.entries()) {
  assert.equal(call.kind, 'rand', `reseed at ${i}`);
  assert.equal(call.before, state, `CRT discontinuity at ${i}`);
  assert.equal(call.position, 5374 + i);
  state = next(state);
  switch (call.returnRva) {
    case '0x30D7424':
      assert(call.blueprint.node.endsWith('.CalcEncountStepCount'));
      assert.equal(call.maximum, 400); categories.counter++; break;
    case '0x11C010F': categories.level++; break;
    case '0x30D7604':
      assert(call.blueprint.node.endsWith('.SpawnNPC'));
      assert(call.blueprint.spawn.values.some(v => v.name === 'NPCLabel'));
      categories.npc++; break;
    case '0xEA7749': categories.lottery++; break;
    default: assert.fail(`unclassified CRT caller ${call.returnRva}`);
  }
}
assert.equal(state, capture.finalState);
assert.equal(state, 0x080e80b0);
assert.equal(capture.records.length, 50);
assert.deepEqual(categories, {counter: 14, level: 11, npc: 24, lottery: 1});

const rules = s.table('Flag/Database/FlagRandomControlDB').rows;
const lottery = capture.records.find(r => r.returnRva === '0xEA7749');
const bytes = Buffer.from(lottery.nativePointers.find(r => r.register === 'rdi').bytes, 'hex');
const matched = rules.filter(r => r.ID === bytes.readInt32LE(8)
  && r.ControlBeginFlag === bytes.readInt32LE(12) && r.ControlEndFlag === bytes.readInt32LE(16)
  && Number(r.ControlTrigger.endsWith('eTimeZoneChange')) === bytes[20]
  && Number(r.ApplyValue) === bytes[21]);
assert.deepEqual(matched.map(r => r.id), ['GMK_BLACKMARKET_CONTROL']);

const events = s.table('Event/Database/EventList').rows;
const prompt = Object.values(JSON.parse(fs.readFileSync(path.join(s.assets.CONTENT, 'Event/json/MS_TOU_10_0910'), 'utf8')));
assert.equal(events.find(r => r.id === 'MS_TOU_10_0910').EncountReset, false);
assert.deepEqual(prompt[0].opt.slice(0, 2), ['eDIALOG_YES', '0']);
assert(prompt.some(c => c.cmd === 1000 && c.opt[0] === 'MS_TOU_10_1000'));
const scripts = Object.fromEntries(['0900','1000','1100'].map(id => [id,
  Object.values(JSON.parse(fs.readFileSync(path.join(s.assets.CONTENT, 'Event/json/MS_TOU_10_' + id), 'utf8')))]));
for (const id of Object.keys(scripts)) assert(events.find(r => r.id === 'MS_TOU_10_' + id).EncountReset);
assert(scripts['1000'].some(c => c.cmd === 1550 && c.opt[0] === '1')); // Day, native enum 2.
assert(scripts['1000'].some(c => c.cmd === 1000 && c.opt[0] === 'MS_TOU_10_1100'));
assert.equal(events.find(r => r.id === 'MS_TOU_10_1100').MapID, '61');
assert(scripts['1100'].some(c => c.cmd === 1500 && c.opt[0] === 'RI_MS_TOU_10_1200'));

console.log(JSON.stringify({start: 5374, end: 5424, state: '0x080E80B0', draws: capture.records.length,
  categories, lottery: matched[0].id, continuity: 'all calls and final state match'}));
