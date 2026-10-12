'use strict';
// Import rules and candidates, never pre-count eligible NPCs or RNG calls.
const fs = require('node:fs'), path = require('node:path'), assert = require('node:assert/strict');
const s = require('./source.cjs');
const {objectHeader} = require('../../OT2BattleEngine/tools/properties.cjs');
const {blueprint} = require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const sources = [];
function flagRanges(values) {
  assert.equal(values.length%2,0);
  const ranges=[];
  for(let i=0;i<values.length;i+=2) {
    const begin=values[i],end=values[i+1];
    if(begin===0&&end===0)continue;
    assert(begin>0&&end>=begin,'Invalid main-story flag range');ranges.push([begin,end]);
  }
  return ranges;
}
function table(name) {
  const t = s.table(name); sources.push({source:t.source,sha256:t.sha256}); return t.rows;
}
const levels = table('Level/Database/LevelTable');
const lists = table('Placement/Database/PlacementList');
const placements = table('Placement/Database/PlacementData');
const npcs = table('Character/Database/NPCData');
const playable = table('Character/Database/PlayableCharacterDB');
const eventRows = table('Event/Database/EventList');
const storyRows = table('Story/Database/MainStory');
const taskRows = table('Story/Database/MainStoryTask');
const tutorials = table('Tutorial/Database/TutorialFlagPart');
const rules = table('Flag/Database/FlagRandomControlDB');
const overrides = table('Flag/Database/FlagOverrideDB');
const maps = levels.filter(r => [19,28,44,45,46,47,48,50,54,57,64,65,69,85,86,87,88,90,100,101,107,110,111,123,124,125,129,132,145,152,153,155,169,170,189].includes(r.ID) || r.id === 'Fld_Fst_1_3' || r.id === 'Twn_Wld_3_1_A' || ['Fld_Dst_2_3','Fld_Dst_2_4'].includes(r.id) || r.id.startsWith('Twn_Cty_1_'));
const allMaps = new Map();
function walk(dir) { for(const e of fs.readdirSync(dir,{withFileTypes:true})) {
  const f = path.join(dir,e.name);
  if(e.isDirectory())walk(f); else if(e.name.endsWith('.umap')) {
    const key=e.name.toLowerCase();
    assert(!allMaps.has(key), 'ambiguous map package: '+e.name); allMaps.set(key,f);
  }
}}
walk(path.join(s.assets.CONTENT,'Level/Level'));
const canoeAsset='Environment/Resource/Object/EnvObj_Canoe/BP/BP_EnvObjMD_Canoe_A';
const canoe=blueprint(canoeAsset,['ReceiveBeginPlay','ExecuteUbergraph_BP_EnvObjMD_Canoe_A']);
const canoeBody=canoe.functions.ExecuteUbergraph_BP_EnvObjMD_Canoe_A;
assert(/RandomFloatInRange\n[^\n]*FloatConst 0\n[^\n]*FloatConst 20/.test(canoeBody));
assert(canoeBody.includes('SetNewTime'));
assert.equal((canoeBody.match(/:RandomFloatInRange/g)||[]).length,1);
sources.push({source:'/Game/'+canoeAsset,sha256:canoe.sha256});
const flagAsset='Environment/Resource/Object/EnvObj_Flag/BP/BP_EnvObjMD_Flag_A';
const flag=blueprint(flagAsset,['ReceiveBeginPlay','ExecuteUbergraph_BP_EnvObjMD_Flag_A']);
const flagBody=flag.functions.ExecuteUbergraph_BP_EnvObjMD_Flag_A;
assert.equal((flagBody.match(/:RandomFloatInRange/g)||[]).length,1);
assert(flagBody.includes('SetPlayRate'));
assert(flag.functions.ReceiveBeginPlay.includes('IntConst 10'));
sources.push({source:'/Game/'+flagAsset,sha256:flag.sha256});
const environmentKinds=new Map([
  [`/Game/${canoeAsset}.BP_EnvObjMD_Canoe_A_C`,'canoe_bobbing_time'],
  [`/Game/${flagAsset}.BP_EnvObjMD_Flag_A_C`,'flag_animation_speed'],
]);
for(const [asset,kind,entry] of [
 ['Environment/Resource/Object/EnvObj_Batlefield/BP/BP_EnvObjMD_Batlefield_A_Flag_A','flag_animation_speed',10],
 ['Environment/Resource/Object/EnvObj_Batlefield/BP/BP_EnvObjMD_Batlefield_A_Flag_A_TwoTone_B','flag_animation_speed',10],
 ['Environment/Resource/Object/EnvObj_Lamp/BP/BP_Lamp_I_Sea','lamp_swing_time',235],
 ['Environment/Resource/Object/EnvObj_Gondola/BP/BP_EnvObjMD_Gondola_A_NoPaddle','gondola_bobbing_time',214],
 ['Environment/Resource/Object/EnvObj_Gondola/BP/BP_EnvObjMD_Gondola_A_Paddle','gondola_bobbing_time',214],
]) {
 const name=asset.split('/').at(-1),bp=blueprint(asset,['ReceiveBeginPlay','ExecuteUbergraph_'+name]);
 assert(bp.functions.ReceiveBeginPlay.includes('IntConst '+entry));
 assert.equal((bp.functions['ExecuteUbergraph_'+name].match(/:RandomFloatInRange/g)||[]).length,1);
 if(kind==='gondola_bobbing_time') {
   assert(/RandomFloatInRange\n[^\n]*FloatConst 0\n[^\n]*FloatConst 20/.test(bp.functions['ExecuteUbergraph_'+name]));
   assert(bp.functions['ExecuteUbergraph_'+name].includes('SetNewTime'));
 }
 environmentKinds.set('/Game/'+asset+'.'+name+'_C',kind);
 sources.push({source:'/Game/'+asset,sha256:bp.sha256});
}
function environmentDraws(streams) {
  return streams.flatMap(level=>{
    const file=allMaps.get((level+'.umap').toLowerCase()),p=s.assets.loadPackage(file,true);
    const candidates=p.exports.filter(e=>environmentKinds.has(s.assets.objectPath(p,e.classIndex)));
    if(!candidates.length)return [];
    // Map packages have a .umap header, so join the split export data explicitly.
    const body=fs.readFileSync(file.replace(/\.umap$/,'.uexp'));
    p.b=Buffer.concat([fs.readFileSync(file),body]);
    sources.push({source:'/Game/'+path.relative(s.assets.CONTENT,file).replaceAll('\\','/').replace(/\.umap$/,'.uexp'),sha256:s.assets.hash(body)});
    const owner=p.exports.find(e=>s.assets.objectPath(p,e.classIndex)==='/Script/Engine.Level');
    const c=objectHeader(p,owner).c,count=c.i32();assert(count>=0&&count<=p.exports.length);
    const actors=Array.from({length:count},()=>c.i32());
    const byIndex=new Map(candidates.map(e=>[e.index,e]));
    const draws=actors.filter(i=>byIndex.has(i)).map(i=>({level,actor:byIndex.get(i).name,kind:environmentKinds.get(s.assets.objectPath(p,byIndex.get(i).classIndex))}));
    assert.equal(draws.length,candidates.length,'Environment export missing from level actor array');
    return draws;
  });
}
function streamed(row) {
  const names = [row.id, ...row.BattleMapNameList.filter(n=>n!=='None')];
  return names.flatMap(n=>['','_Art','_Coll','_GD','_Sound'].flatMap(suffix=>{
    // UE package/FName lookup ignores case; cooked suffixes are not uniform.
    // Fld_Snw_1_2_sound is a real streamed level and its constructor draws CRT.
    const file=allMaps.get((n+suffix+'.umap').toLowerCase());
    if(!file){assert(suffix!=='' || n===row.id, 'missing battle map '+n); return [];}
    sources.push({source:'/Game/'+path.relative(s.assets.CONTENT,file).replaceAll('\\','/'),sha256:s.assets.hash(fs.readFileSync(file))});
    return [path.basename(file,'.umap')];
  }));
}
const labels = new Set(maps.flatMap(m=>lists.find(l=>l.id===m.id)?.LabelList||[]));
const placementBy = new Map(placements.map(p=>[p.id,p]));
const npcBy = new Map(npcs.map(p=>[p.id,p]));
const events = {};
for(const name of [...['0100','0200','0300','0400','0410','0500','0600','0700','0800','0900','0910','1000','1100','1200','1300','1400','1500','1600','1700','1900','2000'].map(s=>'MS_TOU_10_'+s),...['0100','0200','0300','0310','0400'].map(s=>'MS_KEN_00_'+s),'SC_FC13_Tutorial_0100','MS_KUS_00_0100','MS_GAK_00_0100','MS_ODO_00_0200','MS_SHO_00_0100','MS_SHO_00_0200','MS_TOU_2A_0100','MS_ODO_20_0100','MS_SHO_20_0050',...['0100','0110','0200','0300','0400','0410','0500','0600','0610','0615','0620'].map(n=>'MS_SHO_20_'+n),'SC_LinerShip_Cty_to_Wld','SC_LinerShip_Cty_to_Isd','SC_LinerShip_Isd_to_Sea','SYS_INV_GUILD_0010','SYS_INV_GUILD_0C10','SYS_INV_GUILD_0D10','SYS_SCH_GUILD_0010','SYS_SCH_GUILD_0030','SYS_MER_GUILD_0010','SYS_MER_GUILD_0030','MapInStorySelect']) {
  const row=eventRows.find(r=>r.id===name); assert(row);
  const raw=fs.readFileSync(path.join(s.assets.CONTENT,'Event/json',name));
  sources.push({source:'/Game/Event/json/'+name,sha256:s.assets.hash(raw)});
  const commands=Object.entries(JSON.parse(raw)).map(([index,c])=>({index:Number(index),cmd:c.cmd,target:c.target,text:c.text,opt:c.opt}));
  assert(commands.every((c,i)=>c.index===i));
  const task=taskRows.find(t=>t.id===row.MissionLabel);
  const story=task&&storyRows.find(t=>t.id===task.MainStoryLabel);
  const lastTask=story&&taskRows.filter(t=>t.MainStoryLabel===story.id&&t.TaskType==='ESTORY_TASK_TYPE::NORMAL').at(-1);
  const completion=task&&task===lastTask?{scenario:story.ID,character:story.PlayerCharaID,points:story.ClearDifficultyPoint}:null;
  // PlayEvent updates the final mission before its fade/map-load callback.
  // Limit this newly audited completion path to fresh Throné chapter 1.
  if(completion)assert.equal(name,'MS_TOU_10_2000');
  events[name]={map_id:row.MapID==='None'?0:Number(row.MapID),reset_counter:row.EncountReset,completion,commands};
}
const data={
  // These are route initial conditions, not RNG totals. Event commands supply flags/guests.
  fresh_throne:{map:'Dng_Cty_3_1',time:4,party:[7],bench:[],story_types:[7],active_scenarios:[700],prologues:['EPlayableCharacterID::eTHIEF'],shared_levels:['Btl_SinExpose']},
  maps:Object.fromEntries(maps.map(m=>{const streams=streamed(m);return [m.id,{
    id:m.ID,streams,field_streams:streams.filter(n=>['','_art','_coll','_gd','_sound'].some(suffix=>n.toLowerCase()===m.id.toLowerCase()+suffix)),
    environment_draws:environmentDraws(streams),placements:lists.find(l=>l.id===m.id)?.LabelList||[],
    counter:{min:m.EncountStepMin,range_a:m.RandomStepA,range_b:m.RandomStepB,first_ratio:m.FirstEncountStepRatio},
    bar_event:m.BarEventLabel,bar_allowed:m.BarPlacementChara
  }]})),
  placements:Object.fromEntries([...labels].map(label=>{
    const p=placementBy.get(label);assert(p,label);
    const npc=p.PlacementType===1?npcBy.get(p.ResourceLabel):null;
    // Preserve cooked event-only placeholder rows as null resources. They are
    // still gated normally; the runtime refuses to spawn an unresolved NPC.
    if(p.PlacementType===1)assert(npc||p.ResourceLabel==='None','missing NPC '+p.ResourceLabel);
    return [label,{
      kind:p.PlacementType,resource:p.ResourceLabel,action:npc?.ActionID||null,
      begin:p.SpawnStartFlag,end:p.SpawnEndFlag,time:p.TimeZoneSpawnType.split('::').at(-1),
      start_delay:p.SpawnStartDelayCount,end_delay:p.SpawnEndDelayCount,
      story_type:p.MainScenarioType,scenario:p.MainScenarioID,
      not_prologue:p.NorSpawnPrologue,prologue:p.CompProloguePlayerID,not_final:p.NotSpawnFinal,
      excludes:p.NotCoexistencePlacementLabel.filter(n=>n!=='None')
    }];
  })),
  // Preserve cooked row order: native ea0270 walks the table's sparse element order.
  flag_rules:rules.map(r=>({id:r.id,trigger:r.ControlTrigger.split('::').at(-1),begin:r.ControlBeginFlag,end:r.ControlEndFlag,apply:r.ApplyValue,candidates:r.ControlCandidate.map(c=>({flag:c.Flag,weight:c.Rate}))})),
  prologue_completions:Object.fromEntries([1,3,4,6,7,8].map(id=>{
    const character={1:"eFENCER",3:"eALCHEMIST",4:"eMERCHANT",6:"ePROFESSOR",7:"eTHIEF",8:"eDANCER"}[id];
    const stories=storyRows.filter(r=>r.PlayerCharaID.endsWith("::"+character)&&r.StoryCategory==='EMAIN_STORY_CATEGORY::CHARA_PROLOGUE');
    assert(stories.length);
    return [id,stories.map(r=>({scenario:r.ID,character:r.PlayerCharaID,points:r.ClearDifficultyPoint,
      on_flag_ranges:flagRanges(r.EventFlagRange),off_flag_ranges:flagRanges(r.OffEventFlagRange)}))];
  })),
  story_starts:Object.fromEntries(['MS_TOU_01','MS_ODO_01','MS_SHO_01'].map(label=>{
    const story=storyRows.find(r=>r.id===label);assert(story,label);
    const character=playable.find(r=>r.id===story.PlayerCharaID.split('::').at(-1));assert(character,label);
    assert(events[story.OpeningEvent],story.OpeningEvent);
    const event=eventRows.find(r=>r.id===story.OpeningEvent);
    const task=taskRows.find(r=>r.id===event.MissionLabel);assert(task&&task.MainStoryLabel===label,label);
    return [label,{scenario:story.ID,character_type:character.Id,opening_event:story.OpeningEvent,release_flags:story.ReleaseFlag.filter(f=>f>0),
      view_event_flag:task.ViewEventFlag}];
  })),
  events,
  tutorials:Object.fromEntries(tutorials.map(t=>[t.ID,{label:t.id,list_flag:t.TutorialListFlag,opened_flag:t.TutorialOpenedFlag}]))
};
const queriedFlags=new Set([...Object.values(data.placements).flatMap(p=>[p.begin,p.end]),...data.flag_rules.flatMap(r=>[r.begin,r.end]),...Object.values(events).flatMap(e=>e.commands.filter(c=>c.cmd===4000).map(c=>c.target))]);
const applicableOverrides=overrides.filter(r=>r.TargetEventFlags.some(f=>f>0&&queriedFlags.has(f)));
assert.equal(applicableOverrides.length,0,'New flag override dependencies require a runtime query binding');
fs.writeFileSync(path.join(__dirname,'../data/town-loading.json'),JSON.stringify(data,null,2)+'\n');
fs.writeFileSync(path.join(__dirname,'../data/source/canoe-initialization.json'),JSON.stringify(canoe,null,2)+'\n');
fs.writeFileSync(path.join(__dirname,'../data/source/town-loading.json'),JSON.stringify({
  scope:'Sewer, Diamante mansion, New Delsta zones and Harbor, New Delsta Flats, Abandoned Village, Eastern New Delsta Highroad, Cape Cold Snows/Cape Cold, Western Winterbloom Snows/Winterbloom, ocean liner/Crackridge Harbor, Cropdale trails/Cropdale, Southern Oresrush Wilds/Oresrush, Northern Ryu Sands/Ryu, Canalbrine Bridge, and supported recruitment/story events; unfiltered placements, native flag lotteries, event commands, and available streamed packages. No fixed draw totals.',
  flag_override_dependencies:applicableOverrides,
  sources,
  native:{placement:'ec4900 -> ec5180 -> ed5040',lottery:'ea0270 -> ea7520; eaec70/eaecb0',time:'f6aa40 -> f6eb70',level:'ed98a0 -> ec60c0 -> 11c0100'},
  prologue_skip:{native:'f15190 -> f54e00 -> f4ec50 -> f537f0',effect:'SkipPrologueReminiscence completes every CHARAPROLOGUE row for the selected character. Besides one-time ClearDifficultyPoint, f537f0(scenario,1) sets every inclusive EventFlagRange and clears every inclusive OffEventFlagRange. Zero/zero range slots are unused. Apply only on the actual story-skip completion, not generic point-accounting calls.'},
  blueprint:{branch:'EventManagerBP.BitFlagJump selects opt[0] for true / opt[1] for false; EventIndex increments before the next command.',spawn:'BPC_LevelManager_Others.SpawnNPC RandomIntegerInRange(0,15)',tavern:'RefreshBarCharacter returns before any draw when BarEventLabel=None or final chapter is active',counter:'ResetEncountVolume / ResetEncountOnChangeLevel / FinishPre -> CalcEncountStepCount, B then A'},
  no_draw_commands:{'1/2':'input and visibility','42/62/910':'scripted ambush setup/audio','52/7100':'tutorial presentation','230':'SetCharacterPosition, existing player target 0; native ff7ba0 / FName f9ee68','2310':'StartFullRecovery; native ffb8b0 / FName f9f070','310':'restore field actors','1250/1251':'indoor visibility','1500/1501/9980':'skipped rich sequence, audited 0700/0800/0900/1000/1200 have no other CRT callbacks','5010':'explicit dialog selection; 0910 Yes proceeds to 1000 with EncountReset=false, verified by live parlor capture','8030':'StartRefreshEventObject -> RefreshFlagObject -> ed4ea0; refresh existing flag-controlled objects, not chapter setup or an NPC lottery','8900':'second-battle resource setup'},
  parlor_revisit_popup:{event:'MS_TOU_10_1300',tutorial:31,label:'eFIELD_COMMAND_DAYTIME_TOU',command_300:'ff95a0 -> e9d810: enables existing party actors, no spawn',command_40:'EventManagerBP.StartTalk -> SetTalkData: fixed TX_MS_TOU_10_1300_0320 text and voice lookup, no random selection',tutorial_open:'e9f1d0 -> 10f7e30 -> f67070; f37740/f39b50 set opened/list flags',tutorial_close:'f66900 -> f6c860, no counter callback',finish:'FinishEvent -> FinishPre; EncountReset=false; flags 8012/514 do not enable new placements in the imported map'},
},null,2)+'\n');
console.log(JSON.stringify({maps:maps.length,placements:labels.size,flag_rules:rules.length,events:Object.keys(events).length}));
