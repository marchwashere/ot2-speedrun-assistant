//! Inactive Nameless Isle candidate: run to water, one Escape, then Gigantes.
//! Caller supplies daytime island entry after Roque loot and Firestarter.
//! Menu, approach and boss are modeled; automatic JP location selection is inactive.
use crate::{Result,ensure,world::{self,Loader},snows_escape::Entry,galdera_coast::{self,SouthernSai},galdera_flex::point};
use serde::Serialize;
pub const MAP:&str="Dng_Ocn_1_3";
pub const LAND_TO_WATER:u32=70;
pub const WATER_RUN:u32=67;
pub const TOTAL_RUN:u32=318;
pub const LAND_WALK:u32=73;
pub const WATER_WALK:u32=69;
pub const FINAL_RUN:u32=181;
pub const FINAL_WALK:u32=191;
pub const END_LEEWAY:u32=3;
#[derive(Debug,Serialize)]pub struct JpTrip {
 pub setup:Entry,pub water_entry:Entry,pub encounter:crate::encounter::Prediction,
 pub battle_entry:crate::CrtRng,pub fight:ot2_battle_engine::gravell_routine::Fight,
 pub rewards:ot2_battle_engine::gravell_routine::Rewards,pub field_return:Entry,
 pub movement:Movement,pub gigantes_entry:Entry,
}
/// Candidate for the pending automatic location selector. No opening-order
/// manipulation: the caller adjusts upstream RNG for the 25% formation.
/// JP manipulation ends at this victory, not at Gigantes or Georges.
pub fn jp_finish(entry:&Entry,max_inputs:usize)->Result<Option<JpTrip>> {jp_observed(entry,max_inputs,None)}
pub(crate) fn jp_probe(entry:&Entry)->Result<Option<crate::CrtRng>> {
 let mut probe=None;jp_observed(entry,0,Some(&mut probe))?;Ok(probe)
}
fn jp_observed(entry:&Entry,max_inputs:usize,probe:Option<&mut Option<crate::CrtRng>>)->Result<Option<JpTrip>> {
 use crate::encounter::{self,PredictionOptions,Boundary};
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 ensure(l.state.map==MAP&&l.state.time==2&&l.state.evasive_maneuvers,"Nameless JP approach requires daytime island entry with EM")?;
 ensure(l.state.party==[7,6,5,4],"Unexpected Nameless party")?;
 for id in [7,6,5,4] {ensure(l.state.character_progression[&id].jobs.equipped_support.contains(crate::gravell_setup::STEP_AHEAD),"Nameless JP requires four A Step Ahead users")?;}
 ensure(l.state.character_progression[&5].jobs.learned_abilities.get(&5).is_some_and(|a|a.contains("ABI_SET_DAN_070")),"Temenos must know Bewildering Grace")?;
 ensure(l.state.character_progression[&7].jobs.secondary_job==Some(11)&&l.state.inventor_abilities.contains("ABI_SET_INV_020"),"Throne needs Inventor and Springy Boots")?;
 ensure(l.state.character_progression[&4].jobs.learned_abilities.get(&0).is_some_and(|a|a.contains("ABI_SET_MER_050")),"Partitio needs Donate BP")?;
 l.state.enter_encounter_zone("ENT_DNG_OCN_1_3_LND")?;
 crate::nameless_setup::apply(&mut l)?;let setup=point(&mut l,entry.after);
 let options=PredictionOptions{map:MAP.into(),zone:"ENT_DNG_OCN_1_3_WTR".into(),period:"DayTime".into(),progress:l.state.main_progress_points,first_encounter:l.state.first_encounter,boundary:Boundary::CurrentCounter,party_levels:l.state.party.iter().map(|id|l.state.character_progression[id].level).collect(),ability_percent:50,dash:true,..Default::default()};
 let run=encounter::catalog().counter_decrement(&options)?;
 let walk=encounter::catalog().counter_decrement(&PredictionOptions{dash:false,..options.clone()})?;
 ensure(l.state.encounter_counter>LAND_TO_WATER*run,"Encounter before water")?;
 l.state.encounter_counter-=LAND_TO_WATER*run;l.boundary("Run 70 decrements to Nameless water; canoe retains counter");
 l.state.enter_encounter_zone("ENT_DNG_OCN_1_3_WTR")?;
 let water_entry=point(&mut l,setup.after);let w=&water_entry.world_after;let mut rng=water_entry.after;
 let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions{current_counter:Some(w.encounter_counter),..options})?;
 if encounter.enemies.len()!=3||encounter.enemies.iter().any(|e|e.enemy.id!="ENE_NML_WTR_350"){return Ok(None);}
 let situation=match encounter.situation.as_deref(){Some("Normal")=>ot2_battle_engine::escape::Situation::Normal,Some("Preemptive")=>ot2_battle_engine::escape::Situation::Preemptive,_=>return Ok(None)};
 let first=encounter.counter_updates.unwrap();ensure(first<=WATER_RUN,"Nameless encounter outside water crossing")?;
 let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
 let b=ot2_battle_engine::field_escape::nameless_jp_battle(rng,situation,&players,&w.enemy_knowledge)?;

 if let Some(probe)=probe {
  if let Some((menu,_))=ot2_battle_engine::nameless_routine::prefix(b)? {
   *probe=Some(crate::CrtRng{state:menu.state().rng.state,draws:rng.draws+menu.state().rng.draws});
  }return Ok(None);
 }
 let f=ot2_battle_engine::nameless_routine::search_accepting(b,max_inputs,|fight|{
  let (_,_,ret)=crate::inventor::settle_gravell(&water_entry,rng,fight.clone())?;
  Ok(movement(ret.world_after.encounter_counter,first,run,walk).is_some())
 })?;
 let Some(f)=f else{return Ok(None);};
 let (fight,rewards,mut field_return)=crate::inventor::settle_gravell(&water_entry,rng,f)?;
 for (id,&n) in &rewards.drops {*field_return.world_after.route_items.entry(id.clone()).or_default()+=n;}
 let movement=movement(field_return.world_after.encounter_counter,first,run,walk).unwrap();
 let mut l=Loader::new(world::catalog(),field_return.world_after.clone(),field_return.after)?;
 l.state.encounter_counter=movement.counter_at_exit;
 l.boundary(format!("JP100 complete: run {:.2}% of total island distance, walk {:.2}%; reserve 3 running decrements, no second encounter before Gigantes",movement.run_percent,movement.walk_percent));
 l.state.enter_encounter_zone("ENT_DNG_OCN_1_3_LND")?;
  let gigantes_entry=point(&mut l,field_return.after);
 Ok(Some(JpTrip{setup,water_entry,encounter,battle_entry:rng,fight,rewards,field_return,movement,gigantes_entry}))
}
#[derive(Debug,Serialize)]pub struct Trip {pub setup:Entry,pub approach:Approach,pub night:Entry,pub prompt:Entry,pub damage_floor:ot2_battle_engine::gigantes::DamageFloor,pub battle:ot2_battle_engine::lord_sands::Prediction,pub field_return:Entry,pub victory:Entry,pub loot:crate::galdera_flex::Segment,pub finisher_boost_cycles:u32}
/// Inactive continuation for the future automatic location selector. The caller
/// must complete Roque loot and Firestarter first, and load the island by day.
pub fn finish(entry:&Entry,max_cycles:u32)->Result<Option<Trip>> {finish_with_defends(entry,max_cycles,0)}
pub(crate) fn finish_with_defends(entry:&Entry,max_cycles:u32,defends:u8)->Result<Option<Trip>> {
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 crate::nameless_setup::apply(&mut l)?;
 for id in [7,6,5,4] {ensure(l.state.character_progression[&id].jobs.equipped_support.contains(crate::gravell_setup::STEP_AHEAD),"Nameless routine requires A Step Ahead on all four")?;}
 ensure(l.state.character_progression[&6].jobs.learned_abilities.get(&6).is_some_and(|a|a.contains("ABI_SET_SCH_060")),"Osvald must know Elemental Barrage")?;
 let setup=point(&mut l,entry.after);
 let Some(approach)=escape_approach_with_defends(&setup,max_cycles,defends)?else{return Ok(None);};
 let e=&approach.gigantes_entry;let mut l=Loader::new(world::catalog(),e.world_after.clone(),e.after)?;
 ensure(!l.state.flag(40003),"Gigantes already defeated")?;
 l.manual_time_switch(4,"Nameless Isle: switch to night before Gigantes")?;
 let night=point(&mut l,e.after);
 let request=l.skipped_event_until_battle("EV_Dng_Ocn_1_3_0010",&world::EventChoices::from([(("EV_Dng_Ocn_1_3_0010".into(),0),"eDIALOG_YES".into())]))?;
 ensure(request.volume=="EVM_DNG_OCN_1_3_L","Unexpected Gigantes volume")?;
 // Scripted bosses bypass the field selector's player timing initialization.
 for id in l.state.party.clone() {l.draw(world::DrawKind::BattleTiming,&id.to_string(),"Gigantes: BattlePlayerBP.Initialize timing",None);}
 let prompt=point(&mut l,night.after);
 let players:Vec<_>=l.state.party.iter().map(|id|l.state.character_progression[id].character.clone()).collect();
 let damage_floor=ot2_battle_engine::gigantes::damage_floor(&players)?;
 ensure(damage_floor.total>=50000,"Gigantes gear does not meet the conservative damage floor")?;
 // Replay the complete finisher and subsequent callbacks for each adjustment;
 // damage, drops, field restoration and day switch all affect the final check.
 for cycles in 0..=max_cycles {
 let mut l=Loader::new(world::catalog(),prompt.world_after.clone(),prompt.after)?;
 let battle=ot2_battle_engine::gigantes::predict_with_finisher_cycles(l.rng,&players,&l.state.enemy_knowledge,cycles)?;
 for (&id,r) in l.state.party.iter().zip(&battle.party_resources_after){l.state.character_progression.get_mut(&id).unwrap().character.resources=Some(r.clone());}
 l.state.enemy_knowledge=battle.enemy_knowledge.clone();l.state.award_victory(battle.rewards)?;
 for (item,&n) in &battle.drops {*l.state.route_items.entry(item.clone()).or_default()+=n;}
 l.rng=battle.after;l.restore_field_after_battle("Gigantes victory: field restored")?;
 let field_return=point(&mut l,battle.after);
 l.skipped_event(&request.victory_event)?;
 ensure(l.state.flag(40003),"Gigantes defeat flag missing")?;
 let victory=point(&mut l,field_return.after);
 let Some(loot)=post_gigantes(&victory)?else{continue;};
 return Ok(Some(Trip{setup,approach,night,prompt,damage_floor,battle,field_return,victory,loot,finisher_boost_cycles:cycles}));
 }
 Ok(None)
}
/// Reject unsuccessful Scrutinize candidates; never award information on failure.
fn post_gigantes(entry:&Entry)->Result<Option<crate::galdera_flex::Segment>> {
 #[derive(serde::Deserialize)]struct Georges {npc:String,robe:String,steal_level:u32,information:String,chances:std::collections::BTreeMap<String,u32>,before:Vec<String>,success:String,chest:String,claws:String}
 #[derive(serde::Deserialize)]struct Data {georges:Georges}
 let d:Data=serde_json::from_str(include_str!("../data/galdera-nameless.json"))?;let g=d.georges;
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 ensure(l.state.map==MAP&&l.state.flag(40003),"Georges requires Gigantes rescue scene")?;
 ensure(!l.state.route_items.contains_key(&g.information),"Georges already Scrutinized")?;
 let mut s=crate::galdera_flex::Segment::new("Georges and Finisher's Claws");
 let before=l.rng;l.manual_time_switch(2,"After Gigantes and Georges rescue scene: switch to day")?;s.save(&mut l,"Switch to day",before);
 ensure(l.state.party.contains(&7)&&l.state.party.contains(&6),"Georges needs Throne and Osvald")?;
 ensure(l.state.existing_npcs.iter().any(|id|l.data.placements[id].resource==g.npc),"Georges not present")?;
 ensure(l.state.character_progression[&7].level>=g.steal_level,"Mage's Robe Steal is not guaranteed")?;
 let before=l.rng;l.draw(world::DrawKind::PathActionMenu,&g.npc,"Georges Steal menu opening",None);
 l.draw(world::DrawKind::StealCheck,&g.robe,"Guaranteed Mage's Robe Steal",Some(100));
 *l.state.route_items.entry(g.robe).or_default()+=1;
 l.boundary("Switch directly from Steal to Scrutinize in the open path-action menu; suppress Steal dialogue without closing or reopening");
 s.save(&mut l,"Steal Mage's Robe; switch directly to Scrutinize",before);
 let before=l.rng;
 let chance=g.chances[&l.state.character_progression[&6].level.to_string()];
 let roll=l.draw(world::DrawKind::ScrutinizeCheck,&g.npc,format!("Scrutinize success check ({chance}%)"),Some(100))+1;
 if roll>chance{return Ok(None);}
 *l.state.route_items.entry(g.information).or_default()+=1;
 let line=l.draw(world::DrawKind::DialogueLottery,&g.npc,"Scrutinize before dialogue",None);
 l.skipped_event(&g.before[usize::from(line>16383)])?;
 l.skipped_event(&g.success)?;
 s.save(&mut l,"Scrutinize Georges: How to Decipher Unknown Languages",before);
 let before=l.rng;ensure(l.state.opened_route_chests.insert(g.chest),"Finisher's Claws chest already opened")?;
 *l.state.route_items.entry(g.claws).or_default()+=1;
 l.boundary("Open Finisher's Claws chest; fixed item, no RNG draw");
 s.save(&mut l,"Open Finisher's Claws chest",before);
 Ok(Some(s))
}
#[derive(Debug,Serialize)]pub struct Movement {
 pub water_run:u32,pub water_walk:u32,pub land_run:u32,pub land_walk:u32,
 pub leeway_updates:u32,pub counter_cost:u32,pub counter_at_exit:u32,
 /// Percentages of the entire unpadded route, including the forced running prefix.
 pub run_percent:f64,pub walk_percent:f64,
}
/// One run-to-walk switch. Segment lengths are converted separately, with
/// fractional walking updates rounded upward. Reserve three RUNNING updates
/// at the endpoint even when walking, so padding never assumes a cheaper pace.
fn movement(counter:u32,first_water:u32,run:u32,walk:u32)->Option<Movement> {
 if first_water>WATER_RUN {return None;}
 let water_left=WATER_RUN-first_water;
 for fast in (0..=water_left+FINAL_RUN).rev() {
  let water_run=fast.min(water_left);let land_run=fast.saturating_sub(water_left);
  let water_walk=((water_left-water_run)*WATER_WALK).div_ceil(WATER_RUN);
  let land_walk=((FINAL_RUN-land_run)*FINAL_WALK).div_ceil(FINAL_RUN);
  let cost=(water_run+land_run+END_LEEWAY)*run+(water_walk+land_walk)*walk;
  if cost>=counter {continue;}
  let run_distance=LAND_WALK as f64+(first_water+water_run)as f64*WATER_WALK as f64/WATER_RUN as f64+land_run as f64*FINAL_WALK as f64/FINAL_RUN as f64;
  let run_percent=100.*run_distance/(LAND_WALK+WATER_WALK+FINAL_WALK)as f64;
  return Some(Movement {water_run,water_walk,land_run,land_walk,leeway_updates:END_LEEWAY,counter_cost:cost,counter_at_exit:counter-cost,run_percent,walk_percent:100.-run_percent});
 }
 None
}
#[derive(Debug,Serialize)]pub struct Approach {
 pub water_entry:Entry,pub encounter:SouthernSai,pub escape_inputs:Vec<u8>,
 pub first_encounter_total_updates:u32,pub remaining_running_updates:u32,
 pub run_percent:f64,pub walk_percent:f64,pub gigantes_entry:Entry,
 pub ebony_eels:bool,pub movement:Movement,
}
/// Minimize escape boost cycles, then maximize running for that reset counter.
/// None rejects this upstream RNG candidate (ambush or adjustment budget).
/// The first formation is fixed before any battle boost inputs are considered.
pub fn escape_approach(entry:&Entry,max_cycles:u32)->Result<Option<Approach>> {escape_approach_with_defends(entry,max_cycles,0)}
fn escape_approach_with_defends(entry:&Entry,max_cycles:u32,defends:u8)->Result<Option<Approach>> {
 let w=&entry.world_after;
 ensure(w.map==MAP&&w.time==2&&w.evasive_maneuvers,"Nameless approach requires daytime island entry with Evasive Maneuvers")?;
 let mut l=Loader::new(world::catalog(),w.clone(),entry.after)?;
 l.state.enter_encounter_zone("ENT_DNG_OCN_1_3_LND")?;
 let options=crate::encounter::PredictionOptions {zone:"ENT_DNG_OCN_1_3_LND".into(),period:"DayTime".into(),ability_percent:50,dash:true,..Default::default()};
 let decrement=crate::encounter::catalog().counter_decrement(&options)?;
 let water_decrement=crate::encounter::catalog().counter_decrement(&crate::encounter::PredictionOptions {zone:"ENT_DNG_OCN_1_3_WTR".into(),..options.clone()})?;
 let walk_decrement=crate::encounter::catalog().counter_decrement(&crate::encounter::PredictionOptions {dash:false,..options})?;
 ensure(decrement==water_decrement,"Nameless land/water running rates differ")?;
 ensure(l.state.encounter_counter>LAND_TO_WATER*decrement,"Encounter would occur before water")?;
 l.state.encounter_counter-=LAND_TO_WATER*decrement;
 l.boundary("Run 100% for 70 decrements to water; canoe boarding retains the counter");
 l.state.enter_encounter_zone("ENT_DNG_OCN_1_3_WTR")?;
 let water_entry=point(&mut l,entry.after);
 let Some(mut encounter)=galdera_coast::sai_encounter_defends(water_entry.clone(),"ENT_DNG_OCN_1_3_WTR",defends)? else{return Ok(None);};
 let water_updates=encounter.encounter.counter_updates.unwrap();
 ensure(water_updates<=WATER_RUN,"First encounter falls outside the measured water crossing")?;
 let first=LAND_TO_WATER+water_updates;let remaining=TOTAL_RUN-first;
 let ebony_eels=encounter.encounter.enemies.len()==3&&encounter.encounter.enemies.iter().all(|e|e.enemy.id=="ENE_NML_WTR_350");
 for cycles in 0..=max_cycles {
  let inputs:Vec<_>=(0..cycles).flat_map(|_|[1,0]).collect();
  let escape=encounter.menu.clone().attempt(&inputs,encounter.encounter.encounter_level.unwrap())?;
  ensure(escape.success&&escape.chance_percent==100,"Nameless Escape must be guaranteed")?;
  let ret=crate::highroad::restore_field(&water_entry.world_after,&escape,MAP,"Nameless water Escape")?;
  let Some(movement)=movement(ret.world_after.encounter_counter,water_updates,decrement,walk_decrement) else {continue;};
  let mut l=Loader::new(world::catalog(),ret.world_after.clone(),ret.after)?;
  l.state.encounter_counter=movement.counter_at_exit;
  l.boundary(format!("Run first {:.2}% of the entire island route, then walk remaining {:.2}%; first encounter is on water; reserve 3 running decrements at Gigantes; no second encounter",movement.run_percent,movement.walk_percent));
  let gigantes_entry=point(&mut l,ret.after);
  encounter.escape=escape;encounter.field_return=Entry {arrival:ret.arrival,after:ret.after,world_after:ret.world_after,trace:ret.trace,decisions:vec![]};
  return Ok(Some(Approach {water_entry,encounter,escape_inputs:inputs,first_encounter_total_updates:first,remaining_running_updates:remaining,run_percent:movement.run_percent,walk_percent:movement.walk_percent,gigantes_entry,ebony_eels,movement}));
 }
 Ok(None)
}
#[cfg(test)]mod tests {
 use super::*;
 fn geared_entry(seed:u32)->Entry {
  let v:serde_json::Value=serde_json::from_str(include_str!("../data/source/galdera-flex-entry.json")).unwrap();
  let w:world::WorldState=serde_json::from_value(v["world_after"].clone()).unwrap();
  let mut l=Loader::new(world::catalog(),w,crate::CrtRng::new(seed)).unwrap();
  crate::gravell_setup::learn(&mut l.state,6,0,&["ABI_SET_MER_030"]).unwrap();
  l.state.character_progression.get_mut(&6).unwrap().jobs.learned_abilities.entry(6).or_default().insert("ABI_SET_SCH_060".into()); // legacy fixture predates explicit Scholar tracking
  crate::gravell_setup::jp_setup(&mut l).unwrap();
  // This old pre-flex fixture predates the new acquisitions; explicitly provide
  // their exact inventory for the continuation test, never in production.
  for (id,count) in [("ITM_EQP_SWD_380",1),("ITM_EQP_LNS_220",1),("ITM_EQP_HLM_180",1),("ITM_EQP_ACS_250",1),("ITM_EQP_SLD_240",1),("ITM_EQP_AMR_300",1),("ITM_CSM_0610",2),("ITM_CSM_0730",1),("ITM_CSM_0550",1)]{l.state.route_items.insert(id.into(),count);}
  l.visit_map(MAP).unwrap();let before=l.rng;point(&mut l,before)
 }
 #[test]fn purchase_equips_are_not_applied_twice_at_island_menu() {
  let e=geared_entry(0);let mut old=Loader::new(world::catalog(),e.world_after.clone(),e.after).unwrap();
  crate::nameless_setup::apply(&mut old).unwrap();
  let mut early=Loader::new(world::catalog(),e.world_after,e.after).unwrap();
  early.state.map="Twn_Sea_3_1_A".into();
  let speed=early.state.character_progression[&7].character.stats.speed;
  for item in ["ITM_EQP_SLD_240","ITM_EQP_AMR_300"]{crate::nameless_setup::equip_roque_purchase(&mut early,item).unwrap();}
  assert!(early.state.character_progression[&7].character.stats.speed>speed);
  assert_eq!(early.rng,e.after);
  early.state.map=MAP.into();crate::nameless_setup::apply(&mut early).unwrap();
  assert_eq!(serde_json::to_value(&early.state).unwrap(),serde_json::to_value(&old.state).unwrap());
  let once=serde_json::to_value(&early.state).unwrap();crate::nameless_setup::apply(&mut early).unwrap();
  assert_eq!(once,serde_json::to_value(&early.state).unwrap());assert_eq!(early.rng,e.after);
 }
 #[test]fn menu_is_transactional_idempotent_and_preserves_rng() {
  let entry=geared_entry(0);let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after).unwrap();
  let mut missing=l.state.clone();missing.route_items.remove("ITM_CSM_0610");
  let mut invalid=Loader::new(world::catalog(),missing,entry.after).unwrap();let old=serde_json::to_value(&invalid.state).unwrap();
  assert!(crate::nameless_setup::apply(&mut invalid).is_err());assert_eq!(serde_json::to_value(&invalid.state).unwrap(),old);
  crate::nameless_setup::apply(&mut l).unwrap();let once=serde_json::to_value(&l.state).unwrap();
  crate::nameless_setup::apply(&mut l).unwrap();assert_eq!(serde_json::to_value(&l.state).unwrap(),once);assert_eq!(l.rng,entry.after);
  let o=&l.state.character_progression[&6].character;let c=o.combat.as_ref().unwrap();
  assert_eq!(c.magic+c.weapons.iter().map(|w|w.magic).max().unwrap(),797);
  assert_eq!(c.nuts["MATK"],56);assert_eq!(c.fire_damage_bonus,60);
  assert!(!l.state.route_items.contains_key("ITM_CSM_0610"));
 }
 #[test]fn jp_branch_carries_rewards_pom_and_safe_remainder_without_boss() {
  for seed in 0u32..256 {
   let mut e=geared_entry(seed.wrapping_mul(2654435761));e.world_after.route_items.remove("ITM_CSM_0070");
   let Some(t)=jp_finish(&e,64).unwrap()else{continue;};
   assert_eq!(t.rewards.base.jp,111);assert_eq!(t.rewards.awarded.jp,12210);assert_eq!(t.rewards.hired_help_cost,0);
   assert!(!t.field_return.world_after.route_items.contains_key("ITM_CSM_0070"));
   assert_eq!(t.fight.actions.last().unwrap().action,ot2_battle_engine::gravell_routine::Action::Fireball);
   assert!(t.movement.counter_at_exit>0);assert_eq!(t.field_return.after,t.gigantes_entry.after);
   assert!(!t.gigantes_entry.world_after.flag(40003));assert_eq!(t.gigantes_entry.world_after.time,2);
   for id in [7,6,5,4] {assert_eq!(t.field_return.world_after.character_progression[&id].jp_earned,t.setup.world_after.character_progression[&id].jp_earned+12210);}
   return;
  }panic!("Expected reachable Nameless JP candidate");
 }
 #[test]fn complete_inactive_chain_carries_boss_and_cutscene_state() {
  let mut accepted=0;
  let mut adjusted=0;
  for seed in 0..16 {
   let entry=geared_entry(seed);let Some(t)=finish(&entry,1000).unwrap()else{continue;};accepted+=1;
   if t.finisher_boost_cycles>0 {
    adjusted+=1;
    let players:Vec<_>=t.prompt.world_after.party.iter().map(|id|t.prompt.world_after.character_progression[id].character.clone()).collect();
    let base=ot2_battle_engine::gigantes::predict(t.prompt.after,&players,&t.prompt.world_after.enemy_knowledge).unwrap();
    for (a,b) in base.actions.iter().zip(&t.battle.actions).take(base.actions.len()-1) {assert_eq!(a.report.rng_after,b.report.rng_after);}
   }
   assert!(t.victory.world_after.flag(40003));assert!(t.battle.broken);
   if accepted==1 {eprintln!("Gigantes conservative floor: {:?}",t.damage_floor);}
   assert_eq!(t.night.world_after.time,4);assert_eq!(t.battle.before,t.prompt.after);
   assert_eq!(t.prompt.trace.iter().filter(|s|s.kind==world::DrawKind::BattleTiming).count(),4);
   assert!(!t.battle.trace.iter().any(|s|s.reason=="actor_initialization"&&s.actor.is_some_and(|a|a.0<4)),"Core must not initialize the same players twice");
   assert_eq!(t.battle.actions.last().unwrap().report.outcome,ot2_battle_engine::Outcome::Victory);
   assert!(t.victory.world_after.character_progression[&6].character.combat.as_ref().unwrap().nuts.contains_key("MATK"));
   let end=t.loot.end();assert_eq!(end.world_after.time,2);
   for item in ["ITM_EQP_AMR_270","ITM_INF_SS_TM21_0400_0100","ITM_EQP_ACS_940"] {assert_eq!(end.world_after.route_items[item],1);}
   assert!(end.world_after.opened_route_chests.contains("Treasure_Dng_Ocn_1_3_04"));
   assert_eq!(t.loot.stages[3].1.arrival,t.loot.stages[3].1.after); // fixed chest
  }
  assert!(accepted>0);
  assert!(adjusted>0,"Exercise actual Partitio finisher adjustment");
 }
 #[test]fn geared_fireball_ebony_eel_damage_audit() {
  use ot2_battle_engine::combat::{self,Attack,DamageInput,DamageKind,Element};
  let e=geared_entry(0);let mut l=Loader::new(world::catalog(),e.world_after,e.after).unwrap();crate::nameless_setup::apply(&mut l).unwrap();
  let o=l.state.character_progression[&6].character.combat.as_ref().unwrap();
  let attack=(o.magic+o.weapons.iter().map(|w|w.magic).max().unwrap()).min(999);
  for night in [false,true] {for (bp,ratio) in [100,190,280,370].into_iter().enumerate() {
   let skill=Attack{kind:DamageKind::Magical,weapon:None,element:Some(Element::Fire),invocation:105,ability_ratio:ratio,hit_bonus:0,critical_bonus:0,critical_power:125,normal_attack:false};
   let input=DamageInput{attack,defense:231,growth:o.growth_magic,guard:false,weapon_weak:false,element_weak:false,broken:false,critical:false,attack_up:night,defense_down:night,defense_up:false};
   let min=combat::damage_with_equipment(input,&skill,1.01,false,false,o.fire_damage_bonus);
   let max=combat::damage_with_equipment(input,&skill,1.05,false,false,o.fire_damage_bonus);
   assert_eq!(min>=2367,night||bp>=1);
   eprintln!("Fireball BP{bp}, night {night}, MATK{attack}, growth{}: {min}..{max}, eel HP2367",o.growth_magic);
  }}
 }
 #[test]fn georges_rejects_failed_scrutinize_and_keeps_input_unchanged() {
  let mut entry=geared_entry(0);entry.world_after.set_flag(40003,true);entry.world_after.time=4;
  entry.world_after.character_progression.get_mut(&6).unwrap().level=70;
  let mut passed=0;let mut rejected=0;
  for seed in 0..128 {
   entry.after=crate::CrtRng::new(seed);
   let old=serde_json::to_value(&entry.world_after).unwrap();
   match post_gigantes(&entry).unwrap() {
    Some(s)=>{passed+=1;assert_eq!(s.stages.len(),4);
     let scrutinize=&s.stages[2].1;
     let draws:Vec<_>=scrutinize.trace.iter().filter(|t|t.value.is_some()).collect();
     assert_eq!(draws.len(),2); // same open menu: success check, before-dialogue lottery
     assert_eq!(draws[0].kind,world::DrawKind::ScrutinizeCheck);
     assert!(draws[0].value.unwrap()<72);
     assert!(post_gigantes(s.end()).is_err());
    },None=>rejected+=1
   }
   assert_eq!(serde_json::to_value(&entry.world_after).unwrap(),old);
  }
  assert!(passed>0&&rejected>0);
 }
 #[test]fn captured_georges_continuation_uses_one_menu_for_steal_and_scrutinize() {
  let entry=geared_entry(0);
  let mut l=Loader::new(world::catalog(),entry.world_after,entry.after).unwrap();
  l.manual_time_switch(4,"Gigantes night").unwrap();
  l.skipped_event("EV_Dng_Ocn_1_3_0020").unwrap();
  l.state.character_progression.get_mut(&6).unwrap().level=70;
  l.rng=crate::CrtRng{state:0x98d520a9,draws:9963};
  let rng=l.rng;let entry=point(&mut l,rng);
  let s=post_gigantes(&entry).unwrap().expect("Captured no-adjustment finisher must pass with direct path-action switch");
  let checks:Vec<_>=s.stages.iter().flat_map(|(_,e)|&e.trace).filter(|t|t.value.is_some()).collect();
  assert_eq!(checks.iter().filter(|t|t.kind==world::DrawKind::PathActionMenu).count(),1);
  let check=checks.iter().find(|t|t.kind==world::DrawKind::ScrutinizeCheck).unwrap();
  assert_eq!(check.before.draws,9967);
  assert_eq!(check.value,Some(25)); // Native 1-based roll26 succeeds at72%.
  assert_eq!(s.end().after.draws,9969);
  let mut extra_cycle=entry.clone();extra_cycle.after.advance(1);
  assert!(post_gigantes(&extra_cycle).unwrap().is_none()); // Roll91 after the formerly selected cycle.
 }
 #[test]fn movement_padding_strict_thresholds_and_mixed_terrain() {
  // Earliest fight: two water updates. Full run costs 246*7 +21.
  let all=movement(1744,2,7,5).unwrap();
  assert_eq!((all.water_run,all.land_run,all.counter_cost),(65,181,1743));
  assert_eq!(all.run_percent,100.);assert_eq!(all.counter_at_exit,1);
  assert!(movement(1743,2,7,5).unwrap().walk_percent>0.);
  // All walking costs ceil(65*69/67)*5 +191*5 +21 =1311.
  assert!(movement(1311,2,7,5).is_none());
  let slow=movement(1312,2,7,5).unwrap();assert_eq!(slow.counter_at_exit,1);
  assert_eq!((slow.water_run,slow.land_run),(0,0));
  assert!(slow.run_percent>21.9); // includes forced land + first water prefix
  for first in 2..=59 {for counter in 1000..=1798 {
   if let Some(m)=movement(counter,first,7,5) {
    assert!(m.counter_cost<counter);
    assert_eq!(m.counter_cost,(m.water_run+m.land_run+3)*7+(m.water_walk+m.land_walk)*5);
    assert!(m.water_walk==0||m.land_run==0);
    assert!((m.run_percent+m.walk_percent-100.).abs()<1e-9);
   }
  }}
 }
 #[test]fn first_water_escape_preserves_positive_counter_to_gigantes() {
  let v:serde_json::Value=serde_json::from_str(include_str!("../data/source/galdera-flex-entry.json")).unwrap();
  let w:world::WorldState=serde_json::from_value(v["world_after"].clone()).unwrap();
  let mut count=0;
  for seed in 0..16 {
   let mut l=Loader::new(world::catalog(),w.clone(),crate::CrtRng::new(seed)).unwrap();
   l.visit_map(MAP).unwrap();let before=l.rng;let e=point(&mut l,before);
   let Some(r)=escape_approach(&e,1000).unwrap() else{continue;};count+=1;
   assert!((72..=129).contains(&r.first_encounter_total_updates));
   assert_eq!(r.first_encounter_total_updates+r.remaining_running_updates,TOTAL_RUN);
   assert!(r.gigantes_entry.world_after.encounter_counter>0);
   assert_eq!(r.encounter.field_return.world_after.encounter_counter-r.gigantes_entry.world_after.encounter_counter,r.movement.counter_cost);
   assert_eq!(r.gigantes_entry.after,r.encounter.field_return.after);
   assert!((0.0..=100.0).contains(&r.run_percent));
   assert_eq!(r.movement.leeway_updates,3);
  }
  assert!(count>0);
 }
}

