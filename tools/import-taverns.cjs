const fs=require('node:fs'),s=require('./source.cjs');
const levels=s.table('Level/Database/LevelTable'),towns=s.table('Level/Database/TownTable'),text=s.table('GameText/Database/GameTextEN'),params=s.table('GameParam/GameParamDefineTable');
const rate=id=>Number(params.rows.find(r=>r.id===id).Param);
const maps=Object.fromEntries(levels.rows.filter(r=>r.BarEventLabel!=='None').map(r=>{
 const town=towns.rows.find(t=>t.ID===r.TownLabel);
 return [r.id,{name:text.rows.find(t=>t.id===town?.TownName)?.Text.source??town?.TownName,slots:r.BarPCPos2[0]<=-10000?1:2,allowed:r.BarPlacementChara}];
}));
fs.writeFileSync('data/taverns.json',JSON.stringify({no_appearance_rate:rate('BAR_PLACE_CHARA_RATE'),one_character_rate:rate('BAR_PLACE_CHARA_ONE_RATE'),gift_rate:rate('BAR_ITEM_GIFT_RATE'),maps},null,2)+'\n');
console.log(maps);
