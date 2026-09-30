'use strict';
// Reproduce actor/parent and visit-tag audits from local cooked assets.
const fs=require('node:fs'),path=require('node:path');
const s=require('./source.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const levels=s.table('Level/Database/LevelTable');
const world=s.table('Level/Database/WorldMapTable');
const map='Fld_Cty_1_1', packages=[],actors=[],classes=new Set();
for(const suffix of ['','_Art','_Coll','_GD','_Sound']) {
  const asset='Level/Level/City/'+map+suffix,file=path.join(s.assets.CONTENT,asset+'.umap');
  if(!fs.existsSync(file))continue;
  const p=s.assets.loadPackage(file,true);
  packages.push({asset,sha256:s.assets.hash(fs.readFileSync(file)),functions:p.exports.filter(e=>s.assets.objectPath(p,e.classIndex)==='/Script/CoreUObject.Function').map(e=>e.name)});
  for(const e of p.exports) {
    const c=s.assets.objectPath(p,e.classIndex);
    if(c?.startsWith('/Game/')){classes.add(c);actors.push({level:map+suffix,actor:e.name,class:c});}
  }
}
const audited=new Map();
function inspect(c) {
  if(!c?.startsWith('/Game/')||audited.has(c))return;
  const asset=c.slice(6).split('.')[0];
  const p=s.assets.loadPackage(path.join(s.assets.CONTENT,asset+'.uasset'));
  const generated=p.exports.find(e=>s.assets.objectPath(p,e.classIndex)==='/Script/Engine.BlueprintGeneratedClass');
  const parent=generated?s.assets.objectPath(p,generated.superIndex):null;
  const b=blueprint(asset);
  audited.set(c,{...b,parent});
  inspect(parent);
}
for(const c of classes)inspect(c);
const visit=blueprint('Level/BP/BPC_MJLevelManager',['Proc_StartChangeLevel','SaveVisitedMap_Impl']);
const tutorial=blueprint('Level/BP/BPC_LevelManager_Others',['CanOpenFastTravelTutotial','ShowFastTravelTutorial']);
const out={level:levels.rows.find(r=>r.id===map),world_map:world.rows.find(r=>r.RefLevelName.includes(map)),tables:[{source:levels.source,sha256:levels.sha256},{source:world.source,sha256:world.sha256}],packages,actors,blueprints:[...audited.values()],visit,tutorial};
fs.writeFileSync(path.join(__dirname,'../data/source/harbor-audit.json'),JSON.stringify(out,null,2)+'\n');
console.log(JSON.stringify({packages,random_functions:[...audited.values()].flatMap(b=>Object.entries(b.functions).filter(([,body])=>/Random|rand|FRand/.test(body)).map(([name,body])=>({asset:b.asset,name,calls:body.split('\n').filter(l=>/Random|rand|FRand/.test(l))})))},null,2));
