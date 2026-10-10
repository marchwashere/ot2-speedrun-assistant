'use strict';
// Reproduce the first Hikari recruitment chain from the local cooked source.
// This is evidence only: runtime map rules and event commands stay in the shared catalog.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const s=require('./source.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const catalogFile=path.join(__dirname,'../data/town-loading.json');
const catalogBytes=fs.readFileSync(catalogFile),catalog=JSON.parse(catalogBytes),sources=[];
function table(name) {
  const t=s.table(name);sources.push({source:t.source,sha256:t.sha256});return t.rows;
}
function one(rows,label) {const r=rows.find(r=>r.id===label);assert(r,label);return r;}
function flagRanges(values) {
  assert.equal(values.length%2,0);const result=[];
  for(let i=0;i<values.length;i+=2) {
    const first=values[i],last=values[i+1];if(first===0&&last===0)continue;
    assert(first>0&&last>=first);result.push([first,last]);
  }
  return result;
}
const levels=table('Level/Database/LevelTable');
const lists=table('Placement/Database/PlacementList');
const placementRows=table('Placement/Database/PlacementData');
const npcRows=table('Character/Database/NPCData');
const events=table('Event/Database/EventList');
const volumes=table('Battle/Database/EncountVolumeData');
const encounters=table('Battle/Database/BattleEncounterData');
const groups=table('Battle/Database/EnemyGroupData');
const stories=table('Story/Database/MainStory');
const tutorials=table('Tutorial/Database/TutorialFlagPart');
const scripts={};
for(const suffix of ['0100','0200','0300','0310','0400']) {
  const label='MS_KEN_00_'+suffix,row=one(events,label);
  const raw=fs.readFileSync(path.join(s.assets.CONTENT,'Event/json',label));
  sources.push({source:'/Game/Event/json/'+label,sha256:s.assets.hash(raw)});
  const commands=Object.entries(JSON.parse(raw)).map(([index,c])=>({index:Number(index),cmd:c.cmd,target:c.target,text:c.text,opt:c.opt}));
  assert(commands.every((c,i)=>c.index===i),label);
  assert.equal(row.MapID,'28');assert.equal(row.EncountReset,true);
  assert.deepEqual(catalog.events[label].commands,commands,label+' imported commands');
  scripts[label]={event:row,commands};
}
function command(event,index,cmd,target) {
  const c=scripts[event].commands[index];assert.equal(c.cmd,cmd,event+' @'+index);
  if(target!==undefined)assert.equal(c.target,target,event+' @'+index);return c;
}
const first='MS_KEN_00_0100',repeat='MS_KEN_00_0200',victory='MS_KEN_00_0300';
const acceptBattle=command(first,47,5010,0);
assert.deepEqual(acceptBattle.opt.slice(0,4),['eDIALOG_YES','47','eDIALOG_NO','67']);
assert.equal(command(first,48,70,37501).opt[0],'1');
const battleStart=command(first,66,600,0);
assert.deepEqual(battleStart.opt,['ENM_EVE_WAR_C00_010',victory,'0','1','0']);
assert.deepEqual(command(repeat,46,600,0).opt,battleStart.opt);
// cmd600 names an encounter VOLUME, not an EnemyGroupData row.
const volume=one(volumes,battleStart.opt[0]);
assert.equal(volume.StartFlag,0);assert.equal(volume.EndFlag,0);
assert.equal(volume.RareCatlinParam.RareEnemyFlag,false);
assert.equal(volume.RareOctlinParam.RareEnemyFlag,false);
const tiers=volume.EncounterList.filter(r=>r.EncounterName!=='None');assert.equal(tiers.length,1);
assert.equal(tiers[0].ProgressBorder,0);
const progress=4,selectedTier=tiers.filter(r=>r.ProgressBorder<=progress).at(-1);assert(selectedTier);
const encounter=one(encounters,selectedTier.EncounterName);
const choices=encounter.Group.filter(r=>r.Appearance>0);assert.equal(choices.length,1);
assert.equal(choices[0].Appearance,100);assert.equal(encounter.FirstEncount,1);
const group=one(groups,choices[0].GroupID);
assert.equal(group.id,'ENG_EVE_WAR_C00_010');assert.equal(group.EscapeEnable,false);
assert.deepEqual(group.EnemyID.filter(x=>x!=='None'),['ENE_EVE_WAR_DST_090','ENE_EVE_WAR_DST_080','ENE_EVE_WAR_DST_080']);
const invitation=command(victory,127,3010,1);assert.deepEqual(invitation.opt,['127','201']);
command(victory,133,3000,1);
assert.deepEqual(command(victory,138,3120,1).opt,['138','142']);
command(victory,143,7100,66);command(victory,148,3005,1);
command(victory,179,9990);command(victory,181,7100,67);
const finalFlags=scripts[victory].commands.slice(166,179);
assert(finalFlags.every(c=>c.cmd===70&&c.opt[0]==='1'));
assert.deepEqual(finalFlags.map(c=>c.target),[37520,37502,5022,30,25558,1007,1009,1014,1011,1307,1309,1314,1311]);
const entryPlacement=one(placementRows,'NPC_FENCER_0000_0000');
assert.equal(entryPlacement.PlacementType,3);
assert.equal(entryPlacement.EventLabel_A,first);
assert.deepEqual([entryPlacement.SpawnStartFlag,entryPlacement.SpawnEndFlag],[9000,37520]);
const town=one(levels,'Twn_Dst_1_1_A');assert.equal(town.ID,28);
const northern=one(levels,'Fld_Dst_1_1');assert.equal(northern.ID,19);
const townLabels=one(lists,town.id).LabelList;
assert.deepEqual(catalog.maps[town.id].placements,townLabels);
const recruitmentPlacements=placementRows.filter(p=>townLabels.includes(p.id)&&p.id.startsWith('NPC_FENCER_0000_'));
const recruitmentNpcs=[...new Set(recruitmentPlacements.filter(p=>p.PlacementType===1).map(p=>p.ResourceLabel))].map(label=>one(npcRows,label));
const prologue=one(stories,'MS_KEN_00');assert.equal(prologue.ID,100);assert.equal(prologue.ClearDifficultyPoint,1);
const completion={scenario:prologue.ID,character:prologue.PlayerCharaID,points:prologue.ClearDifficultyPoint,
  on_flag_ranges:flagRanges(prologue.EventFlagRange),off_flag_ranges:flagRanges(prologue.OffEventFlagRange)};
assert.deepEqual(catalog.prologue_completions[1],[completion]);
const tutorialRows=[66,67].map(id=>{const t=tutorials.find(r=>r.ID===id);assert(t);return t;});
for(const t of tutorialRows)assert.deepEqual(catalog.tutorials[t.ID],{label:t.id,list_flag:t.TutorialListFlag,opened_flag:t.TutorialOpenedFlag});
const eventBlueprint=blueprint('Event/BP/EventManagerBP',[
  'StartBattleStart','UpdateBattleStart','StartAddMember','UpdateAddMember',
  'StartAddMemberDialog','AddMemberDialogFinishCallback','StartReminisceneDialog',
  'AddReminiscenceDialogFinishCallback','StartAddMemberEnd','UpdateAddMemberEnd',
  'AddMemberFinishCallback','FinishPre','FinishEvent','FinishEventSkip']);
sources.push({source:'/Game/'+eventBlueprint.asset,sha256:eventBlueprint.sha256});
assert(eventBlueprint.functions.StartBattleStart.includes('LocalVirtualFunction StartEventBattle'));
assert(eventBlueprint.functions.UpdateBattleStart.includes('BattleEndNextEventName'));
assert(eventBlueprint.functions.AddReminiscenceDialogFinishCallback.includes(':SkipPrologueReminiscence'));
assert(eventBlueprint.functions.UpdateAddMemberEnd.includes('LocalVirtualFunction OpenPartyCharacterChangeMenu'));
const fieldBlueprint=blueprint('Level/BP/BPC_LevelManager_Others',['On Shown Field After Battle Impl']);
sources.push({source:'/Game/'+fieldBlueprint.asset,sha256:fieldBlueprint.sha256});
const fieldReturn=fieldBlueprint.functions['On Shown Field After Battle Impl'];
assert(fieldReturn.includes('LocalVirtualFunction ResetEncountStep'));
assert(fieldReturn.includes(':RefreshPartyCharacterFormation'));
const nativeFile='Octopath_Traveler2-Win64-Shipping.exe.c';
const nativeText=fs.readFileSync(path.join(s.root,nativeFile),'utf8'),native={};
for(const name of ['FUN_140f0d6d0','FUN_140f0c7f0','FUN_140f15190','FUN_140f35f50','FUN_140f54e00','FUN_140f4ec50','FUN_140f537f0']) {
  const match=new RegExp('^[A-Za-z_][^\\r\\n]* '+name+'\\(','m').exec(nativeText);assert(match,name);
  const end=nativeText.indexOf('\n}',match.index);assert(end>match.index);
  const body=nativeText.slice(match.index,end+2);
  native[name]={source:nativeFile,function:name,sha256:s.assets.hash(Buffer.from(body)),body};
}
assert(native.FUN_140f15190.body.includes('FUN_140f35f50'));
const out={
  scope:'First Hikari introduction in Ryu, the mandatory event battle, then invitation and skipping Chapter 1. Source evidence for a four-member party; no battle or map draw total is hardcoded.',
  initial_context:{map:town.id,map_id:28,approach_map:northern.id,approach_map_id:19,time:2,party:[7,6,8,4],bench:[],main_progress_points:progress,active_scenarios:[701],first_contact:true},
  sources,
  maps:{northern_ryu:northern,ryu:town},
  placement:entryPlacement,recruitment_placements:recruitmentPlacements,recruitment_npcs:recruitmentNpcs,
  battle:{command:battleStart,volume,selected_tier:selectedTier,encounter,group,
    label_resolution:'Event cmd600 opt[0] -> EncountVolumeData -> selected EncounterList row -> BattleEncounterData -> EnemyGroupData. ENM/ENC/ENG are distinct table keys.',
    next_event:battleStart.opt[1],continuation:'Victory resumes MS_KEN_00_0300 through EventManagerBP.UpdateBattleStart; do not independently FinishPre the suspended MS_KEN_00_0100 before this continuation.',
    field_return:'On Shown Field After Battle Impl: EncountType1 selects720, then the victory path1425 ->1587 RefreshPartyCharacterFormation ->754 ->826 ResetEncountStep. This real field-return counter reset is BEFORE the0300 continuation and distinct from its later FinishPre at179.'},
  choices:{first_battle:{event:first,index:47,answer:'eDIALOG_YES',next_index:48},
    join:{event:victory,index:127,answer:'YES',next_index:128},
    skip_chapter:{event:victory,index:138,answer:'NO',next_index:143},
    first_finish_pre:{event:victory,index:179},
    tutorials:[{event:victory,index:143,id:66},{event:victory,index:181,id:67}]},
  party_join:{character:1,add_member_command:133,finish_command:148,
    initial_full_party:[7,6,8,4],replacement:'Explicit player choice is required at the full-party change menu; do not append a fifth active member or silently select who is benched.',
    callback:'EventManagerBP.StartAddMemberEnd reads MainMemberID. UpdateAddMemberEnd can open OpenPartyCharacterChangeMenu and then checks the chosen active membership.',
    native_join:'JoinPlayerCharacter f0d6d0 adds the newly recruited character to the bench when MainMemberID is full; f0c7f0 writes the resulting party arrays.',
    party_change:'PartyList.RefreshMainMember in AddFriendMode invokes RefreshBarChangeCharacter rather than the ordinary RefreshSpawnCurrentLevel. Recompute active-party support skills after the chosen swap; do not keep an Evasive Maneuvers flag if its equipped character leaves the active party.',
    progression_limit:'Native join array updates do not themselves write EXP/level. Audit evidence here does not infer an average-party recruitment level.'},
  prologue,completion,
  completion_callback:{native:'f15190 -> f54e00 -> f4ec50 -> f537f0',
    effect:'The actual NO/SkipPrologueReminiscence completion applies the inclusive on/off ranges and the one-time ClearDifficultyPoint. Mere generic point bookkeeping must not apply these flags.',
    equipment:'f15190 removes all equipment, grants the fixed skip-prologue items and calls f35f50 (SetFirstEquipment). Those operations must not be mistaken for an average-party EXP or level grant.'},
  final_flags:finalFlags,tutorials:tutorialRows,
  event_blueprint:eventBlueprint,field_blueprint:fieldBlueprint,native,scripts,
  validation:'All selected rows, aliases, branch destinations, imported commands, tutorial flags and prologue ranges are checked against cooked tables. New battle and full-party endpoints still require retail validation.'
};
const destination=path.join(__dirname,'../data/source/hikari-recruitment-audit.json');
fs.writeFileSync(destination,JSON.stringify(out,null,2)+'\n');
assert(fs.readFileSync(catalogFile).equals(catalogBytes),'Audit changed the runtime catalog');
console.log(JSON.stringify({file:destination,sources:sources.length,events:Object.keys(scripts).length,
  volume:volume.id,encounter:encounter.id,group:group.id,enemies:group.EnemyID.filter(x=>x!=='None'),
  party_join:1,skip_scenario:completion.scenario,progress_before:progress,progress_after:progress+completion.points,catalog_unchanged:true}));
