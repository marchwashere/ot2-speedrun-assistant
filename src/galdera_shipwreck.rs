//! Owned boat, no sea encounter, one land encounter, Rusty Dagger.
//! Candidate only: automatic JP-location selection is not activated here.
use crate::{Result,ensure,world::{self,Loader},snows_escape::Entry,galdera_flex::{point,Segment},encounter::{self,PredictionOptions,Boundary}};
use serde::Serialize;
pub const MAP:&str="Dng_Ocn_1_1";
#[derive(Debug,Serialize)]pub struct JpBattle {pub encounter:encounter::Prediction,pub battle_entry:crate::CrtRng,pub fight:ot2_battle_engine::gravell_routine::Fight,pub rewards:ot2_battle_engine::gravell_routine::Rewards,pub field_return:Entry}
#[derive(Debug,Serialize)]pub struct Trip {pub travel:Segment,pub escape:Option<crate::galdera_coast::SouthernSai>,pub escape_inputs:Vec<u8>,pub jp:Option<JpBattle>,pub dagger:Entry,pub jp100_complete:bool}
/// Caller has purchased the boat. The specified route has exactly one land
/// encounter; no unmeasured distance or encounterless guarantee is fabricated.
pub fn run(entry:&Entry,jp100:bool,max_inputs:usize)->Result<Option<Trip>> {run_observed(entry,jp100,max_inputs,None)}
pub(crate) fn jp_probe(entry:&Entry)->Result<Option<crate::CrtRng>> {
 let mut probe=None;run_observed(entry,true,0,Some(&mut probe))?;Ok(probe)
}
fn run_observed(entry:&Entry,jp100:bool,max_inputs:usize,probe:Option<&mut Option<crate::CrtRng>>)->Result<Option<Trip>> {
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 ensure(!l.state.opened_route_chests.contains("Treasure_Dng_Ocn_1_1_06"),"Rusty Dagger chest already opened")?;
 let mut travel=Segment::new("Crackridge Harbor to Shipwreck");
 let before=l.rng;l.visit_map("Fld_Wld_1_2")?;travel.save(&mut l,"Teleport to Crackridge Harbor",before);
 let before=l.rng;l.visit_map("Fld_Ocn_1_1")?;travel.save(&mut l,"Board owned boat; enter the sea",before);
 let before=l.rng;l.visit_map(MAP)?;travel.save(&mut l,"Enter Shipwreck; no sea encounters",before);
 let before=l.rng;let period=if jp100{4}else{2};
 if l.state.time!=period{l.manual_time_switch(period,if jp100{"Shipwreck: night for JP100"}else{"Shipwreck: day for Escape"})?;}
 if jp100 {crate::nameless_setup::apply(&mut l)?;}
 travel.save(&mut l,if jp100{"Night; retain/apply Osvald and Throne gear menu"}else{"Stay during day"},before);
 let e=travel.end();
 let (escape,escape_inputs,jp,ret)=if jp100 {
  let Some(j)=jp_battle(e,max_inputs,probe)?else{return Ok(None);};let ret=j.field_return.clone();(None,vec![],Some(j),ret)
 }else{
  let Some((f,inputs))=escape_battle(e,max_inputs)?else{return Ok(None);};let ret=f.field_return.clone();(Some(f),inputs,None,ret)
 };
 let mut l=Loader::new(world::catalog(),ret.world_after.clone(),ret.after)?;
 let data:serde_json::Value=serde_json::from_str(include_str!("../data/galdera-shipwreck.json"))?;let c=&data["chest"];
 ensure(l.state.opened_route_chests.insert(c["id"].as_str().unwrap().into()),"Rusty Dagger chest already opened")?;
 *l.state.route_items.entry(c["HaveItemLabel"].as_str().unwrap().into()).or_default()+=1;
 l.boundary("Open Rusty Dagger chest after the specified single land encounter; fixed chest, no RNG draw; remaining distance unmeasured");
 let dagger=point(&mut l,ret.after);
 Ok(Some(Trip{travel,escape,escape_inputs,jp,dagger,jp100_complete:jp100}))
}
fn escape_battle(e:&Entry,max_inputs:usize)->Result<Option<(crate::galdera_coast::SouthernSai,Vec<u8>)>> {
 let w=&e.world_after;let mut rng=e.after;let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
 let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions{map:MAP.into(),zone:"ENT_DNG_OCN_1_1_LND".into(),period:"DayTime".into(),progress:w.main_progress_points,first_encounter:w.first_encounter,boundary:Boundary::CurrentCounter,current_counter:Some(w.encounter_counter),party_levels:levels.clone(),ability_percent:if w.evasive_maneuvers{50}else{100},dash:true,..Default::default()})?;
 let situation=match encounter.situation.as_deref(){Some("Normal")=>ot2_battle_engine::escape::Situation::Normal,Some("Preemptive")=>ot2_battle_engine::escape::Situation::Preemptive,_=>return Ok(None)};
 let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();let ids:Vec<_>=encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
 for id in &w.party{ensure(w.character_progression[id].jobs.equipped_support.contains(crate::gravell_setup::STEP_AHEAD),"Shipwreck Escape expects four A Step Ahead users")?;}
 let menu=ot2_battle_engine::field_escape::FieldEscapeMenu::with_step_ahead(rng,situation,&players,&levels,&ids,&w.enemy_knowledge)?;
 for cycles in 0..=max_inputs/2 {
  let inputs:Vec<_>=(0..cycles).flat_map(|_|[1,0]).collect();let escape=menu.clone().attempt(&inputs,encounter.encounter_level.unwrap())?;
  if !escape.success{continue;}
  let ret=crate::highroad::restore_field(w,&escape,MAP,"Shipwreck daytime Escape")?;
  let field_return=Entry{arrival:ret.arrival,after:ret.after,world_after:ret.world_after,trace:ret.trace,decisions:vec![]};
  return Ok(Some((crate::galdera_coast::SouthernSai{arrival:e.clone(),encounter,battle_entry:rng,menu,escape,field_return,defends:vec![],defend_rounds:vec![],escape_round:1},inputs)));
 }Ok(None)
}
fn jp_battle(e:&Entry,max_inputs:usize,probe:Option<&mut Option<crate::CrtRng>>)->Result<Option<JpBattle>> {
 let w=&e.world_after;ensure(w.party==[7,6,5,4]&&w.time==4,"Shipwreck JP needs Galdera party at night")?;
 for id in [7,6,5,4]{ensure(w.character_progression[&id].jobs.equipped_support.contains(crate::gravell_setup::STEP_AHEAD),"Shipwreck needs four A Step Ahead users")?;}
 ensure(w.character_progression[&5].jobs.learned_abilities.get(&5).is_some_and(|a|a.contains("ABI_SET_DAN_070")),"Temenos needs BG")?;
 ensure(w.character_progression[&7].jobs.secondary_job==Some(11)&&w.inventor_abilities.contains("ABI_SET_INV_020"),"Throne needs Springy Boots")?;
 ensure(w.character_progression[&4].jobs.learned_abilities.get(&0).is_some_and(|a|a.contains("ABI_SET_MER_050")),"Partitio needs Donate BP")?;
 let mut rng=e.after;
 let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions{map:MAP.into(),zone:"ENT_DNG_OCN_1_1_LND".into(),period:"Night".into(),progress:w.main_progress_points,first_encounter:w.first_encounter,boundary:Boundary::CurrentCounter,current_counter:Some(w.encounter_counter),party_levels:w.party.iter().map(|id|w.character_progression[id].level).collect(),ability_percent:if w.evasive_maneuvers{50}else{100},dash:true,..Default::default()})?;
 let situation=match encounter.situation.as_deref(){Some("Normal")=>ot2_battle_engine::escape::Situation::Normal,Some("Preemptive")=>ot2_battle_engine::escape::Situation::Preemptive,_=>return Ok(None)};
 let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();let ids:Vec<_>=encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
 let b=ot2_battle_engine::field_escape::shipwreck_jp_battle(rng,situation,&players,&ids,&w.enemy_knowledge)?;
 if let Some(probe)=probe {
  if let Some((menu,_))=ot2_battle_engine::nameless_routine::prefix(b)? {
   *probe=Some(crate::CrtRng{state:menu.state().rng.state,draws:rng.draws+menu.state().rng.draws});
  }return Ok(None);
 }
 let Some(f)=ot2_battle_engine::nameless_routine::search_shipwreck(b,max_inputs)?else{return Ok(None);};
 let (fight,rewards,mut field_return)=crate::inventor::settle_gravell(e,rng,f)?;
 if !crate::galdera_jp::sufficient(&field_return.world_after)? {return Ok(None);}
 for(id,&n)in &rewards.drops{*field_return.world_after.route_items.entry(id.clone()).or_default()+=n;}
 Ok(Some(JpBattle{encounter,battle_entry:rng,fight,rewards,field_return}))
}
#[cfg(test)]mod tests {
 use super::*;
 fn entry(seed:u32)->Entry {
  let v:serde_json::Value=serde_json::from_str(include_str!("../data/source/galdera-flex-entry.json")).unwrap();
  let w:world::WorldState=serde_json::from_value(v["world_after"].clone()).unwrap();
  let mut l=Loader::new(world::catalog(),w,crate::CrtRng::new(seed)).unwrap();
  crate::gravell_setup::learn(&mut l.state,6,0,&["ABI_SET_MER_030"]).unwrap();crate::gravell_setup::jp_setup(&mut l).unwrap();
  crate::nameless_setup::track_augmentation_for_legacy_fixture(&mut l.state);
  // Old pre-flex fixture; explicitly supply acquired gear for this branch test.
  for (id,n) in [("ITM_EQP_SWD_380",1),("ITM_EQP_LNS_220",1),("ITM_EQP_HLM_180",1),("ITM_EQP_ACS_250",1),("ITM_EQP_SLD_240",1),("ITM_EQP_AMR_300",1),("ITM_CSM_0610",2),("ITM_CSM_0730",1),("ITM_CSM_0550",1),("ITM_CSM_0070",0)]{l.state.route_items.insert(id.into(),n);}
  point(&mut l,crate::CrtRng::new(seed))
 }
 #[test]fn day_escape_and_night_jp_both_award_dagger() {
  for jp in [false,true] {
   let mut found=false;
   for seed in 0u32..128 {
    let e=entry(seed.wrapping_mul(2654435761));let Some(t)=run(&e,jp,64).unwrap()else{continue;};found=true;
    assert_eq!(t.dagger.world_after.map,MAP);assert_eq!(t.dagger.world_after.time,if jp{4}else{2});
    assert_eq!(t.dagger.world_after.route_items["ITM_TRE_WPM_03"],1);assert_eq!(t.dagger.arrival,t.dagger.after);
    assert_eq!(t.travel.stages[0].1.world_after.map,"Fld_Wld_1_2");assert_eq!(t.travel.stages[1].1.world_after.map,"Fld_Ocn_1_1");
    assert_eq!(t.jp100_complete,jp);
    if let Some(j)=t.jp {assert!(t.escape.is_none());assert_eq!(j.fight.actions.last().unwrap().action,ot2_battle_engine::gravell_routine::Action::FireballMax);assert_eq!(j.rewards.hired_help_cost,0);assert!(j.rewards.awarded.jp>=12210);assert_eq!(t.dagger.world_after.route_items.get("ITM_CSM_0070").copied().unwrap_or(0),0);}
    else {let f=t.escape.unwrap();assert!(f.escape.success);assert_eq!(t.dagger.world_after.route_items.get("ITM_CSM_0070").copied().unwrap_or(0),0);}
    break;
   }assert!(found,"Missing branch jp={jp}");
  }
 }
}
