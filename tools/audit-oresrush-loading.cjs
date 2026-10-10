'use strict';
// Source audit for the new route's placed Blueprint actors and story choices.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const s=require('./source.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const data=require('../data/town-loading.json');
const maps=new Map(),classes=new Map(),instances=[];
function walk(dir) {for(const e of fs.readdirSync(dir,{withFileTypes:true})) {
  const file=path.join(dir,e.name);
  if(e.isDirectory())walk(file);else if(e.name.endsWith('.umap'))maps.set(e.name.toLowerCase(),file);
}}
walk(path.join(s.assets.CONTENT,'Level/Level'));
function auditClass(classPath) {
  if(!classPath?.startsWith('/Game/')||classes.has(classPath))return;
  const asset=classPath.slice('/Game/'.length).split('.')[0];
  const pkg=s.assets.loadPackage(path.join(s.assets.CONTENT,asset+'.uasset'));
  const generated=pkg.exports.find(e=>e.name===classPath.split('.').at(-1));assert(generated,classPath);
  const parent=s.assets.objectPath(pkg,generated.superIndex),decoded=blueprint(asset);
  classes.set(classPath,{parent,sha256:decoded.sha256,functions:decoded.functions});
  auditClass(parent);
}
for(const map of ['Fld_Wld_1_3','Twn_Wld_1_1_C']) {
  for(const level of data.maps[map].streams) {
    const file=maps.get((level+'.umap').toLowerCase());assert(file,level);
    const pkg=s.assets.loadPackage(file,true);
    for(const e of pkg.exports) {
      const classPath=s.assets.objectPath(pkg,e.classIndex);
      if(!classPath?.startsWith('/Game/'))continue;
      instances.push({map,level,actor:e.name,class:classPath});auditClass(classPath);
    }
  }
}
const randomFunctions=[...classes].flatMap(([classPath,c])=>Object.entries(c.functions)
  .filter(([,body])=>/Random/.test(body)).map(([name,body])=>({class:classPath,function:name,body})));
const placement=s.table('Placement/Database/PlacementData').rows.find(r=>r.id==='PC_Party_Join_SHO_0000');
assert.equal(placement.EventLabel_A,'MS_SHO_00_0100');assert.equal(placement.EventEndFlag_A,37505);
assert.equal(placement.EventLabel_B,'MS_SHO_00_0200');assert.equal(placement.EventStartFlag_B,37505);
const mother=s.table('Story/Database/MainStory').rows.find(r=>r.id==='MS_TOU_01');
assert.equal(mother.OpeningEvent,'MS_TOU_2A_0100');assert.equal(mother.ID,701);
const first=data.events.MS_SHO_00_0100.commands;
assert.deepEqual([first[58].cmd,first[58].target,first[72].cmd,first[72].target],[3010,4,3120,4]);
fs.writeFileSync(path.join(__dirname,'../data/source/oresrush-loading-audit.json'),JSON.stringify({
  scope:'Southern Oresrush Wilds and current-era Oresrush: directly placed Blueprint classes and their Blueprint inheritance; event source data. Component/native callbacks require the shared loader audit.',
  instances,classes:Object.fromEntries(classes),randomFunctions,
  recruitment_placement:placement,mother_story:mother,
  event_choices:{MS_SHO_00_0100:{invite_yes:58,skip_chapter_no:72},MS_SHO_00_0200:{invite_yes:13,skip_chapter_no:30}},
},null,2)+'\n');
console.log(JSON.stringify({instances:instances.length,classes:classes.size,randomFunctions:randomFunctions.map(r=>({class:r.class,function:r.function}))}));
