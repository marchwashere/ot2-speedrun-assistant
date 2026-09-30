'use strict';
// Source evidence and capture expectations. Runtime evaluates imported world rules.
const fs = require('node:fs'), path = require('node:path'), assert = require('node:assert/strict');
const s = require('./source.cjs');
const { audit } = require('../../OT2BattleEngineRust/tools/audit-sequence.cjs');
const d = require('../../OT2BattleEngine/tools/kismet_decoder.cjs');
const sources = [];
function table(name) {
  const t = s.table(name); sources.push({source:t.source,sha256:t.sha256}); return t.rows;
}
function commands(id) {
  const b = fs.readFileSync(path.join(s.assets.CONTENT,'Event/json',id));
  sources.push({source:'/Game/Event/json/'+id,sha256:s.assets.hash(b)});
  return Object.values(JSON.parse(b));
}
function blueprint(asset, name) {
  const file=path.join(s.assets.CONTENT,asset+'.uasset'),p=s.assets.loadPackage(file),q=d.loadPackage(file);
  const e=p.exports.find(e=>e.name===name); assert(e,name);
  const sc=s.extract.extractPackage(p,file).schemas.find(sc=>sc.exportIndex===e.index);
  const decoded=d.decodeFrom(q,name,sc.scriptStart-e.serialOffset,true);
  assert.equal(decoded.exprs.at(-1).name,'EndOfScript');
  sources.push({source:'/Game/'+asset,function:name,sha256:s.assets.hash(p.b)});
  return decoded.exprs.map(e=>d.flatten(e).join('\n')).join('\n');
}
const events=table('Event/Database/EventList'),levels=table('Level/Database/LevelTable');
const eventRows=['0600','0700','0800'].map(n=>events.find(r=>r.id==='MS_TOU_10_'+n));
assert.deepEqual(eventRows.map(r=>r.MapID),['57','57','60']);
assert(eventRows.every(r=>r.EncountReset&&r.EventSkip===1));
const sewer=commands('MS_TOU_10_0700'),city=commands('MS_TOU_10_0800');
assert(sewer.findIndex(c=>c.cmd===9990)<sewer.findIndex(c=>c.cmd===1000));
assert(sewer.some(c=>c.cmd===1000&&c.opt[0]==='MS_TOU_10_0800'));
assert(city.some(c=>c.cmd===1550&&c.opt[0]==='3'));
assert(city.some(c=>c.cmd===500&&c.target===61));
assert(city.findIndex(c=>c.cmd===500)<city.findIndex(c=>c.cmd===9990));
assert(city.findIndex(c=>c.cmd===9990)<city.findIndex(c=>c.cmd===7100));
assert(city.some(c=>c.cmd===7100&&c.target===7));
const tutorial=table('Tutorial/Database/TutorialFlagPart').find(r=>r.ID===7);
assert.equal(tutorial.id,'eRADAR_MAP');

const placement=table('Placement/Database/PlacementData'),lists=table('Placement/Database/PlacementList');
const npcs=table('Character/Database/NPCData'),by=new Map(placement.map(r=>[r.id,r]));
const flags=new Set();
function applyEventFlags(script) {
  let pc=0,steps=0;
  while(pc<script.length) {
    assert(++steps<1024,'event loop');const c=script[pc++];
    if(c.cmd===10000)break;
    if(c.cmd===4000){pc=Number(c.opt[flags.has(c.target)?0:1])+1;continue;}
    if(c.cmd===5){pc=c.target+1;continue;}
    if([70,71].includes(c.cmd)) {
      if(c.opt[0]==='1')flags.add(c.target);else if(c.opt[0]==='0')flags.delete(c.target);
    }
  }
}
for(const suffix of ['0100','0200','0300','0400','0410','0500','0600','0700']) {
  applyEventFlags(commands('MS_TOU_10_'+suffix));
}
function npcPlacements(map) {
  const labels=new Set(lists.find(r=>r.id===map).LabelList);
  function enabled(r,seen=[]) {
    assert(!seen.includes(r.id),'placement exclusion cycle');
    if(r.SpawnStartFlag&&!flags.has(r.SpawnStartFlag)||r.SpawnEndFlag&&flags.has(r.SpawnEndFlag))return false;
    if(!['ETIMEZONE_TRIGGER_TYPE::eALL','ETIMEZONE_TRIGGER_TYPE::eNIGHT'].includes(r.TimeZoneSpawnType))return false;
    // Native ec4900: Throné's chapter 1 (700) is in progress (status 2).
    if(![0,7].includes(r.MainScenarioType)||![0,700].includes(r.MainScenarioID))return false;
    if(r.NorSpawnPrologue&&r.CompProloguePlayerID==='EPlayableCharacterID::eTHIEF')return false;
    if(r.NotSpawnFinal&&flags.has(2008)&&!flags.has(2009))return false;
    return !r.NotCoexistencePlacementLabel.some(id=>labels.has(id)&&enabled(by.get(id),[...seen,r.id]));
  }
  return [...labels].map(id=>by.get(id)).filter(r=>r.PlacementType===1&&enabled(r)).map(r=>{
    assert.equal(r.SpawnStartDelayCount,0);assert.equal(r.SpawnEndDelayCount,0);
    const npc=npcs.find(n=>n.id===r.ResourceLabel);assert(npc,r.ResourceLabel);
    return {placement:r.id,resource:r.ResourceLabel,action:npc.ActionID};
  });
}
const cityA=npcPlacements('Twn_Cty_1_1_A');applyEventFlags(city);
const cityB=npcPlacements('Twn_Cty_1_1_B');
assert.equal(cityA.length,59);assert.equal(cityB.length,26);
// This off-map marker is still an eligible NPC: 0100 set flag 2206. Its
// Epilogue name is not an additional gate. Do not exclude it by name/position.
assert(cityB.some(n=>n.placement==='Epilogue_PLAYER_IS_Throne'));
const allMaps=[];
function walk(dir){for(const e of fs.readdirSync(dir,{withFileTypes:true})) {
  const f=path.join(dir,e.name); if(e.isDirectory())walk(f);else if(e.name.endsWith('.umap'))allMaps.push(f);
}}
walk(path.join(s.assets.CONTENT,'Level/Level'));
function streamed(map) {
  const row=levels.find(r=>r.id===map);assert.equal(row.BarEventLabel,'None');
  assert.deepEqual([row.EncountStepMin,row.RandomStepB,row.RandomStepA],[1000,400,400]);
  assert.deepEqual(row.EncountVolumeNameList,['None']);
  const names=[map,...row.BattleMapNameList.filter(n=>n!=='None')];
  return names.flatMap(n=>['','_Art','_Coll','_GD','_Sound'].flatMap(suffix=>{
    const f=allMaps.find(f=>path.basename(f)===n+suffix+'.umap');
    if(!f)return [];
    sources.push({source:path.relative(s.assets.CONTENT,f).replaceAll('\\','/'),sha256:s.assets.hash(fs.readFileSync(f))});
    return [n+suffix];
  }));
}
const loadsA=streamed('Twn_Cty_1_1_A'),loadsB=streamed('Twn_Cty_1_1_B');
assert.equal(loadsA.length,6);assert.equal(loadsB.length,5);
const bp={};
for(const [asset,names] of [
  ['Level/BP/BPC_MJLevelManager',['RequestLoadFieldLevel','Proc_PrevFadeIn_Exit']],
  ['Level/BP/LevelFunction',['RequestMapChangeForEvent','RequestMapChangeWithBGM']],
  ['Level/BP/BPC_LevelManager_Others',['Proc_OnFinishLoadFieldLevel_Impl','Proc_OnFieldLevelShown_Last_Impl','ResetEncountOnChangeLevel','RefreshPlacementOnChangeLevel','SpawnNPC','RefreshBarCharacter','On Shown Field After Battle Impl']],
  ['Event/BP/EventManagerBP',['FinishPre','EventLoadLevel','StartChangeMap']]
]) for(const name of names)bp[name]=blueprint(asset,name);
assert(bp.RequestLoadFieldLevel.includes('LocalVirtualFunction Proc_OnFinishLoadFieldLevel'));
assert(bp.Proc_OnFinishLoadFieldLevel_Impl.includes('LocalVirtualFunction ResetEncountVolume'));
assert(bp.Proc_OnFieldLevelShown_Last_Impl.includes('LocalVirtualFunction ResetEncountOnChangeLevel'));
assert(bp.SpawnNPC.includes(':RandomIntegerInRange'));
assert(bp.FinishPre.includes('LocalVirtualFunction ResetEncountStep'));
const sequences=['0700','0800'].map(n=>{
  const asset='Event/RichEvent/01_Main/70_TOU/ep1/RI_MS_TOU_10_'+n,keys=audit(asset);
  assert(!keys.some(k=>/Random|SetTimer/.test(k.body)));
  return {asset,keys:keys.length,calls:[...new Set(keys.flatMap(k=>[...k.body.matchAll(/LocalVirtualFunction ([^\n]+)/g)].map(m=>m[1])))]};
});
fs.writeFileSync(path.join(__dirname,'../data/source/throne-city.json'),JSON.stringify({
  status:'Runtime world evaluation matched every live CRT call: drops 5264 -> popup 5374. No fixed-count runtime profile.',
  reported_checkpoint:'reported-city-checkpoint-20260927.json',live_capture:'live-second-city-20260927.json',
  boundary:'Second victory after drops -> skip 0700 and 0800 -> backstreets radar tutorial dismissed',
  assumptions:['Fresh Throné chapter 1, main scenario 700 in progress; Night already active; no unrelated actions.',
    'Both city maps are newly loaded; Btl_SinExpose remains resident from the sewer.',
    'Regular saved NPCs are not defeated, recruited, or otherwise removed; no darkness/late-story overrides.'],
  evidence:{level_colors:'ed31b0 requests Art, Coll, GD, Sound; ed8170 creates only existing sublevels. ec60c0 -> 11c0100 consumes one CRT per fresh streamed level.',
    placements:'ec4900 checks flags, time and story; ec5180 checks exclusion labels; ed5040 refreshes existing NPCs and spawns missing eligible ones.',
    night:'0800 command 1550 sets Night (4) again. f6aa40 -> f6eb70 returns on equal old/new time: no new Black Market lottery.',
    serial_cleanup:'0600 rich battle resumes into 0700, like opening 0200 -> 0300. Do not add a separate suspended-event FinishPre pair.',
    map_resets:'RequestLoadFieldLevel resets the old counter immediately. Proc_PrevFadeIn_Exit then reaches Proc_OnFieldLevelShown_Last_Impl and resets the new map counter.',
    bar:'RefreshBarCharacter exits on BarEventLabel=None before its random placement roll on both maps.',
    popup:'0800 executes 9990 before 7100 (tutorial 7); tutorial close is not a new event or another counter reset.',
    excluded:'Placed scenery uses explicit RandomStreams or baked construction values. No extra shared-level recreation, title-start timer, or Unreal/CRI stream is added.'},
  limitations:'Live validation covers the captured fresh route. World rules and candidates are imported separately; changed world states are tested structurally but need additional live captures.',
  events:eventRows,commands:{sewer,city},sequences,loads:{main:loadsA,backstreets:loadsB},npcs:{main:cityA,backstreets:cityB},blueprints:bp,sources
},null,2)+'\n');
require('./import-town-loading.cjs');
console.log(JSON.stringify({capture_level_colors:loadsA.length+loadsB.length,capture_npc_frames:cityA.length+cityB.length}));
