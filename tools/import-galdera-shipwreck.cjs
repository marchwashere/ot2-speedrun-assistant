'use strict';
const fs=require('fs'),assert=require('assert/strict'),s=require('./source.cjs');
const objects=s.table('Environment/Database/ObjectData');
const chest=objects.rows.find(r=>r.id==='Treasure_Dng_Ocn_1_1_06');
assert.equal(chest.HaveItemLabel,'ITM_TRE_WPM_03');assert.equal(chest.HaveItemCnt,1);
assert(chest.RandomItemCandidates.every(r=>r.ItemLabel==='None'));
fs.writeFileSync('data/galdera-shipwreck.json',JSON.stringify({source:objects.source,sha256:objects.sha256,chest},null,2)+'\n');
