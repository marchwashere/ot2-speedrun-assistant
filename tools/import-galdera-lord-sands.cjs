'use strict';
const fs=require('fs'),path=require('path'),assert=require('assert/strict'),s=require('./source.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const root=path.join(__dirname,'../data'),p=path.join(root,'town-loading.json'),world=JSON.parse(fs.readFileSync(p));const rows=s.table('Event/Database/EventList').rows;
const names=['EV_Dng_Dst_2_2_0010','EV_Dng_Dst_2_2_0020'];
for(const name of names){const row=rows.find(r=>r.id===name);const raw=JSON.parse(fs.readFileSync(path.join(s.assets.CONTENT,'Event/json',name)));world.events[name]={map_id:0,reset_counter:row.EncountReset,completion:null,commands:Object.entries(raw).map(([i,c])=>({index:Number(i),cmd:c.cmd,target:c.target,text:c.text,opt:c.opt}))};}
assert.deepEqual(world.events[names[0]].commands[1].opt,['EVM_DNG_DST_2_2_L',names[1],'0','1','0']);assert.equal(world.events[names[1]].commands[0].target,40010);
const volume=s.table('Battle/Database/EncountVolumeData').rows.find(r=>r.id==='EVM_DNG_DST_2_2_L');const encounter=s.table('Battle/Database/BattleEncounterData').rows.find(r=>r.id===volume.EncounterList[0].EncounterName);assert.equal(encounter.Group[0].GroupID,'ENG_DNG_DST_2_2_L_110');
fs.writeFileSync(p,JSON.stringify(world,null,2)+'\n');
fs.writeFileSync(path.join(root,'source/galdera-lord-sands.json'),JSON.stringify({events:Object.fromEntries(names.map(n=>[n,world.events[n]])),volume,encounter,scripted_start:blueprint('Battle/BP/EncountManagerBP',['StartEventBattle','StartEventBattleByEncountVolume']),night:blueprint('Battle/BP/BattleManagerBP',['ExecSpecialAction_BuffInNight','ExecSpecialAction_DebuffInNight'])},null,2)+'\n');
console.log('Imported boss prompt and victory flag40010; fixed first group, no encounter lottery');
