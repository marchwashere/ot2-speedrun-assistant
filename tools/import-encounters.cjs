'use strict';
// Build-time import. Runtime uses only the checked-in, compact catalog.
const fs = require('node:fs'), path = require('node:path'), s = require('./source.cjs');
const sources = [];
function table(name) { const t=s.table(name); sources.push({source:t.source,sha256:t.sha256,rows:t.rows.length}); return t.rows; }
const levels=table('Level/Database/LevelTable');
const triggers=table('Battle/Database/EncountTriggerData');
const volumes=table('Battle/Database/EncountVolumeData');
const encounters=table('Battle/Database/BattleEncounterData');
const groups=table('Battle/Database/EnemyGroupData');
const enemies=table('Character/Database/EnemyDB');
const params=table('GameParam/GameParamDefineTable');
const mappingFile=path.join(s.root,'outputs/encounter-zones/zone-data.json');
const mappingBytes=fs.readFileSync(mappingFile), mapping=JSON.parse(mappingBytes);
const textBytes=fs.readFileSync(path.join(s.assets.CONTENT,'GameText/Database/GameTextEN.uexp'));
function text(key) {
  const needle=Buffer.from(key+'_Text\0'), start=textBytes.indexOf(needle);
  if(start<0)return '';
  const p=start+needle.length, n=textBytes.readInt32LE(p);
  return n>0 ? textBytes.toString('utf8',p+4,p+4+n-1) : textBytes.toString('utf16le',p+4,p+4+(-n-1)*2);
}
const object = rows => Object.fromEntries(rows.map(r=>[r.id,r]));
const rare = r => ({enabled:r.RareEnemyFlag,rate:r.RareEnemyRate,counts:[r.RareEnemyCnt_Low,r.RareEnemyCnt_Middle,r.RareEnemyCnt_High]});
const catalog={
  maps:object(levels.map(r=>({id:r.id,name:text(r.id)||mapping.rows.find(z=>z.map===r.id)?.mapName||r.id,
    counter:{min:r.EncountStepMin,range_a:r.RandomStepA,range_b:r.RandomStepB,first_ratio:r.FirstEncountStepRatio},
    zones:mapping.rows.filter(z=>z.map===r.id).map(z=>({id:z.trigger,mapping:z.placements.length?'placed':'inferred',conditions:z.placements.filter(p=>p.replacementFlag||p.startFlags.some(Boolean)||p.endFlags.some(Boolean))}))}))),
  zones:object(triggers.map(r=>({id:r.id,flags:[r.StartFlag,r.StartFlag_2,r.StartFlag_3,r.EndFlag,r.EndFlag_2,r.EndFlag_3],
    periods:Object.fromEntries(['DayTime','Night','LostWay','DarkArea'].map(p=>[p,{volume:r[p+'EncountVolumeName'],decrement:r[p+'DecreaseValue']}]))}))),
  volumes:object(volumes.map(r=>({id:r.id,area:r.Area.split('::').at(-1),water:r.WaterSide,flags:[r.StartFlag,r.EndFlag],cait:rare(r.RareCatlinParam),octopuff:rare(r.RareOctlinParam),
    encounters:r.EncounterList.map(e=>({progress:e.ProgressBorder,level:e.DisplayLevel,danger:e.DangerLevel,table:e.EncounterName}))}))),
  encounters:object(encounters.map(r=>({id:r.id,first:r.FirstEncount,groups:r.Group.map(g=>({id:g.GroupID,weight:g.Appearance}))}))),
  groups:object(groups.map(r=>({id:r.id,enemies:r.EnemyID}))),
  enemies:object(enemies.map(r=>({id:r.id,name:text(r.DisplayNameID)||r.id,level:r.EnemyLevel,size:r.Size.split('::').at(-1)}))),
  rare_labels:Object.fromEntries(params.filter(r=>r.id.startsWith('RARE_ENEMY_')).map(r=>[r.id.replace('RARE_ENEMY_',''),[r.Min,r.Max,r.Init]])),
  dash_multiplier:Number(params.find(r=>r.id==='ENCOUNT_DASH_REVISION').Param),
  coverage:mapping.stats
};
for(const r of Object.values(catalog.volumes)) for(const e of r.encounters) if(e.table!=='None'&&!catalog.encounters[e.table]) throw Error('Missing encounter '+e.table);
// Cooked tables retain unused/missing references. Preserve them; prediction
// rejects incomplete active rows instead of guessing a replacement formation.
catalog.coverage.missing_weighted_groups=Object.values(catalog.encounters).flatMap(r=>r.groups.filter(g=>g.weight>0&&!catalog.groups[g.id]).map(g=>({table:r.id,group:g.id})));
const out=path.resolve(__dirname,'../data');
fs.writeFileSync(path.join(out,'encounters.json'),JSON.stringify(catalog)+'\n');
fs.writeFileSync(path.join(out,'source/encounters.json'),JSON.stringify({sources,mapping:{source:'outputs/encounter-zones/zone-data.json',sha256:s.assets.hash(mappingBytes),stats:mapping.stats,unmatched:mapping.noMap},text_sha256:s.assets.hash(textBytes),functions:['EncountManagerBP.CalcEncountStepCount','EncountManagerBP.CalcDecreaseStepCount','EncountManagerBP.SetEncounter','EncountManagerBP.JudgeBattleSituation','LibBattle.LotteryEncountData','LibRareEnemy.GetAdditionalRareEnemyRate','BattleManagerBP.CreateEnemyCharacter','BattleManagerBP.CreateRareEnemyReplaceList']},null,2)+'\n');
console.log(JSON.stringify({maps:levels.length,zones:triggers.length,volumes:volumes.length,encounters:encounters.length,groups:groups.length,enemies:enemies.length,bytes:fs.statSync(path.join(out,'encounters.json')).size}));
