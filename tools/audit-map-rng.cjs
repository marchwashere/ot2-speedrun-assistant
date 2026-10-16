'use strict';
// Independent inventory: discover actors before checking the modeled allowlist.
const fs=require('fs'),path=require('path');
const s=require('./source.cjs');
const {objectHeader}=require('../../OT2BattleEngine/tools/properties.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const data=require('../data/town-loading.json'),files=new Map(),classes=new Map(),levels={};
function walk(dir){for(const e of fs.readdirSync(dir,{withFileTypes:true})){const f=path.join(dir,e.name);if(e.isDirectory())walk(f);else if(e.name.endsWith('.umap'))files.set(e.name.slice(0,-5).toLowerCase(),f);}}
walk(path.join(s.assets.CONTENT,'Level/Level'));
for(const level of new Set(Object.values(data.maps).flatMap(m=>m.streams))){
 const file=files.get(level.toLowerCase());if(!file)throw Error(level);
 const p=s.assets.loadPackage(file,true);p.b=Buffer.concat([fs.readFileSync(file),fs.readFileSync(file.replace(/\.umap$/,'.uexp'))]);
 const owner=p.exports.find(e=>s.assets.objectPath(p,e.classIndex)==='/Script/Engine.Level');
 const c=objectHeader(p,owner).c,n=c.i32(),actors=[];
 for(let i=0;i<n;i++){const index=c.i32(),e=p.exports.find(e=>e.index===index);if(!e)continue;const cls=s.assets.objectPath(p,e.classIndex);actors.push({actor:e.name,cls});if(!classes.has(cls))classes.set(cls,[]);classes.get(cls).push({level,actor:e.name});}
 levels[level]=actors;
}
const results=[];
for(const [cls,instances]of classes){
 if(!cls.startsWith('/Game/'))continue;
 const asset=cls.slice(6).split('.')[0];
 try{const p=s.assets.loadPackage(path.join(s.assets.CONTENT,asset+'.uasset'));
 const generated=p.exports.find(e=>e.name===cls.split('.').at(-1));
 const parent=generated?s.assets.objectPath(p,generated.superIndex):null;
 if(parent?.startsWith('/Game/')){if(!classes.has(parent))classes.set(parent,[]);classes.get(parent).push(...instances.map(i=>({...i,inherited:cls})));}
 const b=blueprint(asset);const random=Object.entries(b.functions).filter(([k,v])=>/random|randrange|shuffle/i.test(v));
 results.push({cls,parent,instances,sha256:b.sha256,functions:random.length?b.functions:undefined,random_functions:random.map(([k])=>k)});
 }catch(e){results.push({cls,instances,error:String(e)});}
}
fs.mkdirSync('target',{recursive:true});
fs.writeFileSync('target/map-rng-audit.json',JSON.stringify({maps:Object.keys(data.maps),levels,classes:results},null,2));
const beginPlay=new Map([
 ['BP_EnvObjMD_Batlefield_A_Flag_B','flag_animation_speed'],['BP_EnvObjMD_Batlefield_A_Flag_A','flag_animation_speed'],
 ['BP_EnvObjMD_Batlefield_A_Flag_A_TwoTone_B','flag_animation_speed'],['BP_EnvObjMD_Flag_A','flag_animation_speed'],
 ['BP_EnvObjMD_Canoe_A','canoe_bobbing_time'],['BP_Lamp_I_Sea','lamp_swing_time'],
 ['BP_EnvObjMD_Gondola_A_NoPaddle','gondola_bobbing_time'],['BP_EnvObjMD_Gondola_A_Paddle','gondola_bobbing_time'],['BP_OnWaveObject_A','gondola_bobbing_time'],
]);
const assert=require('node:assert/strict');
assert.equal(results.filter(r=>r.error).length,0,'Undecoded actor classes');
for(const r of results.filter(r=>r.random_functions.length)){
 const name=r.cls.split('.').at(-1).replace(/_C$/,'');
 if(beginPlay.has(name))continue;
 if(name==='BP_Lamp_Hanging_Jail')continue; // BeginPlay194 -> Play157; RNG only Finished10.
 for(const fn of r.random_functions){
  if(fn==='UserConstructionScript')continue; // Serialized placed actors, no construction on load.
  assert(!/CallMath[^\n]*:(RandomFloatInRange|RandomInteger)\n/.test(r.functions[fn]),'Unclassified global RNG: '+r.cls+'.'+fn);
 }
}
const rows=[];
for(const [id,map]of Object.entries(data.maps)){
 const expected=map.streams.flatMap(level=>levels[level].flatMap(a=>{
  const name=a.cls.split('.').at(-1).replace(/_C$/,'');const kind=beginPlay.get(name);
  return kind?[{level,actor:a.actor,kind}]:[];
 }));
 assert.deepEqual(map.environment_draws,expected,'Environment actor coverage '+id);
 rows.push(`| ${id} | ${map.streams.length} | ${expected.filter(e=>map.field_streams.includes(e.level)).length} | ${expected.filter(e=>!map.field_streams.includes(e.level)).length} |`);
}
fs.writeFileSync('docs/map-rng-audit.md',`# Map-loading RNG audit — 2026-10-16\n\nAudited ${Object.keys(data.maps).length} modeled maps, ${Object.keys(levels).length} unique streamed levels and ${results.length} Blueprint classes including ancestors. Actor inventories come from cooked Level actor arrays, independently of the importer allowlist. All decoded successfully. Reproduce with \`node tools/audit-map-rng.cjs\`; detailed disassembly is written to \`target/map-rng-audit.json\`.\n\n## Findings\n\n- Southern Sai: six Flag B BeginPlay calls, Western Sai: three, Eastern Ku: one. Fixed in the preceding correction.\n- Shipwreck: one BP_OnWaveObject_A actor, BeginPlay entry342 selects RandomFloatInRange(0,20) and SetNewTime. Added one gondola-bobbing draw. Timeline updates do not repeat that draw.\n- Dng_Cty_3_1 hanging jail lamp: BeginPlay194 jumps to Play157, bypassing RandomFloatInRange at entry10. The Finished callback reaches entry10. Do not add a load draw; a later timeline completion is a separate timing-dependent concern.\n- Light-beam and procedural placement randomness uses private RandomStream functions; it is not the tracked global CRT sequence. Flipbook randomization is construction-script-only on serialized placed actors, not BeginPlay.\n\n## Lifecycle and limits\n\nThe loader initializes field actors on load, reinitializes field actors on battle return, and refreshes NPCs without environment BeginPlay on manual time switches. Hidden battle streams are inventoried but are not initialized on field entry. Counts below are environment calls only; NPC placement, streamed-level constructors and encounter-counter calls remain separate. This is a static Blueprint/actor-coverage audit, not a retail trace of every map or a complete audit of every native component, dynamic spawn, event, or timed callback.\n\n## Every modeled map\n\n| Map | Streams | Field initialization draws | Battle-stream environment draws |\n|---|---:|---:|---:|\n${rows.join('\n')}\n`);
console.log(JSON.stringify({maps:Object.keys(data.maps).length,levels:Object.keys(levels).length,classes:classes.size,blueprints:results.length,errors:results.filter(r=>r.error),random:results.filter(r=>r.random_functions?.length).map(r=>({cls:r.cls,count:r.instances.length,functions:r.random_functions}))},null,2));
