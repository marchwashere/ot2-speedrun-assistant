'use strict';
// Snapshot the story boundary; generic map data remains in encounters.json.
const fs=require('node:fs'), path=require('node:path'), s=require('./source.cjs');
const {objectHeader}=require('../../OT2BattleEngine/tools/properties.cjs');
const sources=[];
function table(name){const t=s.table(name); sources.push({source:t.source,sha256:t.sha256});return t.rows;}
function script(id){const file=path.join(s.assets.CONTENT,'Event/json/'+id), b=fs.readFileSync(file);sources.push({source:'/Game/Event/json/'+id,sha256:s.assets.hash(b)});return Object.values(JSON.parse(b));}
const events=table('Event/Database/EventList'), after=events.find(r=>r.id==='MS_TOU_10_0300'), later=events.find(r=>r.id==='MS_TOU_10_0400');
const level=table('Level/Database/LevelTable').find(r=>String(r.ID)===after.MapID);
const task=table('Story/Database/MainStoryTask').find(r=>r.id===after.MissionLabel);
const opening=script('MS_TOU_10_0100'), guests=script('MS_TOU_10_0200'), exit=script(after.id), disable=script(later.id);
const file=path.join(s.assets.CONTENT,'Level/Level/City/Dng_Cty_3_1_Art.umap'), p=s.assets.loadPackage(file);
p.b=Buffer.concat([fs.readFileSync(file),fs.readFileSync(file.replace(/\.umap$/,'.uexp'))]);
sources.push({source:'/Game/Level/Level/City/Dng_Cty_3_1_Art',sha256:s.assets.hash(p.b)});
const actor=p.exports.find(e=>e.name===after.EncountTrigger);
if(!actor)throw Error('Post-event encounter actor missing');
const placement=s.fields(objectHeader(p,actor).properties);
const changeTime=opening.find(c=>c.cmd===1550);
// FUN_140e97200 parses opt[0], adds one, and calls SetTimeZoneType.
if(Number(changeTime?.opt[0])+1!==4)throw Error('Opening is no longer night');
if(level.id!=='Dng_Cty_3_1'||!after.EncountReset||placement.EncountTriggerLabel!=='ENT_DNG_CTY_3_1_LND'||!placement.EndFlag.includes(25304)||!disable.some(c=>c.cmd===70&&c.target===25304&&c.opt[0]==='1')||!guests.some(c=>c.cmd===3000&&c.target===15)||!guests.some(c=>c.cmd===3000&&c.target===16)||exit.some(c=>c.cmd===3020))throw Error('Story boundary changed');
const profile={
 id:'throne_after_opening',label:'After Throné’s opening battle',
 boundary_label:'After the post-battle event counter reset',event:after.id,valid_until_event:later.id,
 encounter_actor:after.EncountTrigger,story_task_progress:task.ProgressValue,
 known_flags:{'25304':false},party:['Throné','Pirro','Scaracci'],
 options:{map:level.id,zone:placement.EncountTriggerLabel,period:'Night',progress:0,first_encounter:true,boundary:'current_counter',party_levels:[1,1,1]},
 notes:[
  'New game as Throné, after MS_TOU_10_0300 and before MS_TOU_10_0400. No chapter has cleared: main progress is 0; task progress 30 is a separate value.',
  'The event battle leaves FirstEncount active. The next field encounter uses the fixed first group without a group lottery draw.',
  'MS_TOU_10_0400 sets flag 25304, selects ENT_EMPTY_1 and removes Pirro/Scaracci. This frame must not be reused after that event.',
  'The battle result already includes both field counter resets. Continue from its final CRT state and existing counter without resetting again.'
 ]
};
fs.writeFileSync(path.join(__dirname,'../data/throne-field.json'),JSON.stringify(profile,null,2)+'\n');
fs.writeFileSync(path.join(__dirname,'../data/source/throne-field.json'),JSON.stringify({sources,profile,native:{change_time:'FUN_140e97200: cmd 1550 option 3 + 1 = ETimeZoneType::NIGHT (4)',day_check:'FUN_140f64a40/FUN_140f64a60: only timezone 2 or 3 is day',progress:'FUN_140f05560 reads save+0x648; FUN_140f09f70 resets it to zero; FUN_140f04de0 commits chapter-clear points. MainStoryTask.ProgressValue is independent.'},blueprints:['EventManagerBP: SetUpEncountTriggerByName(PlayEventData.EncountTrigger)','EncountManagerBP.SetEncounter: select progress border using GetMainProgressPoint','EncountManagerBP: IsDayTime chooses DayTime/Night volume and decrement']},null,2)+'\n');
console.log(JSON.stringify(profile));
