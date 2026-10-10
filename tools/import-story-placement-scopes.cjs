'use strict';
// f4c660 -> f49fa0(current map, false) -> f2b480: placement story
// filters use active stories related to this map, not every active story.
const fs=require('node:fs'),path=require('node:path'),s=require('./source.cjs');
const stories=s.table('Story/Database/MainStory'),players=s.table('Character/Database/PlayableCharacterDB');
const rows=Object.fromEntries(stories.rows.map(r=>[r.ID,{
  character:players.rows.find(p=>p.id===r.PlayerCharaID.split('::').at(-1))?.Id||0,
  maps:r.RelationLevel.filter(m=>m!=='None'),
  switches:r.SwitchWMapLocation.map(v=>({begin:v.StartFlag,end:v.EndFlag,maps:v.RelationLevel.filter(m=>m!=='None')})),
}]));
fs.writeFileSync(path.join(__dirname,'../data/story-placement-scopes.json'),JSON.stringify(rows,null,2)+'\n');
