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
const maps=['Fld_Sea_2_2','Fld_Sea_2_3','Twn_Sea_2_1_A'];
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
for(const map of maps) {
const row=levels.find(r=>r.id===map);
const streams=[row.id,...row.BattleMapNameList.filter(n=>n!=='None')].flatMap(n=>
  ['','_Art','_Coll','_GD','_Sound'].flatMap(suffix=>{
    const file=mapFiles.get((n+suffix+'.umap').toLowerCase());return file?[path.basename(file,'.umap')]:[];
  }));

assert.deepEqual(imported.maps[map].streams,streams);


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
}
const randomFunctions=[...classes].flatMap(([classPath,c])=>Object.entries(c.functions).filter(([,body])=>/Random|FRand|RandRange/.test(body)).map(([name,body])=>({class:classPath,function:name,body})));
const result={sources:[...sources.values()],instances,randomFunctions,embeddedRandom:embedded.filter(f=>/Random|FRand|RandRange/.test(f.body))};
fs.writeFileSync("data/source/conning-loading-audit.json",JSON.stringify(result,null,2)+"\n");
console.log(JSON.stringify({random:randomFunctions.map(f=>[f.class,f.function]),embedded:result.embeddedRandom,environment:Object.fromEntries(maps.map(id=>[id,imported.maps[id].environment_draws]))},null,2));
