'use strict';
const fs=require('node:fs'),path=require('node:path');
const s=require('./source.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const {audit}=require('../../OT2BattleEngineRust/tools/audit-sequence.cjs');
const data=JSON.parse(fs.readFileSync(path.join(__dirname,'../data/town-loading.json')));
const files=new Map();
function walk(dir){for(const e of fs.readdirSync(dir,{withFileTypes:true})){
  const f=path.join(dir,e.name);if(e.isDirectory())walk(f);else if(e.name.endsWith('.umap'))files.set(e.name.toLowerCase(),f);
}}
walk(path.join(s.assets.CONTENT,'Level/Level'));
const blueprints=new Map(),packages=[];
function inspect(c){
  if(!c?.startsWith('/Game/')||blueprints.has(c))return;
  const asset=c.slice(6).split('.')[0],p=s.assets.loadPackage(path.join(s.assets.CONTENT,asset+'.uasset'));
  const generated=p.exports.find(e=>s.assets.objectPath(p,e.classIndex)==='/Script/Engine.BlueprintGeneratedClass');
  const parent=generated?s.assets.objectPath(p,generated.superIndex):null;
  const b=blueprint(asset);blueprints.set(c,{...b,parent});inspect(parent);
}
for(const map of ['Fld_Ocn_1_1','Fld_Wld_1_2'])for(const level of data.maps[map].streams){
  const file=files.get((level+'.umap').toLowerCase()),p=s.assets.loadPackage(file,true);
  const classes=[...new Set(p.exports.map(e=>s.assets.objectPath(p,e.classIndex)).filter(c=>c?.startsWith('/Game/')))];
  for(const c of classes)inspect(c);
  packages.push({level,sha256:s.assets.hash(fs.readFileSync(file)),classes});
}
const shop=blueprint('UserInterface/Shop/BP/ShopMenuWidget',['StartMenuMode_LinerShip','UpdateState_LinerShip','RequestLevelChangeByLinerShip','UIOpen','CompleteTrade']);
const shipRoute=s.table('Shop/Database/LinerShipRoute');
const event=s.table('Event/Database/EventList').rows.find(r=>r.id==='SC_LinerShip_Cty_to_Wld');
const scene=audit('Event/RichEvent/24_LinerShip/RI_LINERSHIP_CTY_to_WLD');
const out={route:shipRoute.rows.find(r=>r.id==='SHIP_ROUTE_001'),route_source:{source:shipRoute.source,sha256:shipRoute.sha256},event,
  commands:data.events.SC_LinerShip_Cty_to_Wld.commands,scene,packages,blueprints:[...blueprints.values()],shop};
fs.writeFileSync(path.join(__dirname,'../data/source/crackridge-voyage-audit.json'),JSON.stringify(out,null,2)+'\n');
console.log(JSON.stringify({packages:packages.map(p=>p.level),random_functions:out.blueprints.flatMap(b=>Object.entries(b.functions)
  .filter(([,body])=>/Random|rand|FRand/.test(body)).map(([name,body])=>({asset:b.asset,name,calls:body.split('\n').filter(l=>/Random|rand|FRand/.test(l))})))},null,2));
