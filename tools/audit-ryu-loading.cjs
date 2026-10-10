'use strict';
// Reproduce the Northern Ryu Sands -> Ryu loading and movement source audit.
// This writes evidence only; production loading data is imported separately.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const s=require('./source.cjs');
const {objectHeader}=require('../../OT2BattleEngine/tools/properties.cjs');
const decoder=require('../../OT2BattleEngine/tools/kismet_decoder.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const imported=require('../data/town-loading.json');
const sources=new Map(),mapFiles=new Map(),classes=new Map(),instances=[],embeddedScripts=[];
const targets=[['Fld_Dst_1_1',19],['Twn_Dst_1_1_A',28]];
const expectedStreams={
  Fld_Dst_1_1:['Fld_Dst_1_1','Fld_Dst_1_1_Art','Fld_Dst_1_1_Coll','Fld_Dst_1_1_Sound','Btl_Fld_Dst_Land_B','Btl_Fld_Dst_River_B'],
  Twn_Dst_1_1_A:['Twn_Dst_1_1_A','Twn_Dst_1_1_A_Art','Twn_Dst_1_1_A_Coll','Twn_Dst_1_1_A_sound','Btl_Twn_Dst_1_1_A'],
};
function recordFile(file) {
  const source='/Game/'+path.relative(s.assets.CONTENT,file).replaceAll('\\','/');
  sources.set(source,{source,sha256:s.assets.hash(fs.readFileSync(file))});
}
function table(name) {
  const t=s.table(name);sources.set(t.source,{source:t.source,sha256:t.sha256});return t.rows;
}
function walk(dir) {for(const e of fs.readdirSync(dir,{withFileTypes:true})) {
  const file=path.join(dir,e.name);
  if(e.isDirectory())walk(file);else if(e.name.endsWith('.umap')) {
    const key=e.name.toLowerCase();assert(!mapFiles.has(key),'Ambiguous map package '+key);mapFiles.set(key,file);
  }
}}
walk(path.join(s.assets.CONTENT,'Level/Level'));
function auditClass(classPath) {
  if(!classPath?.startsWith('/Game/')||classes.has(classPath))return;
  const asset=classPath.slice('/Game/'.length).split('.')[0];
  const file=path.join(s.assets.CONTENT,asset+'.uasset'),pkg=s.assets.loadPackage(file);
  const generated=pkg.exports.find(e=>e.name===classPath.split('.').at(-1));assert(generated,classPath);
  const parent=s.assets.objectPath(pkg,generated.superIndex),decoded=blueprint(asset);
  classes.set(classPath,{source:'/Game/'+asset,parent,sha256:decoded.sha256,functions:decoded.functions});
  recordFile(file);recordFile(file.replace(/\.uasset$/,'.uexp'));auditClass(parent);
}
const levels=table('Level/Database/LevelTable'),auditedMaps={};
for(const [map,id]of targets) {
  const row=levels.find(r=>r.id===map);assert.equal(row.ID,id);
  const streams=[row.id,...row.BattleMapNameList.filter(n=>n!=='None')].flatMap(n=>
    ['','_Art','_Coll','_GD','_Sound'].flatMap(suffix=>{
      const file=mapFiles.get((n+suffix+'.umap').toLowerCase());return file?[path.basename(file,'.umap')]:[];
    }));
  assert.deepEqual(streams,expectedStreams[map]);
  assert.deepEqual(imported.maps[map].streams,streams);
  assert.deepEqual(imported.maps[map].environment_draws,[]);
  assert.deepEqual([row.EncountStepMin,row.RandomStepA,row.RandomStepB,row.FirstEncountStepRatio],[1000,400,400,0.5]);
  const before=instances.length;
  for(const level of streams) {
    const file=mapFiles.get((level+'.umap').toLowerCase()),pkg=s.assets.loadPackage(file,true);
    const bodyFile=file.replace(/\.umap$/,'.uexp');recordFile(file);recordFile(bodyFile);
    pkg.b=Buffer.concat([fs.readFileSync(file),fs.readFileSync(bodyFile)]);
    const owner=pkg.exports.find(e=>s.assets.objectPath(pkg,e.classIndex)==='/Script/Engine.Level');assert(owner,level);
    const cursor=objectHeader(pkg,owner).c,count=cursor.i32();assert(count>=0&&count<=pkg.exports.length);
    const actorIndices=new Set(Array.from({length:count},()=>cursor.i32()));
    for(const e of pkg.exports) {
      const classPath=s.assets.objectPath(pkg,e.classIndex);
      if(classPath?.startsWith('/Game/')) {
        instances.push({map,level,object:e.name,export_index:e.index,class:classPath,
          placement:actorIndices.has(e.index)?'level_actor':'component_or_subobject'});
        auditClass(classPath);
      }
      if(classPath==='/Script/CoreUObject.Function') {
        const fn=s.assets.decodeFunction(pkg,e);
        embeddedScripts.push({map,level,function:e.name,runtime_bytes:fn.runtimeBytes,
          body:fn.expressions.map(expr=>decoder.flatten(expr).join('\n')).join('\n')});
      }
    }
  }
  auditedMaps[map]={id,level_row:row,streams,environment_draws:[],
    placed_blueprint_actors:instances.slice(before).filter(i=>i.placement==='level_actor').length};
}
const directClasses=new Set(instances.map(i=>i.class));
assert.equal(directClasses.size,40);assert.equal(classes.size,49);
assert.equal(auditedMaps.Fld_Dst_1_1.placed_blueprint_actors,256);
assert.equal(auditedMaps.Twn_Dst_1_1_A.placed_blueprint_actors,139);
assert(![...directClasses].some(c=>/BP_EnvObjMD_(?:Canoe|Flag)_A\./.test(c)));
const randomPattern=/Random|FRand|RandRange/;
const randomFunctions=[...classes].flatMap(([classPath,c])=>Object.entries(c.functions)
  .filter(([,body])=>randomPattern.test(body)).map(([name,body])=>({class:classPath,function:name,
    phase:name==='UserConstructionScript'?'construction':'runtime',
    generator:body.includes('FromStream')?'FRandomStream':'global random',body})));
assert.equal(randomFunctions.length,3);
const runtimeRandomFunctions=randomFunctions.filter(f=>f.phase==='runtime');
assert.deepEqual(runtimeRandomFunctions,[]);
assert(embeddedScripts.every(f=>!randomPattern.test(f.body)));
assert.deepEqual(embeddedScripts.map(f=>[f.level,f.function]),[
  ['Btl_Fld_Dst_River_B','ExecuteUbergraph_Btl_Fld_Dst_River_B'],
  ['Btl_Fld_Dst_River_B','ReceiveBeginPlay'],
]);
const emptyRiverGraph=embeddedScripts[0].body;
assert(!/Function|CallMath/.test(emptyRiverGraph));
assert(emptyRiverGraph.includes('Return')&&emptyRiverGraph.includes('Nothing'));
const transition=table('Level/Database/LevelTriggerTable').find(r=>r.id==='Fld_Dst_1_1_3');
assert.equal(transition.ID,1002);assert.equal(transition.NextMapName,'Twn_Dst_1_1_A');
assert.equal(transition.NextLevelTriggerID,'ELevelTriggerID::Trigger_01');
assert.equal(transition.MapInCharaMoveMode,'ECharacterMoveModeForDB::WALK');
assert(transition.FlagList.every(f=>f===0));
assert(transition.ConditionalTransitionData.every(r=>r.EnableFlag===0&&r.NextMapName==='None'));
const landZone=table('Battle/Database/EncountTriggerData').find(r=>r.id==='ENT_FLD_DST_1_1_LND');
assert.deepEqual([landZone.DayTimeDecreaseValue,landZone.NightDecreaseValue],[9,18]);
assert(['StartFlag','StartFlag_2','StartFlag_3','EndFlag','EndFlag_2','EndFlag_3'].every(k=>landZone[k]===0));
const encounterData=require('../data/encounters.json');assert.equal(encounterData.dash_multiplier,1.4);
assert.equal(encounterData.zones[landZone.id].periods.DayTime.decrement,landZone.DayTimeDecreaseValue);
const cost={measured_running_updates:137,measured_walking_updates:143,leeway_updates:3,
  padded_running_updates:140,padded_walking_updates:146,daytime_base:9,evasive_percent:50,
  dash_multiplier:encounterData.dash_multiplier,running_decrement:6,walking_decrement:4,
  running_cost:840,walking_cost:584,minimum_safe_running_counter:841,minimum_safe_walking_counter:585,
  first_counter_min:500,first_counter_max:899,
  mixed_distance_share:'floor(clamp((counter - 1 - 584) / (840 - 584), 0, 1) * 1000) / 1000; counter <= 584 is unsafe',
  semantics:'Counter must remain positive. Progression changes the encounter tier, not this zone decrement. Movement is budget interpolation; it does not simulate speed-switch overshoot.'};
const out={
  scope:'Northern Ryu Sands and Ryu: every placed Blueprint actor/component class, its Blueprint parents, and embedded level scripts. Random Blueprint callbacks are distinguished from cooked construction scripts. Shared native level-constructor/NPC/counter callbacks remain the Loader responsibility. No recruitment event is executed by this audit.',
  sources:[...sources.values()],maps:auditedMaps,transition,land_zone:landZone,movement:cost,
  instances,classes:Object.fromEntries(classes),embedded_scripts:embeddedScripts,
  random_functions:randomFunctions,runtime_random_functions:runtimeRandomFunctions,
  conclusion:'No additional environmental CRT callbacks were found. Both imported environment_draws arrays are empty; signboards start deterministic timelines, light-beam runtime graph and river level BeginPlay graph have no random work.',
};
const output=path.join(__dirname,'../data/source/ryu-loading-audit.json');
fs.writeFileSync(output,JSON.stringify(out,null,2)+'\n');
console.log(JSON.stringify({maps:targets.map(([map])=>({map,streams:auditedMaps[map].streams.length,
  placed_blueprint_actors:auditedMaps[map].placed_blueprint_actors})),direct_classes:directClasses.size,
  audited_classes:classes.size,embedded_functions:embeddedScripts.length,
  construction_random_functions:randomFunctions.length,runtime_random_functions:runtimeRandomFunctions.length,
  sources:sources.size,output:path.relative(process.cwd(),output)}));
