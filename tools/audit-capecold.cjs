'use strict';
// Source evidence for the on-foot Cape Cold load after recruiting Osvald.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const s=require('./source.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const level=s.table('Level/Database/LevelTable').rows.find(r=>r.id==='Twn_Snw_1_2_A');
assert.equal(level.ID,169); assert.equal(level.BarEventLabel,'None');
const files=new Map();
function walk(dir){for(const e of fs.readdirSync(dir,{withFileTypes:true})) {
 const file=path.join(dir,e.name);if(e.isDirectory())walk(file);else if(e.name.endsWith('.umap'))files.set(e.name.toLowerCase(),file);
}}
walk(path.join(s.assets.CONTENT,'Level/Level'));
const packages=[],classes=new Set(),actors=[];
for(const name of [level.id,...level.BattleMapNameList.filter(n=>n!=='None')])for(const suffix of ['','_Art','_Coll','_GD','_Sound']) {
 const file=files.get((name+suffix+'.umap').toLowerCase());if(!file)continue;
 const actualName=path.basename(file,'.umap'),p=s.assets.loadPackage(file,true);
 packages.push({level:actualName,sha256:s.assets.hash(fs.readFileSync(file))});
 for(const e of p.exports){const c=s.assets.objectPath(p,e.classIndex);if(c?.startsWith('/Game/')){classes.add(c);actors.push({level:actualName,actor:e.name,class:c});}}
}
const audited=new Map();
function inspect(c){if(!c?.startsWith('/Game/')||audited.has(c))return;
 const asset=c.slice(6).split('.')[0],p=s.assets.loadPackage(path.join(s.assets.CONTENT,asset+'.uasset'));
 const generated=p.exports.find(e=>s.assets.objectPath(p,e.classIndex)==='/Script/Engine.BlueprintGeneratedClass');
 const parent=generated?s.assets.objectPath(p,generated.superIndex):null;
 const b=blueprint(asset);audited.set(c,{...b,parent});inspect(parent);
}
for(const c of classes)inspect(c);
const out={level,world_map:s.table('Level/Database/WorldMapTable').rows.find(r=>r.RefLevelName.includes(level.id)),packages,actors,blueprints:[...audited.values()]};
fs.writeFileSync(path.join(__dirname,'../data/source/capecold-audit.json'),JSON.stringify(out,null,2)+'\n');
console.log(JSON.stringify({packages,classes:out.blueprints.map(b=>({asset:b.asset,parent:b.parent})),random_functions:out.blueprints.flatMap(b=>Object.entries(b.functions).filter(([,body])=>/Random|rand|FRand/.test(body)).map(([name,body])=>({asset:b.asset,name,calls:body.split('\n').filter(l=>/Random|rand|FRand/.test(l))})))},null,2));
