'use strict';
// Reproducible source evidence; this does not attach to the running game.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const s=require('./source.cjs');
const {audit}=require('../../OT2BattleEngineRust/tools/audit-sequence.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const tables=['Event/Database/EventList','Level/Database/LevelTable','Story/Database/MainStory','Story/Database/MainStoryTask'].map(s.table);
const eventRows=tables[0].rows.filter(r=>['MS_TOU_10_1900','MS_TOU_10_2000'].includes(r.id));
assert.deepEqual(eventRows.map(r=>r.MapID),['50','61']);
const scripts=Object.fromEntries(eventRows.map(r=>[r.id,Object.values(JSON.parse(fs.readFileSync(path.join(s.assets.CONTENT,'Event/json',r.id))))]));
assert(eventRows.every(r=>r.EncountReset&&r.StartFadeOutFlag===1));
assert(scripts.MS_TOU_10_1900.some(c=>c.cmd===1000&&c.opt[0]==='MS_TOU_10_2000'));
assert(scripts.MS_TOU_10_2000.some(c=>c.cmd===1550&&c.opt[0]==='0'));
assert(scripts.MS_TOU_10_2000.some(c=>c.cmd===500&&c.target===46));
assert.equal(tables[1].rows.find(r=>r.ID===46).id,'Fld_Cty_1_3');
const last=tables[3].rows.filter(r=>r.MainStoryLabel==='MS_TOU_00'&&r.TaskType==='ESTORY_TASK_TYPE::NORMAL').at(-1);
assert.equal(last.id,'ML_TOU_10_2000');
const sequences=['1600','1700'].map(n=>{
  const asset='Event/RichEvent/01_Main/70_TOU/ep1/RI_MS_TOU_10_'+n;
  const keys=audit(asset);
  assert(!keys.some(k=>/Random|SetTimer|ChangeTimeZone|SetBitFlag|StartBattle/.test(k.body)));
  return {asset,sha256:s.assets.hash(s.assets.loadPackage(path.join(s.assets.CONTENT,asset+'.uasset')).b),keys};
});
const hooks=blueprint('Event/BP/EventManagerBP',['PlayEvent','PlayAfterFadeOut','UpdateMainMissionProgress','StartRefreshEventObject','FinishPre']);
const decomp=fs.readFileSync(path.join(s.root,'Octopath_Traveler2-Win64-Shipping.exe.c'),'utf8');
const native={};
for(const addr of ['140f32470','140f321d0','140f55560','140f4ec50','140f6aa40','140f6eb70','140ed4ea0']) {
  const m=decomp.match(new RegExp('^.* FUN_'+addr+'\\([^;]*?\\n\\{[\\s\\S]*?\\n\\}','m'));
  assert(m,addr);native[addr]=m[0];
}
assert(native['140f32470'].includes('plVar19 = &DAT_144cbe590'));
assert(native['140f321d0'].includes('iVar4 < iVar2 + -1'));
assert(native['140f55560'].includes('FUN_140f4ec50(param_1,uVar6,1)'));
assert(native['140f4ec50'].includes('FUN_140f36750(param_2,param_3 + 4)'));
assert(native['140f6eb70'].includes("if (param_3 == '\\x01') goto LAB_140f6ec65"));
const result={status:'Source-derived; awaiting retail validation',
  boundary:'After Pirro victory drops -> skip ending scenes -> Eastern New Delsta Highroad control',
  eventRows,scripts,last_task:last,sequences,hooks,native,
  facts:{
    battle_return:'Return counter B/A, then 1900. Suspended 1800 has no separate FinishPre pair.',
    completion:'PlayEvent updates mission before the fade/map callback. The normal-task successor map is keyed by MainStoryLabel; 2000 is final for MS_TOU_00. f55560 -> f4ec50 sets status 5, closing active-scenario/prologue placement gates before backstreets loading.',
    time:'2000 command1550 opt0 means Night(4) -> Dawn(1). f6eb70 refreshes old-phase placements, ages timers by one, and does not dispatch event0x27/Black Market lottery.',
    refresh8030:'Native ff dce0 FName at144f9f180 is StartRefreshEventObject (initializer140679a56). BP calls RefreshFlagObject -> ed4ea0 for flag-controlled objects, not a chapter setup or an NPC lottery.',
    endpoint:'2000 command500 loads map46, command230 positions existing player, command9990 refreshes dirty placements and resets the incoming counter; command52 fade adds no CRT call.',
  },sources:tables.map(t=>({source:t.source,sha256:t.sha256}))};
fs.writeFileSync(path.join(__dirname,'../data/source/pirro-aftermath.json'),JSON.stringify(result,null,2)+'\n');
console.log('Audited 1900 -> 2000 -> map46, chapter completion and Night-to-Dawn.');
