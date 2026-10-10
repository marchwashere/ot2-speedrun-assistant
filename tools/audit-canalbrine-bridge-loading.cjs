'use strict';
// Audit every placed Blueprint/component, Blueprint parent and embedded level
// script used by Canalbrine Bridge. Shared loader rules remain data-driven.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const s=require('./source.cjs');
const {objectHeader}=require('../../OT2BattleEngine/tools/properties.cjs');
const decoder=require('../../OT2BattleEngine/tools/kismet_decoder.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const imported=require('../data/town-loading.json');
const sources=new Map(),mapFiles=new Map(),classes=new Map(),instances=[],embedded=[];
const map='Fld_Sea_1_2';
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
const levels=table('Level/Database/LevelTable');
const row=levels.find(r=>r.id===map);assert.equal(row.ID,107);assert.equal(row.BarEventLabel,'None');
const streams=[row.id,...row.BattleMapNameList.filter(n=>n!=='None')].flatMap(n=>
  ['','_Art','_Coll','_GD','_Sound'].flatMap(suffix=>{
    const file=mapFiles.get((n+suffix+'.umap').toLowerCase());return file?[path.basename(file,'.umap')]:[];
  }));
assert.deepEqual(streams,['Fld_Sea_1_2','Fld_Sea_1_2_Art','Fld_Sea_1_2_Coll','Fld_Sea_1_2_GD','Fld_Sea_1_2_Sound','Btl_Fld_Sea_Land_A','Btl_Fld_Sea_River_A']);
assert.deepEqual(imported.maps[map].streams,streams);
assert.deepEqual(imported.maps[map].environment_draws,[]);
assert.deepEqual([row.EncountStepMin,row.RandomStepA,row.RandomStepB,row.FirstEncountStepRatio],[1000,400,400,0.5]);
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
        placement:actorIndices.has(e.index)?'level_actor':'component_or_subobject'});auditClass(classPath);
    }
    if(classPath==='/Script/CoreUObject.Function') {
      const fn=s.assets.decodeFunction(pkg,e);
      embedded.push({map,level,function:e.name,runtime_bytes:fn.runtimeBytes,
        body:fn.expressions.map(expr=>decoder.flatten(expr).join('\n')).join('\n')});
    }
  }
}
const randomPattern=/Random|FRand|RandRange/;
const lineClass='/Game/Environment/BP/BP_MJLinePlace.BP_MJLinePlace_C';
const lineFunctions=classes.get(lineClass)?.functions;assert(lineFunctions,'Bridge procedural line Blueprint');
assert(lineFunctions.UserConstructionScript.includes('LocalVirtualFunction CreateMeshTable'));
assert(lineFunctions.UserConstructionScript.includes('LocalVirtualFunction CreateLinePlace'));
const lineHelpers=['CreateMeshTable','PrepareMeshLoopSize','CreateLinePlace','GeneratePartsMesh','CreateLinePlaceY','GetPartsMesh'];
for(const name of ['ExecuteUbergraph_BP_MJLinePlace','ReceiveDestroyed','ReceiveEndPlay']) {
  const body=lineFunctions[name];assert(body,name);
  assert(!lineHelpers.some(helper=>body.includes('Function '+helper)),'Runtime line callback invokes mesh generation: '+name);
}
const lineRandomCalls=['CreateMeshTable','GetPartsMesh'].flatMap(name=>
  [...lineFunctions[name].matchAll(/(?:CallMath|FinalFunction) [^:\n]+:[^:\n]+:([^\n]+)/g)]
    .map(m=>m[1]).filter(name=>randomPattern.test(name)));
assert.deepEqual(lineRandomCalls,['SetRandomStreamSeed','ResetRandomStream','RandomIntegerFromStream']);
const randomFunctions=[...classes].flatMap(([classPath,c])=>Object.entries(c.functions)
  .filter(([,body])=>randomPattern.test(body)).map(([name,body])=>({class:classPath,function:name,
    phase:name==='UserConstructionScript'?'construction':classPath===lineClass&&lineHelpers.includes(name)?'construction_helper':'runtime',
    generator:/FromStream|SetRandomStreamSeed|ResetRandomStream/.test(body)?'FRandomStream':'global random',body})));
const runtimeRandom=randomFunctions.filter(f=>f.phase==='runtime');
const embeddedRandom=embedded.filter(f=>randomPattern.test(f.body));
const levelTriggers=table('Level/Database/LevelTriggerTable');
const transitionLabels=['Twn_Dst_1_1_A_1','Fld_Dst_1_1_2','Fld_Dst_1_1_4'];
const transitions=transitionLabels.map(label=>{const t=levelTriggers.find(r=>r.id===label);assert(t);return t;});
assert.deepEqual(transitions.map(t=>[t.ID,t.NextMapName,t.NextLevelTriggerID,t.MapInCharaMoveMode]),[
  [1130,'Fld_Dst_1_1','ELevelTriggerID::Trigger_03','ECharacterMoveModeForDB::WALK'],
  [1001,map,'ELevelTriggerID::Trigger_04','ECharacterMoveModeForDB::WALK'],
  [1003,map,'ELevelTriggerID::Trigger_05','ECharacterMoveModeForDB::CANOE'],
]);
assert(transitions.every(t=>t.FlagList.every(f=>f===0)&&t.ConditionalTransitionData.every(c=>c.EnableFlag===0&&c.NextMapName==='None')));
const placementList=table('Placement/Database/PlacementList').find(r=>r.id===map);assert(placementList);
assert.equal(placementList.LabelList.length,36);
assert.deepEqual(imported.maps[map].placements,placementList.LabelList);
const allPlacements=table('Placement/Database/PlacementData');
const placements=placementList.LabelList.map(label=>{const p=allPlacements.find(r=>r.id===label);assert(p);return p;});
const npcRows=table('Character/Database/NPCData');
const npcs=[...new Set(placements.filter(p=>p.PlacementType===1).map(p=>p.ResourceLabel))].map(label=>{
  const npc=npcRows.find(r=>r.id===label);assert(npc,label);return npc;
});
const out={
  scope:'Ryu -> Northern Ryu Sands -> Canalbrine Bridge by the walking transition, after Scholar setup. User guarantees no encounter before arrival. No movement distance or extra counter decrement is invented. All Bridge field/battle streams and their placed Blueprint classes, parents and embedded scripts are audited.',
  sources:[...sources.values()],level:row,streams,environment_draws:imported.maps[map].environment_draws,
  transitions:{ryu_return:transitions[0],bridge_land_entry:transitions[1],alternative_canoe_entry:transitions[2]},
  placement_list:placementList,placements,npcs,instances,classes:Object.fromEntries(classes),
  embedded_scripts:embedded,random_functions:randomFunctions,runtime_random_functions:runtimeRandom,
  embedded_random_functions:embeddedRandom,
  procedural_line_helpers:{class:lineClass,construction_calls:['CreateMeshTable','CreateLinePlace'],
    helper_functions:lineHelpers,random_calls:lineRandomCalls,
    effect:'Procedural mesh variation belongs to the construction helper chain and uses its own FRandomStream. Its runtime event graph/Destroyed/EndPlay callbacks do not invoke the mesh-generation chain. These stream operations do not advance CRT rand.'},
  conclusion:runtimeRandom.length||embeddedRandom.length?'Additional runtime random callbacks require review before predicting this map.':'No additional environmental CRT callback found. Shared Loader still performs the real level-construction, eligible NPC-frame and outgoing/incoming counter callbacks; Bridge has no tavern event. Cooked construction-script randomness is not replayed as a runtime callback.',
};
const output=path.join(__dirname,'../data/source/canalbrine-bridge-loading-audit.json');
fs.writeFileSync(output,JSON.stringify(out,null,2)+'\n');
console.log(JSON.stringify({map,streams:streams.length,placements:placements.length,
  placed_blueprint_actors:instances.filter(i=>i.placement==='level_actor').length,
  direct_classes:new Set(instances.map(i=>i.class)).size,audited_classes:classes.size,
  embedded_functions:embedded.length,construction_random_functions:randomFunctions.filter(f=>f.phase==='construction').length,
  construction_random_helpers:randomFunctions.filter(f=>f.phase==='construction_helper').length,
  runtime_random_functions:runtimeRandom.map(f=>({class:f.class,function:f.function})),
  embedded_random_functions:embeddedRandom.map(f=>({level:f.level,function:f.function})),sources:sources.size,output}));
assert.equal(runtimeRandom.length,0,'New runtime Blueprint randomness requires a shared-loader implementation');
assert.equal(embeddedRandom.length,0,'New embedded level randomness requires a shared-loader implementation');
