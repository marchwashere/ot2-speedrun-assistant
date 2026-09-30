'use strict';
// Route-only provenance: boss data and combat remain in the imported core.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const s=require('./source.cjs'),{audit}=require('../../OT2BattleEngineRust/tools/audit-sequence.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const events=s.table('Event/Database/EventList');
const event=events.rows.find(r=>r.id==='MS_TOU_10_1800');
assert.equal(event.MapID,'50');assert.equal(event.EncountReset,true);
const raw=fs.readFileSync(path.join(s.assets.CONTENT,'Event/json/'+event.id));
const commands=Object.values(JSON.parse(raw));
assert(commands.some(c=>c.cmd===71&&c.target===8017&&c.opt[0]==='1'));
const asset='Event/RichEvent/01_Main/70_TOU/ep1/RI_MS_TOU_10_1500';
assert(commands.some(c=>c.cmd===1500&&c.opt.includes('RI_MS_TOU_10_1500')));
const sequence=audit(asset),start=sequence.find(k=>/LocalVirtualFunction StartBattle\b/.test(k.body));
assert(start.body.includes('"EVM_BOS_THI_C01_010"'));
assert(start.body.includes('"MS_TOU_10_1900"'));
assert(!sequence.some(k=>/Random|SetTimer/.test(k.body)));
const hooks=[blueprint('Event/BP/BP_RichEventCommandBase',['StartBattleFunction']),blueprint('Event/BP/EventManagerBP',['StartBattleStart','UpdateBattleStart'])];
const decomp=fs.readFileSync(path.join(s.root,'Octopath_Traveler2-Win64-Shipping.exe.c'),'utf8');
const native=addr=>{const f=decomp.match(new RegExp('^.* FUN_'+addr+'\\([^;]*?\\n\\{[\\s\\S]*?\\n\\}','m'));assert(f,addr);return f[0];};
const handoff=native('140eb2700'),skip=native('140eb67a0'),parent=native('140e965b0');
assert(handoff.includes('0x6c8) = 600'));assert(handoff.includes('0x69e) = 1'));
const skipGuard=skip.slice(skip.indexOf('if (*(char *)(param_1 + 0x69e)'),skip.indexOf('bVar6 = false'));
assert(skipGuard.includes('return 0;'));
assert(!/140ff7430|140e8e740|ResetEncount|rand\(/.test(handoff+skipGuard+parent));
const result={status:'Source-derived; awaiting live validation',expected_crt_draws:0,
  boundary:'Pirro trigger -> skip 1800 -> before boss initialization; no map change or FinishPre',
  event,commands,sequence,start_battle:start,hooks,native:{handoff,skipGuard,parent},
  sources:[{source:events.source,sha256:events.sha256},{source:'/Game/Event/json/'+event.id,sha256:s.assets.hash(raw)},
    {source:'/Game/'+asset,sha256:s.assets.hash(s.assets.loadPackage(path.join(s.assets.CONTENT,asset+'.uasset')).b)}]};
fs.writeFileSync(path.join(__dirname,'../data/source/throne-pirro-transition.json'),JSON.stringify(result,null,2)+'\n');
console.log('1800: flag 8017, direct rich-event boss handoff, +0 before initialization.');
