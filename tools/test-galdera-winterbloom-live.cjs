'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
function find(v){if(v&&typeof v==='object'){if(v.coast&&v.fight)return v;for(const x of Object.values(v)){const r=find(x);if(r)return r;}}}
(async()=>{
 const origin=process.argv[2]||'http://127.0.0.1:54349';
 assert.equal(await(await fetch(origin+'/viewer.js')).text(),fs.readFileSync(path.join(__dirname,'../viewer/viewer.js'),'utf8'));
 for(const [label,voices,first,hp]of [['32','NNNNNNNNSSSNSSNNSNNSNSNSNSSNSSSS','throne',197],['29','SNNNNSSSSSNSNSSSSNNNSNSSNNSNS','pirro',154]]){
  const res=await fetch(origin+'/api/identified-route?'+new URLSearchParams({voices,r3_first:first,hp,route:'galdera'}));assert(res.ok);
  const json=await res.json(),g=find(json);assert(g,'Complete Galdera route must be available');
  const o=g.coast.oresrush_supplies,w=o.winterbloom;assert(w);
  const baseline=label==='32'?{draws:4621,state:3642146968}:{draws:3726,state:173771461};
  assert.deepEqual(o.augmentor.entreat.after,baseline,'Earlier route endpoint must remain unchanged');
  assert.deepEqual(w.arrival.arrival,o.augmentor.entreat.after);
  const state=w.soldier_entreat.world_after;
  assert.equal(state.map,'Twn_Snw_2_1_A');assert.equal(state.time,4);
  assert.equal(state.route_items.ITM_TRE_SS_TSn21_0200_0010,undefined);
  for(const item of ['ITM_TRE_WPM_01','ITM_CSM_0110','ITM_CSM_0550','ITM_EQP_ACS_250'])assert.equal(state.route_items[item],1);
  assert.deepEqual(state.hired_merchant,o.augmentor.entreat.world_after.hired_merchant);
  assert.deepEqual(state.active_scenarios,o.augmentor.entreat.world_after.active_scenarios);
  assert.equal(Number(w.soldier_entreat.after.draws)-Number(w.sword_quest.after.draws),1);
  assert.equal(w.required_level,26);assert(w.agnea_level>=26);assert.equal(w.reward_money,7000);
  const wg=o.wellgrove;assert(wg);
  assert.equal(w.soldier_entreat.after.draws,label==='32'?4658:3763,'Winterbloom endpoint is frozen');
  assert.deepEqual(wg.sai.arrival,w.soldier_entreat.after);
  assert.equal(wg.day.world_after.time,2);
  assert.equal(wg.southern_sai.world_after.map,'Fld_Dst_2_3');
  assert.deepEqual(wg.western_sai.arrival.arrival,wg.southern_sai.after,'No Southern Sai battle');
  assert.equal(wg.western_sai.arrival.world_after.map,'Fld_Dst_2_2');
  assert(['Normal','Preemptive'].includes(wg.western_sai.encounter.situation));
  assert.equal(wg.western_sai.escape.chance_percent,100);assert.equal(wg.western_sai.escape.success,true);
  assert.deepEqual(wg.eastern_wellgrove.arrival,wg.western_sai.field_return.after);
  assert.equal(wg.eastern_wellgrove.world_after.map,'Fld_Fst_2_2');assert.equal(wg.eastern_wellgrove.world_after.time,2);
  assert.deepEqual(wg.eastern_wellgrove.world_after.character_progression,state.character_progression);
  assert.deepEqual(wg.eastern_wellgrove.world_after.route_items,state.route_items);
  console.log(JSON.stringify({fixture:label,extra_sai_switches:wg.additional_time_switches,western_sai_entry:wg.western_sai.arrival.after.draws,western_sai_menu:wg.western_sai.menu.rng.draws,wellgrove_entry:wg.eastern_wellgrove.after.draws}));
  fs.writeFileSync(path.join(__dirname,'../outputs/galdera-winterbloom-'+label+'.json'),JSON.stringify(json));
  console.log(JSON.stringify({fixture:label,arrival:w.arrival.after.draws,quest:w.sword_quest.after.draws,entreat:w.soldier_entreat.after.draws,quest_draws:Number(w.sword_quest.after.draws)-Number(w.arrival.after.draws),counter:w.sword_quest.world_after.encounter_counter}));
 }
})().catch(e=>{console.error(e);process.exitCode=1});

