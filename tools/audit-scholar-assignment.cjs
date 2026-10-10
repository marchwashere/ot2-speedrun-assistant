'use strict';
// Regenerate Scholar assignment/learning data from cooked tables. No menu or
// combat RNG total is inferred from the fact that these are menu operations.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const s=require('./source.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const sources=[];
function table(name) {const t=s.table(name);sources.push({source:t.source,sha256:t.sha256});return t.rows;}
function one(rows,label) {const row=rows.find(r=>r.id===label);assert(row,label);return row;}
const jobs=table('Character/Database/JobData');
const setRows=table('Ability/Database/AbilitySetData');
const abilityRows=table('Ability/Database/AbilityData');
const supportRows=table('Ability/Database/SupportAbilityData');
const ailmentRows=table('Ability/Database/AilmentData');
const guildRows=table('Shop/Database/GuildData');
const itemRows=table('Item/Database/ItemDB');
const eventRows=table('Event/Database/EventList');
const tutorialRows=table('Tutorial/Database/TutorialFlagPart');
const textFile=path.join(s.assets.CONTENT,'GameText/Database/GameTextEN.uexp');
const textBytes=fs.readFileSync(textFile);
sources.push({source:'/Game/GameText/Database/GameTextEN.uexp',sha256:s.assets.hash(textBytes)});
const texts={};
function text(key) {
  if(texts[key]!==undefined)return texts[key];
  const needle=Buffer.from(key+'_Text\0'),start=textBytes.indexOf(needle);assert(start>=0,key);
  const offset=start+needle.length,length=textBytes.readInt32LE(offset);assert(length!==0,key);
  return texts[key]=length>0?textBytes.toString('utf8',offset+4,offset+4+length-1)
    :textBytes.toString('utf16le',offset+4,offset+4+(-length-1)*2);
}
const job=one(jobs,'ePROFESSOR');assert.equal(job.ID,6);assert.equal(job.JobID,6);
assert.deepEqual(job.JPCost,[0,0,30,100,500,1000,3000,5000]);
const boostKeys=['NoBoost','BoostLv1','BoostLv2','BoostLv3'];
const includedSets=new Map(),includedAbilities=new Map();
function includeSet(label) {
  if(label==='None'||includedSets.has(label))return;
  const set=one(setRows,label);includedSets.set(label,set);
  for(const key of boostKeys) {
    const row=one(abilityRows,set[key]);includedAbilities.set(row.id,row);text(row.DisplayName);
  }
  includeSet(set.SuperMagicLabel);includeSet(set.HyperMagicLabel);
}
const abilities=job.JobCommandAbility.map(entry=>{
  includeSet(entry.AbilityName);const set=includedSets.get(entry.AbilityName);
  const ability=one(abilityRows,set.NoBoost);
  return {id:entry.AbilityName,name:text(ability.DisplayName),get_param:entry.GetParam,
    default:entry.GetParam===1,divine:set.IsDivineAbility,ability_rows:boostKeys.map(k=>set[k]),
    super_magic:set.SuperMagicLabel==='None'?null:set.SuperMagicLabel,
    hyper_magic:set.HyperMagicLabel==='None'?null:set.HyperMagicLabel};
});
const defaults=abilities.filter(a=>a.default).map(a=>a.id);
assert.deepEqual(defaults,['ABI_SET_SCH_010','ABI_SET_SCH_020']);
const selected=['ABI_SET_SCH_030','ABI_SET_SCH_060'];
assert.deepEqual(selected.map(id=>abilities.find(a=>a.id===id)?.name),['Lightning Bolt','Elemental Barrage']);
const supports=job.JobSupportAbility.map(entry=>{
  const row=one(supportRows,entry.AbilityName);
  return {id:row.id,name:text(row.DisplayName),support_ability_id:row.SupportAbilityID,
    learned_ability_count:entry.GetParam,effect:row.SupportAilmentType.split('::').at(-1),value:row.InvocationValue};
});
assert.deepEqual(supports.map(r=>r.learned_ability_count),[4,5,6,7]);
assert.equal(supports[0].id,'ABI_SCH_SUP_01');assert.equal(supports[0].support_ability_id,61);
assert.equal(supports[0].effect,'eENCOUNT_DOWN');assert.equal(supports[0].value,50);
assert.equal(supports[0].name,'Evasive Maneuvers');
const guild=one(guildRows,'GUILD_005'),license=one(itemRows,job.EquipJobItem);
assert.equal(guild.LicenseItem,license.id);assert.equal(license.MaxNum,3);
assert.deepEqual([guild.JobLicenseData[0].NeedItemLabel,guild.JobLicenseData[0].NeedItemNum,
  guild.JobLicenseData[0].NeedMoney,guild.JobLicenseData[0].NeedAbility,guild.JobLicenseData[0].LearnedCount],['None',0,0,'None',0]);
const scripts={};
for(const name of [guild.FistTaskEvent,guild.CompleteEvent[0]]) {
  const event=one(eventRows,name),raw=fs.readFileSync(path.join(s.assets.CONTENT,'Event/json',name));
  sources.push({source:'/Game/Event/json/'+name,sha256:s.assets.hash(raw)});
  const commands=Object.entries(JSON.parse(raw)).map(([index,c])=>({index:Number(index),...c}));
  assert.equal(event.EncountReset,false);scripts[name]={event,commands};
}
const complete=scripts.SYS_SCH_GUILD_0030.commands;
assert(complete.some(c=>c.cmd===70&&c.target===406&&c.opt[0]==='0'));
assert(complete.some(c=>c.cmd===70&&c.target===411&&c.opt[0]==='1'));
const runtime={
  job_id:job.JobID,label:job.id,name:text(job.DisplayName),permission_flag:job.PermissionFlag,
  license:{item:license.id,name:text(license.ItemNameID),max_count:license.MaxNum,guild:guild.id,
    completion_flag:411,in_progress_flag:406},
  default_abilities:defaults,selected_abilities:selected,jp_costs:job.JPCost,
  abilities,supports,parameter_revision:job.ParameterRevision,proper_equipment:job.ProperEquipment,
  tutorials:[58,54,57,56].map(id=>{const row=tutorialRows.find(r=>r.ID===id);assert(row);
    return {id,label:row.id,list_flag:row.TutorialListFlag,opened_flag:row.TutorialOpenedFlag};}),
};
const totalCost=selected.reduce((sum,_,i)=>sum+job.JPCost[defaults.length+i],0);
assert.equal(totalCost,130);
assert.equal(defaults.length+selected.length,supports[0].learned_ability_count);
const ailments=[...new Set([...includedAbilities.values()].flatMap(a=>a.Ailment.map(x=>x.AilmentName)).filter(x=>x!=='None'))].map(label=>one(ailmentRows,label));
const blueprints=[];
for(const [asset,functions] of [
  ['UserInterface/Guild/BP/GuildMenuWidget',['TaskClear','FirstTaskClear','EventRequestSetGetLicense','GetLicense','AfterLicenseAcquire']],
  ['Library/LibCharacter',['ChangePlayerSecondJob','AcquisitionAbility','AcquisitionSupportAbility','GetJobAcquisitionCommandAbilityNum','EquipmentSupportAbility','SettingPartySupportAbility']],
  ['System/BP/SaveLoad/KSSaveGameBP',['SetCharacterJob','SetJobPoint','SetSupportAbility']],
  ['UserInterface/MainMenu/Job/BP/JobStatusWidget',['ChangeCurrentCharacterSubJob','JobAbilityConfirmCallback','UpdateJobAbilitySupport','CancelEvent']],
  ['UserInterface/MainMenu/Job/BP/MainMenuJobWidget',['MenuOpen']],
  ['UserInterface/MainMenu/SupportAbility/BP/MainMenuAbilityWidget',['OpenSupportAbilityChange','MenuDecide']],
  ['UserInterface/MainMenu/Other/BP/LibAbilityAcquisition',['GetAcquisitionJPCost']],
  ['UserInterface/MainMenu/SupportAbility/BP/SupportAbilityChangeWidget',['ChangeSupportAbilityEvent']],
  ['UserInterface/MainMenu/Main/BP/MainMenuWidget',['Close','ExecuteUbergraph_MainMenuWidget']],
  ['Battle/BP/EncountManagerBP',['SetAfterMainMenu']],
]) {
  const b=blueprint(asset,functions);blueprints.push(b);
  sources.push({source:'/Game/'+asset,sha256:b.sha256});
}
const nativeFile='Octopath_Traveler2-Win64-Shipping.exe.c';
const nativeText=fs.readFileSync(path.join(s.root,nativeFile),'utf8'),native={};
for(const name of ['FUN_140f34b30','FUN_140f34cf0','FUN_140f271e0','FUN_140f11790',
  'FUN_14103a620','FUN_140e66f70','FUN_140e3f3e0','FUN_140f03a00','FUN_141057c40']) {
  const match=new RegExp('^[A-Za-z_][^\\r\\n]* '+name+'\\(','m').exec(nativeText);assert(match,name);
  const end=nativeText.indexOf('\n}',match.index);assert(end>match.index);
  const body=nativeText.slice(match.index,end+2);
  native[name]={source:nativeFile,function:name,sha256:s.assets.hash(Buffer.from(body)),body};
}
const audit={
  scope:'Scholar secondary-job assignment and acquisition of Lightning Bolt plus Elemental Barrage, followed by Evasive Maneuvers. All command-ability sets, boosted rows and super/hyper variants are retained as source evidence; this does not make them executable in the battle core.',
  sources,job,ability_sets:[...includedSets.values()],abilities:[...includedAbilities.values()],ailments,
  support_abilities:job.JobSupportAbility.map(a=>one(supportRows,a.AbilityName)),guild,license,scripts,texts,blueprints,native,
  tutorials:runtime.tutorials.map(t=>tutorialRows.find(r=>r.ID===t.id)),
  selected_plan:{job_id:job.JobID,default_abilities:defaults,
    learn:selected.map((id,i)=>({id,name:abilities.find(a=>a.id===id).name,cost:job.JPCost[defaults.length+i],known_before:defaults.length+i})),
    total_jp:totalCost,learned_count_after:defaults.length+selected.length,unlocked_support:supports[0]},
  interpretation:{
    defaults:'CreateAcquitisionAbilityData exec141095b30 -> f03a00 initializes the saved acquisition arrays for every JobData row: each command is initially learned exactly when GetParam==1. It counts those defaults, then initializes each support flag by comparing GetParam to that command count. The two Scholar default abilities therefore preexist in the saved per-job ledger; assigning the secondary job only changes SecondJobID.',
    jp_cost:'JPCost is indexed by the number of this job\'s already learned command abilities, including the two defaults. First two additional skills cost30 and100JP, independently of which non-divine abilities are chosen.',
    support:'JobSupportAbility.GetParam counts total learned job command abilities. Four known Scholar abilities unlock Evasive Maneuvers. Unlocking and equipping it are separate operations.',
    revision:'ParameterRevision stores native integer percentages. Apply the actual subjob stat formula/rounding from native evidence; do not add these percentages to equipment or repeatedly rescale already-adjusted stats.',
    equipment:'ProperEquipment is the original fourteen-slot permission mask. The first six weapon slots are sword/lance/dagger/axe/bow/rod; Scholar adds rod permission. Permission is not evidence of receiving or auto-equipping a new staff.',
    license:'GuildMenuWidget.CheckFirstFlowEvent -> FirstTaskClear -> EventRequestSetGetLicense plays SYS_SCH_GUILD_0030 (clears406, sets411). Its completion callback GetLicense -> TaskClear(index0) calls SetAcquiredJobLicense(GuildData.ID=6,index0,true), then ItemFunction.AddBackpackItem(ITM_EQP_JOB_0006,1). The event flags alone do not grant a license. Native f34cf0 -> f271e0/f11790 writes the guild record/license bit and sets global event flag100 using the same f11950(save+250,index,bool) writer as Blueprint SetEventFlag exec141057c40. Additional license copies/allocation require their own carried inventory state.',
    tutorials:'MainMenuJobWidget.MenuOpen requests58; JobStatusWidget.JobAbilityConfirmCallback requests54 and UpdateJobAbilitySupport requests57; MainMenuAbilityWidget.OpenSupportAbilityChange requests56. Their viewed/list flags are independent of command-ability and support-ability acquisition. Closing them does not reset the encounter counter.',
    menu_callbacks:'SetCharacterJob writes SecondJobID; AcquisitionAbility and AcquisitionSupportAbility update learned flags, JP and support unlock state; EquipmentSupportAbility and SettingPartySupportAbility rebuild equipped party effects. RefreshCharacterResource 14103a620 -> e66f70 -> e3f3e0 changes resource residency/load lists only. MainMenuWidget.Close refreshes party formation; OnCloseFinish reaches EncountManager.SetAfterMainMenu, which records a boolean and current position. These audited operations consume no CRT draw and do not call ResetEncountStep.',
  },
  menu_rng_evidence_status:'Source-audited menu callbacks and native resource refresh have no CRT draw or encounter-counter reset. This audit does not execute any learned Scholar battle ability or substitute a fixed draw count for its future battle behavior.',
};
fs.writeFileSync(path.join(__dirname,'../data/scholar-assignment.json'),JSON.stringify(runtime,null,2)+'\n');
fs.writeFileSync(path.join(__dirname,'../data/source/scholar-assignment-audit.json'),JSON.stringify(audit,null,2)+'\n');
console.log(JSON.stringify({job:runtime.job_id,name:runtime.name,defaults,selected,total_jp:totalCost,
  evasive_after_abilities:supports[0].learned_ability_count,sets:includedSets.size,ability_rows:includedAbilities.size,
  supports:supports.length,sources:sources.length}));
