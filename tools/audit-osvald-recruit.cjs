'use strict';
// Reproducible source audit: recruitment script, map actors and their parents.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const s=require('./source.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const level=s.table('Level/Database/LevelTable').rows.find(r=>r.id==='Fld_Snw_1_2');
const names=[level.id,...level.BattleMapNameList.filter(n=>n!=='None')];
const files=new Map();
function walk(dir){for(const e of fs.readdirSync(dir,{withFileTypes:true})) {
 const file=path.join(dir,e.name);if(e.isDirectory())walk(file);else if(e.name.endsWith('.umap'))files.set(e.name.toLowerCase(),file);
}}
walk(path.join(s.assets.CONTENT,'Level/Level'));
const packages=[],classes=new Set(),actors=[];
for(const name of names)for(const suffix of ['','_Art','_Coll','_GD','_Sound']) {
 const file=files.get((name+suffix+'.umap').toLowerCase());if(!file)continue;
 const actualName=path.basename(file,'.umap');
 const p=s.assets.loadPackage(file,true);
 packages.push({level:actualName,sha256:s.assets.hash(fs.readFileSync(file))});
 for(const e of p.exports){const c=s.assets.objectPath(p,e.classIndex);if(c?.startsWith('/Game/')){classes.add(c);actors.push({level:actualName,actor:e.name,class:c});}}
}
const audited=new Map();
function inspect(c){if(!c?.startsWith('/Game/')||audited.has(c))return;
 const asset=c.slice(6).split('.')[0],p=s.assets.loadPackage(path.join(s.assets.CONTENT,asset+'.uasset'));
 const generated=p.exports.find(e=>s.assets.objectPath(p,e.classIndex)==='/Script/Engine.BlueprintGeneratedClass');
 const parent=generated?s.assets.objectPath(p,generated.superIndex):null;
 const b=blueprint(asset);audited.set(c,{...b,parent});inspect(parent);
}
for(const c of classes)inspect(c);
const event=s.table('Event/Database/EventList').rows.find(r=>r.id==='MS_GAK_00_0100');
assert.equal(event.MapID,'152');assert(event.EncountReset);
const raw=fs.readFileSync(path.join(s.assets.CONTENT,'Event/json',event.id));
const commands=Object.entries(JSON.parse(raw)).map(([index,c])=>({index:Number(index),...c}));
const manager=blueprint('Event/BP/EventManagerBP',['StartAddMember','UpdateAddMember','StartAddMemberDialog','UpdateAddMemberDialog','StartAddMemberEnd','UpdateAddMemberEnd','AddMemberDialogFinishCallback','AddMemberFinishCallback','MemberCountJump','SpawnPlayerCharacter','FinishPre','StartReminisceneDialog','UpdateReminiscenceDialog','StartBeginReminiscene','StartEndReminiscene']);
fs.writeFileSync(path.join(__dirname,'../data/source/osvald-recruit-functions.json'),JSON.stringify(manager,null,2)+'\n');
const party=blueprint('Library/LibCharacter',['RefreshPartyAfterChangeMember','RefreshPartyAll']);
fs.writeFileSync(path.join(__dirname,'../data/source/osvald-party-functions.json'),JSON.stringify(party,null,2)+'\n');
const nativeText=fs.readFileSync(path.join(s.root,'Octopath_Traveler2-Win64-Shipping.exe.c'),'utf8');
const native={};
for(const name of ['FUN_1410bd860','FUN_140f0d690','FUN_140f0d6d0','FUN_140f0c7f0','FUN_1410bcc10','FUN_140f04de0','FUN_14103a780','FUN_140e67700','FUN_140e67040','FUN_140eff200']) {
 const match=new RegExp('^[A-Za-z_][^\\r\\n]* '+name+'\\(','m').exec(nativeText);assert(match,name);
 native[name]=nativeText.slice(match.index,nativeText.indexOf('\n}',match.index)+4);
}
const out={level,event,commands,script_sha256:s.assets.hash(raw),packages,actors,blueprints:[...audited.values()],native,
 route:{help:'15 Yes ->16',join:'61 Yes ->62, 3000 joins character 6',prologue:'70 No ->76 (tutorial 66)',end:'3005 finds member active; no prologue flag 2900 so EndPrologueReminiscence returns',variables:'7410 adds 1 to event variable 1; 7420 compares equality to 8. First recruit follows != branch ->110 ->131.',finish:'131..143 apply joined/skip-prologue flags;144 FinishPre refreshes placements and resets B/A;146 tutorial71 does not reset again.',randomness:'7 new streamed levels, including lowercase Fld_Snw_1_2_sound; ordinary eligible NPC frames; outgoing and incoming B/A reset. Map actors only use baked construction/random streams; recruitment fixed dialogue and joins have no CRT lottery.'}};
fs.writeFileSync(path.join(__dirname,'../data/source/osvald-recruit-audit.json'),JSON.stringify(out,null,2)+'\n');
console.log(JSON.stringify({packages,random_functions:[...audited.values()].flatMap(b=>Object.entries(b.functions).filter(([,body])=>/Random|rand|FRand/.test(body)).map(([name,body])=>({asset:b.asset,name,calls:body.split('\n').filter(l=>/Random|rand|FRand/.test(l))})))},null,2));
