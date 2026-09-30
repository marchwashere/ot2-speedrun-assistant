'use strict';
// Static route boundary audit; combat data is imported by the battle repository.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const s=require('./source.cjs'),{audit}=require('../../OT2BattleEngineRust/tools/audit-sequence.cjs');
const decoder=require('../../OT2BattleEngine/tools/kismet_decoder.cjs');
const sources=[];
function table(name){const t=s.table(name);sources.push({source:t.source,sha256:t.sha256});return t.rows;}
function blueprint(asset,name) {
  const file=path.join(s.assets.CONTENT,asset+'.uasset');
  const p=s.assets.loadPackage(file),q=decoder.loadPackage(file);
  const ex=p.exports.find(e=>e.name===name);assert(ex,name);
  const schema=s.extract.extractPackage(p,file).schemas.find(e=>e.exportIndex===ex.index);
  const decoded=decoder.decodeFrom(q,name,schema.scriptStart-ex.serialOffset,true);
  assert.equal(decoded.exprs.at(-1).name,'EndOfScript');
  sources.push({source:'/Game/'+asset,function:name,sha256:s.assets.hash(p.b)});
  return decoded.exprs.map(e=>decoder.flatten(e).join('\n')).join('\n');
}
const event=table('Event/Database/EventList').find(r=>r.id==='MS_TOU_10_0600');
assert.equal(event.MapID,'57');assert.equal(event.EncountReset,true);
const bytes=fs.readFileSync(path.join(s.assets.CONTENT,'Event/json',event.id));
const commands=Object.values(JSON.parse(bytes));
assert(commands.some(c=>c.cmd===8900));
sources.push({source:'/Game/Event/json/'+event.id,sha256:s.assets.hash(bytes)});
const asset='Event/RichEvent/01_Main/70_TOU/ep1/RI_MS_TOU_10_0600';
const keys=audit(asset);
sources.push({source:'/Game/'+asset,sha256:s.assets.hash(s.assets.loadPackage(path.join(s.assets.CONTENT,asset+'.uasset')).b)});
const start=keys.find(e=>/LocalVirtualFunction StartBattle\b/.test(e.body));
assert(start.body.includes('"EVM_EVE_THI_C01_020"'));
assert(start.body.includes('"MS_TOU_10_0700"'));
assert(!keys.some(e=>/Random|SetTimer/.test(e.body)));
// The reset flag alone says nothing about whether cleanup precedes combat.
const rich=blueprint('Event/BP/BP_RichEventCommandBase','StartBattleFunction');
const startBattle=blueprint('Event/BP/EventManagerBP','StartBattleStart');
const updateBattle=blueprint('Event/BP/EventManagerBP','UpdateBattleStart');
assert(rich.includes(':StartEventBattle'));
assert(startBattle.includes('VirtualFunction StartEventBattle'));
assert(startBattle.includes('InstanceVariable IsMainSeqStop'));
assert(!/FinishPre|ResetEncountStep/.test(rich+startBattle));
assert(updateBattle.includes('InstanceVariable BattleEndNextEventName'));
const decomp=fs.readFileSync(path.join(s.root,'Octopath_Traveler2-Win64-Shipping.exe.c'),'utf8');
function native(name) {
  const match=new RegExp('^[A-Za-z_][A-Za-z0-9_ *]* '+name+'\\(','m').exec(decomp);assert(match,name);
  const body=decomp.slice(match.index,decomp.indexOf('\n}',match.index)+2);
  sources.push({source:'Octopath_Traveler2-Win64-Shipping.exe.c',function:name,sha256:s.assets.hash(Buffer.from(body))});
  return body;
}
const handoff=native('FUN_140eb2700'),skip=native('FUN_140eb67a0'),parent=native('FUN_140e965b0');
assert(handoff.includes('0x6c8) = 600'));
assert(handoff.includes('0x69e) = 1'));
const skipGuard=skip.slice(skip.indexOf('if (*(char *)(param_1 + 0x69e)'),skip.indexOf('bVar6 = false'));
assert(skipGuard.includes('return 0;'));
assert(!/140ff7430|140e8e740|ResetEncount|rand\(/.test(handoff+skipGuard+parent));
const result={status:'Source-derived; awaiting live validation',
  boundary:'After skipped 0500 rejoin; walk without encountering; skip 0600; before ENG_EVE_THI_C01_020 actor initialization',
  expected_crt_draws:0,
  assumptions:['The short walk triggers no encounter, as specified by the route.','The existing shared level remains loaded.'],
  evidence:{finish_pre:'EncountReset is true, but is consumed by FinishPre at event cleanup, not automatically when the rich cutscene enters battle. The former +2 prediction incorrectly moved that cleanup before combat.',
    handoff:'BP_RichEventCommandBase.StartBattleFunction -> native StartEventBattle FUN_140eb2700: sets current command 600, dispatches StartBattleStart, sets handoff +0x69e. No FinishPre.',
    battle:'EventManagerBP.StartBattleStart -> EncountManager.StartEventBattle, sets IsMainSeqStop; UpdateBattleStart waits for combat and selects BattleEndNextEventName afterward.',
    skip:'FUN_140eb67a0 returns 0 immediately while the +0x69e rich battle handoff is active, rather than advancing to cleanup commands. Native StartBattleStart FUN_140e965b0 saves continuation state and does not reset the counter.',
    native_dispatch:'RichEventCommandBase vtable 0x143e9c508 +0x408 is 0x1408acef0 (ret 0, no work). EventManager vtable 0x143ca8a48 +0x938 is FUN_140e965b0.',
    unlock:'Native event command 8900 calls FUN_140f34b30(15,18,2,1), learning Pierce Through for Pirro.',
    progression:'Opening base EXP 4, minimum break bonus 1, minimum overkill bonus 1; CharacterGrow level 2 requires 6. Level-up restores HP/SP.'},
  limitations:'Authored scalar event keys and existing reset path; not a proof of every transitive runtime callback or timing-dependent reaction.',
  event,commands,start_battle:start,sequence_keys:keys.length,
  handoff_native:handoff,skip_guard:skipGuard,start_battle_native:parent,
  blueprint:{rich_start:rich,event_start:startBattle,event_update:updateBattle},sources};
fs.writeFileSync(path.join(__dirname,'../data/source/throne-second.json'),JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify({event:event.id,draws:result.expected_crt_draws,sequence_keys:keys.length}));
