'use strict';
const fs=require('fs'),path=require('path'),assert=require('assert/strict'),s=require('./source.cjs');
const {blueprint}=require('../../OT2BattleEngineRust/tools/audit-pirro.cjs');
const objects=s.table('Environment/Database/ObjectData');
const chests=['Treasure_Dng_Dst_2_2_01','Treasure_Dng_Dst_2_2_05'].map(id=>objects.rows.find(r=>r.id===id));
assert.deepEqual(chests.map(r=>r.HaveItemLabel),['ITM_TRE_WPM_04','ITM_EQP_LNS_220']);
for(const r of chests){assert.equal(r.HaveItemCnt,1);assert(!r.IsMoney&&!r.IsRevivalByLevelChange);assert(r.RandomItemCandidates.every(c=>c.ItemLabel==='None'));}
const placement=s.table('Placement/Database/PlacementData').rows.filter(r=>chests.some(c=>c.id===r.id));
assert.equal(placement.length,2);for(const r of placement)assert(Object.entries(r).filter(([k])=>k.startsWith('EventLabel_')).every(([,v])=>v==='None'));
const replacement=require('../data/encounters.json').maps.Dng_Dst_2_2.zones.find(r=>r.id==='ENT_DNG_DST_2_2_L_LND');
assert.equal(replacement.conditions[0].replacementFlag,40010);
const out={source:objects.source,sha256:objects.sha256,chests,placement,replacement,trigger:blueprint('Battle/BP/EncountTrigger'),opening:blueprint('Environment/BP/Object/TreasureBoxBP',['Open','OpenDialog','PickItem','IsRandomItem','DialogFinishCallback','ExecuteUbergraph_TreasureBoxBP'])};
fs.writeFileSync(path.join(__dirname,'../data/source/galdera-quicksand-loot.json'),JSON.stringify(out,null,2)+'\n');
console.log('Audited fixed Rusty Axe and Scorched Bone Spear chests');
