//! Return to New Delsta, one daytime Highroad encounter, then Arkar.
use crate::{Result,CrtRng,ensure,SimError,world::{self,Loader},snows_escape::Entry,
 encounter::{self,Boundary,PredictionOptions},manipulation::{BoostInputs,describe_inputs}};
use ot2_battle_engine::{escape::Situation,field_escape::{FieldEscapeMenu,FieldEscapeAttempt}};
use serde::Serialize;
#[derive(Debug,Serialize)]
pub struct Approach {pub town:Entry,pub agnea_story:Entry,pub highroad:Entry,pub encounter:encounter::Prediction,pub battle_entry:CrtRng,pub surprise_percent:u32}
#[derive(Debug,Serialize)]
pub struct ClockbankAdjustment {pub switches:u32,pub target:CrtRng,pub departure:Entry,pub input_cost:u32}
#[derive(Debug,Serialize)]
pub struct Prediction {pub clockbank_adjustment:Option<ClockbankAdjustment>,pub approach:Approach,pub menu:FieldEscapeMenu,pub inputs:BoostInputs,pub escape:FieldEscapeAttempt,pub field_return:Entry,pub license:Entry,pub bomb:Entry,pub scope:Entry,pub western_clockbank:Entry,pub southern_clockbank:Entry,pub clockbank_reach:Option<crate::winter_voyage::CropdaleReach>,pub clockbank:Entry,pub partitio_story:Entry,pub clockite_stages:Vec<(String,Entry)>,pub crackridge_return:HarborReturn}
#[derive(Debug,Serialize)]
pub struct HarborReturn {pub defends:Vec<ot2_battle_engine::StepReport>,pub arrival:Entry,pub encounter:encounter::Prediction,pub battle_entry:CrtRng,pub menu:FieldEscapeMenu,pub inputs:BoostInputs,pub escape:Option<FieldEscapeAttempt>,pub victory:Option<ot2_battle_engine::field_escape::FieldRareVictory>,pub field_return:Entry,pub southern:Option<Box<HarborReturn>>,pub approach_stages:Vec<(String,Entry)>,pub gravell:Option<GravellArrival>}
#[derive(Debug,Serialize)]
pub struct GravellArrival {pub license:Entry,pub setup:Entry,pub arrival:Entry,pub night:Entry,pub encounter:encounter::Prediction,pub battle_entry:CrtRng,pub fight:ot2_battle_engine::gravell_routine::Fight,pub rewards:ot2_battle_engine::gravell_routine::Rewards,pub field_return:Entry}
fn field_encounter(entry:&Entry,zone:&str)->Result<(encounter::Prediction,CrtRng)> {
 let w=&entry.world_after;let mut rng=entry.after;
 let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions {map:w.map.clone(),zone:zone.into(),period:if w.time==2 {"DayTime"}else{"Night"}.into(),progress:w.main_progress_points,first_encounter:w.first_encounter,boundary:Boundary::CurrentCounter,current_counter:Some(w.encounter_counter),party_levels:w.party.iter().map(|id|w.character_progression[id].level).collect(),ability_percent:if w.evasive_maneuvers {50}else{100},dash:true,..Default::default()})?;
 Ok((encounter,rng))
}
fn gravell_arrival(entry:&Entry)->Result<Option<GravellArrival>> {gravell_target(entry,None)}
type FightCache=std::cell::RefCell<std::collections::BTreeMap<(u32,Vec<u8>),(u32,Option<ot2_battle_engine::gravell_routine::Fight>)>>;
#[derive(Clone,Copy)]
struct BgTarget<'a> {rng:CrtRng,budget:u32,cache:&'a FightCache,guard_extra:usize,fight_cap:usize,action_cap:u32,work:Option<&'a std::cell::Cell<usize>>}
fn gravell_target(entry:&Entry,target:Option<BgTarget>)->Result<Option<GravellArrival>> {
 if target.is_some_and(|t|t.work.is_some_and(|w|w.get()==0)){return Ok(None);}
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 // GuildData GUILD_003: first license has no item/money/ability requirement.
 l.skipped_event("SYS_MER_GUILD_0010")?;l.skipped_event("SYS_MER_GUILD_0030")?;
 l.state.job_licenses.entry(0).or_insert(1);l.state.set_flag(100,true);
 l.dismiss_tutorial(59)?;l.boundary("Merchant License acquired");
 let license=point(&mut l,entry.after);
 crate::gravell_setup::apply(&mut l)?;let setup=crate::gravell_setup::snapshot(&mut l,license.after);
 l.visit_map("Fld_Wld_3_2")?;let arrival=point(&mut l,license.after);
 l.manual_time_switch(4,"Western Gravell Wilds: switch to night")?;
 let night=point(&mut l,arrival.after);
 let (encounter,rng)=field_encounter(&night,"ENT_FLD_WLD_3_2_LND")?;
 if !matches!(encounter.situation.as_deref(),Some("Normal"|"Preemptive"))||!encounter.enemies.iter().any(|e|e.enemy.id=="ENE_NML_OTR_150") {return Ok(None);}

 if target.is_some_and(|t|rng.draws>=t.rng.draws) {return Ok(None);}
 let situation=if encounter.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal};
 let players:Vec<_>=night.world_after.party.iter().map(|id|night.world_after.character_progression[id].character.clone()).collect();
 let ids:Vec<_>=encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
 let key=(rng.state,serde_json::to_vec(&(situation,&players,&ids,&night.world_after.enemy_knowledge)).map_err(|e|SimError(e.to_string()))?);
 let cached=target.and_then(|t|t.cache.borrow().get(&key).filter(|(budget,fight)|*budget>=t.budget||fight.is_some()).cloned());
 let fight=if let Some((_,fight))=cached {
  fight.filter(|f|fight_input_cost(f)<=target.unwrap().budget)
 }else {
  let battle=ot2_battle_engine::field_escape::carried_step_ahead(rng,situation,&players,&ids,true,&night.world_after.enemy_knowledge)?;
  let fight=if let Some(t)=target {
   ot2_battle_engine::gravell_routine::search_target_windowed(battle,(t.budget as usize).min(t.fight_cap),t.rng.state,t.guard_extra,t.work)?
  }else{ot2_battle_engine::gravell_routine::search(battle,usize::MAX)?};
  if let Some(t)=target {if !t.work.is_some_and(|w|w.get()==0) {t.cache.borrow_mut().insert(key,(t.budget,fight.clone()));}}
  fight
 };
 let Some(mut fight)=fight else {return Ok(None);};
 let mut rewards=ot2_battle_engine::gravell_routine::rewards(&fight,night.world_after.party.iter().any(|id|night.world_after.character_progression[id].jobs.equipped_support.contains(crate::gravell_setup::MONEY_UP)))?;
 // The battle core starts a relative draw ledger; the route retains its absolute origin.
 for action in &mut fight.actions {action.before.draws+=rng.draws;action.report.rng_before.draws+=rng.draws;action.report.rng_after.draws+=rng.draws;}
 for span in fight.trace.iter_mut().chain(rewards.trace.iter_mut()) {span.before.draws+=rng.draws;span.after.draws+=rng.draws;}
 fight.after.draws+=rng.draws;fight.state.rng.draws+=rng.draws;rewards.after.draws+=rng.draws;
 if let Some(trace)=fight.state.opening_crt.as_mut().and_then(|c|c.trace.as_mut()) {for span in trace {span.before.draws+=rng.draws;span.after.draws+=rng.draws;}}
 let mut state=night.world_after.clone();state.enemy_knowledge=fight.enemy_knowledge.clone();
 for (&id,actor) in state.party.iter().zip(&fight.state.actors) {state.character_progression.get_mut(&id).unwrap().character.resources=Some(actor.resources.clone());}
 state.award_victory(rewards.awarded)?;state.first_encounter=false;
 let mut l=Loader::new(world::catalog(),state,rewards.after)?;
 l.restore_field_after_battle("Western Gravell EXP ×100 victory")?;
 let field_return=point(&mut l,rewards.after);
 Ok(Some(GravellArrival {license,setup,arrival,night,encounter,battle_entry:rng,fight,rewards,field_return}))
}
fn field_continuation(entry:&Entry,stage:usize,target:Option<BgTarget>)->Result<Option<HarborReturn>> {
 if target.is_some_and(|t|entry.after.draws>=t.rng.draws||t.budget<6||t.work.is_some_and(|w|w.get()==0)){return Ok(None);}
 let (map,zone,label)=[("Fld_Wld_1_2","ENT_FLD_WLD_1_2_LND","Crackridge Harbor return"),("Fld_Wld_2_2","ENT_FLD_WLD_2_2_LND","Southern Crackridge Wilds"),("Fld_Wld_2_1","ENT_FLD_WLD_2_1_LND","Western Crackridge Wilds first visit"),("Fld_Wld_2_1","ENT_FLD_WLD_2_1_LND","Western Crackridge Wilds after town")][stage];
 // Fast travel is user-authorized to the Harbor tagged earlier in this route.
 // Older continuation fixtures do not retain the complete visited-map ledger.
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 let mut approach_stages=vec![];
 if stage==3 {l.visit_map("Twn_Wld_2_1_A")?;approach_stages.push(("Crackridge · loaded".into(),point(&mut l,entry.after)));}
 let before=l.rng;l.visit_map(map)?;let arrival=point(&mut l,before);
 let w=&arrival.world_after;
 let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
 let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
 let mut rng=arrival.after;
 let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions {map:map.into(),zone:zone.into(),period:if w.time==2 {"DayTime"}else{"Night"}.into(),progress:w.main_progress_points,first_encounter:w.first_encounter,boundary:Boundary::CurrentCounter,current_counter:Some(w.encounter_counter),party_levels:levels.clone(),ability_percent:if w.evasive_maneuvers {50}else{100},dash:true,..Default::default()})?;
 let situation=match encounter.situation.as_deref(){Some("Normal")=>Situation::Normal,Some("Preemptive")=>Situation::Preemptive,_=>return Ok(None)};
 let ids:Vec<_>=encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
 let step_ahead=w.party.iter().all(|id|w.character_progression[id].jobs.equipped_support.contains(crate::gravell_setup::STEP_AHEAD));
 let menu=if step_ahead {FieldEscapeMenu::with_step_ahead(rng,situation,&players,&levels,&ids,&w.enemy_knowledge)?}else{FieldEscapeMenu::with_players(rng,situation,&players,&levels,&ids,&w.enemy_knowledge)?};
 let mut menus=vec![(menu.clone(),vec![])];
 if target.is_some() {for _ in 0..3 {
  let (m,actions)=menus.last().unwrap();
  let Some((next,report))=m.safe_step_ahead_defend()? else {break};
  let mut actions=actions.clone();actions.push(report);menus.push((next,actions));
 }}
 let mut escape_cursors:Vec<_>=menus.iter().map(|(m,_)|(m.clone(),0u32)).collect();
 let mut best:Option<HarborReturn>=None;
 for cost in 0..=target.map_or(u32::MAX/2,|t|t.budget.min(t.action_cap)) {
 if target.is_some_and(|t|t.work.is_some_and(|w|w.get()==0)){return Ok(best);}
 for (menu_index,(menu,defends)) in menus.iter().enumerate() {
 for kill in [false,true] {
  if kill&&(stage!=3||!defends.is_empty()) {continue;}
  let action_cost=if kill {10}else{0}+defends.len() as u32*crate::escape_search::DEFEND_INPUT_COST;
  if cost<action_cost||(cost-action_cost)%2!=0 {continue;}
  let n=(cost-action_cost)/2;
  if let Some(work)=target.and_then(|t|t.work) {
   let units=if kill {5}else{1};
   if work.get()<units {work.set(0);return Ok(best);}
   work.set(work.get()-units);
  }
  let inputs=describe_inputs(n,0,crate::conning::path(n))?;
  let (escape,victory,field_return)=if kill {
   let Some(v)=ot2_battle_engine::field_escape::carried_soulstone_with_step_ahead(rng,situation,&players,&levels,&ids,w.time!=2,&w.enemy_knowledge,&inputs.levels,"wind_soulstone_l",step_ahead)? else {continue;};
   let mut state=w.clone();crate::conning::apply_item_victory(&mut state,&v)?;state.first_encounter=false;
   let mut l=Loader::new(world::catalog(),state,v.after)?;l.restore_field_after_battle("Western Crackridge Wind Soulstone L victory")?;
   let ret=point(&mut l,v.after);(None,Some(v),ret)
  }else{
   let (cursor,cycles)=&mut escape_cursors[menu_index];
   while *cycles<n {cursor.boost_cancel_cycle();*cycles+=1;}
   let e=cursor.clone().attempt(&[],encounter.encounter_level.unwrap())?;
   if !e.success {continue;}
   let ret=crate::highroad::restore_field(w,&e,map,&format!("{label} Escape"))?;
   (Some(e),None,Entry {arrival:ret.arrival,after:ret.after,world_after:ret.world_after,trace:ret.trace,decisions:vec![]})
  };
  let southern=if stage<3 {
   // Freeze this control at its first safe next opening. Do not recursively
   // reopen it to rescue a distant BG target.
   if target.is_some() {
    let mut probe=Loader::new(world::catalog(),field_return.world_after.clone(),field_return.after)?;
    if stage==2 {probe.visit_map("Twn_Wld_2_1_A")?;}
    let (map,zone)=if stage==0 {("Fld_Wld_2_2","ENT_FLD_WLD_2_2_LND")}else{("Fld_Wld_2_1","ENT_FLD_WLD_2_1_LND")};
    probe.visit_map(map)?;let check=point(&mut probe,field_return.after);
    if !matches!(field_encounter(&check,zone)?.0.situation.as_deref(),Some("Normal"|"Preemptive")) {continue;}
   }
   let Some(south)=field_continuation(&field_return,stage+1,target.map(|t|BgTarget {budget:t.budget-cost,..t}))? else {if target.is_some(){return Ok(None);}continue;};
   Some(Box::new(south))
  } else {None};
  let gravell=if stage==3 {let Some(g)=gravell_target(&field_return,target.map(|t|BgTarget {budget:best.as_ref().map_or(t.budget,|b|t.budget.min(route_cost(b).saturating_sub(1))).saturating_sub(cost),..t}))? else {continue;};Some(g)}else{None};
  let candidate=HarborReturn {defends:defends.clone(),arrival:arrival.clone(),encounter:encounter.clone(),battle_entry:rng,menu:menu.clone(),inputs,escape,victory,field_return,southern,approach_stages:approach_stages.clone(),gravell};
  if stage<3 {return Ok(Some(candidate));}
  if best.as_ref().is_none_or(|b|route_cost(&candidate)<route_cost(b)) {best=Some(candidate);}
 }
 }
 if best.as_ref().is_some_and(|b|cost>=route_cost(b)) {break;}
 }
 if best.is_some()||target.is_some(){return Ok(best);}
 Err(SimError("Crackridge return Escape search exhausted".into()))
}
fn point(l:&mut Loader<'_>,arrival:CrtRng)->Entry {Entry {arrival,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}}
/// Start fast on water, then switch once to slow for the remaining route.
/// Estimate terrain distance shares from unpadded slow-update counts (68:49).
/// Raw measurements: slow 68 water + 49 land; fast 66 water + 44 land.
/// Reserve six additional land updates in either mode.
fn clockbank_reach(state:&world::WorldState)->Result<Option<crate::winter_voyage::CropdaleReach>> {
 let mut walking_cost=0;let mut running_cost=0;let mut water_extra=0;
 for (zone,slow,fast) in [("ENT_FLD_CTY_2_1_WTR",68,66),("ENT_FLD_CTY_2_1_LND",55,50)] {
  let mut options=PredictionOptions {zone:zone.into(),period:if state.time==2 {"DayTime"}else{"Night"}.into(),ability_percent:if state.evasive_maneuvers {50}else{100},dash:false,..Default::default()};
  let slow_cost=slow*encounter::catalog().counter_decrement(&options)?;
  walking_cost+=slow_cost;
  options.dash=true;
  let fast_cost=fast*encounter::catalog().counter_decrement(&options)?;
  running_cost+=fast_cost;
  if zone.ends_with("WTR") {water_extra=fast_cost-slow_cost;}
 }
 let counter=state.encounter_counter;
 if counter<=walking_cost {return Ok(None);}
 let water_share=68.0/117.0;
 let land_extra=running_cost-walking_cost-water_extra;
 let budget=(counter-1-walking_cost) as f64;
 let limit=if counter>running_cost {1.0}else if budget<=water_extra as f64 {
  water_share*budget/water_extra as f64
 }else{water_share+(1.0-water_share)*(budget-water_extra as f64)/land_extra as f64};
 let fraction=(limit*1000.0).floor()/1000.0;
 let water_fast=(fraction/water_share).min(1.0);
 let land_fast=((fraction-water_share)/(1.0-water_share)).max(0.0);
 let consumed=(walking_cost as f64+water_extra as f64*water_fast+land_extra as f64*land_fast).ceil() as u32;
 Ok(Some(crate::winter_voyage::CropdaleReach {initial_counter:counter,walking_updates:123,running_updates:116,walking_cost,running_cost,run_percent:fraction*100.0,estimated_counter_at_exit:counter-consumed}))
}
pub fn approach(entry:&Entry)->Result<Approach> {
 ensure(entry.world_after.time==2,"Inventor route requires daytime")?;
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 l.visit_map("Twn_Cty_1_1_A")?;let town=point(&mut l,entry.after);
 l.skipped_event_with_choices("MapInStorySelect",&world::EventChoices::from([(("MapInStorySelect".into(),1),"MS_ODO_01".into())]))?;
 ensure(l.state.active_scenarios.contains(&801) && l.state.flag(6550),"Agnea Chapter 2 opening did not finish")?;
 let agnea_story=point(&mut l,town.after);
 l.visit_map("Fld_Cty_1_3")?;let highroad=point(&mut l,agnea_story.after);
 let levels:Vec<_>=l.state.party.iter().map(|id|l.state.character_progression[id].level).collect();
 let mut rng=l.rng;
 let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions {map:l.state.map.clone(),zone:"ENT_FLD_CTY_1_3_LND".into(),period:"DayTime".into(),progress:l.state.main_progress_points,first_encounter:l.state.first_encounter,boundary:Boundary::CurrentCounter,current_counter:Some(l.state.encounter_counter),party_levels:levels.clone(),ability_percent:if l.state.evasive_maneuvers {50}else{100},dash:true,..Default::default()})?;
 let enemy=encounter.enemies.iter().map(|e|e.enemy.level as i32).sum::<i32>()/encounter.enemies.len() as i32;
 let party=levels.iter().sum::<u32>() as i32/levels.len() as i32;
 Ok(Approach {town,agnea_story,highroad,encounter,battle_entry:rng,surprise_percent:(5+4*(enemy-party).clamp(-1,6)) as u32})
}
pub fn safe(a:&Approach)->bool {matches!(a.encounter.situation.as_deref(),Some("Normal"|"Preemptive"))}
pub fn finish(a:Approach)->Result<Prediction> {
 ensure(safe(&a),"Inventor Highroad encounter must not surprise the party")?;
 let w=&a.highroad.world_after;
 let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
 let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
 let ids:Vec<_>=a.encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
 let menu=FieldEscapeMenu::with_players(a.battle_entry,if a.encounter.situation.as_deref()==Some("Preemptive"){Situation::Preemptive}else{Situation::Normal},&players,&levels,&ids,&w.enemy_knowledge)?;
 for n in 0..u32::MAX/2 {
  let inputs=describe_inputs(n,0,crate::conning::path(n))?;
  let escape=menu.clone().attempt(&inputs.levels,a.encounter.encounter_level.unwrap())?;
  if !escape.success {continue;}
  let ret=crate::highroad::restore_field(w,&escape,"Fld_Cty_1_3","Inventor Highroad Escape")?;
  let field_return=Entry {arrival:ret.arrival,after:ret.after,world_after:ret.world_after,trace:ret.trace,decisions:vec![]};
  let mut l=Loader::new(world::catalog(),field_return.world_after.clone(),field_return.after)?;
  l.skipped_event("SYS_INV_GUILD_0010")?;let license=point(&mut l,field_return.after);
  l.skipped_event("SYS_INV_GUILD_0D10")?;let bomb=point(&mut l,license.after);
  l.skipped_event("SYS_INV_GUILD_0C10")?;crate::gravell_setup::equip_step_ahead(&mut l)?;let scope=point(&mut l,bomb.after);
  l.visit_map("Fld_Cty_1_4")?;let western_clockbank=point(&mut l,scope.after);
  l.visit_map("Fld_Cty_2_1")?;let southern_clockbank=point(&mut l,western_clockbank.after);
  let clockbank_reach=clockbank_reach(&southern_clockbank.world_after)?;
  // Escape and onward walking safety are joint required outcomes at this menu.
  // Retry from the original menu; rejected candidates never alter route state.
  let Some(reach)=clockbank_reach.as_ref() else {continue;};
  l.state.encounter_counter=reach.estimated_counter_at_exit;
  l.visit_map("Twn_Cty_2_1_A")?;let clockbank=point(&mut l,southern_clockbank.after);
  l.skipped_event_with_choices("MapInStorySelect",&world::EventChoices::from([(("MapInStorySelect".into(),1),"MS_SHO_01".into())]))?;
  ensure(l.state.active_scenarios.contains(&401) && l.state.flag(6076),"Partitio Chapter 2 opening did not finish")?;
  let partitio_story=point(&mut l,clockbank.after);
  let mut clockite_stages=Vec::new();
  l.visit_map("Twn_Cty_2_1_B")?;let stage=point(&mut l,partitio_story.after);
  clockite_stages.push(("Clockbank: Industrial District · loaded".into(),stage));
  for (event,label) in [("MS_SHO_20_0100","Will and Ori scenes skipped"),("MS_SHO_20_0200","Factory / Roque scenes skipped"),("MS_SHO_20_0400","Headquarters visit and follow-up finished"),("MS_SHO_20_0500","Floyd's request · returned to Industrial District"),("MS_SHO_20_0610","Weaponsmith · Clockite x5 received, popup dismissed")] {
   let before=l.rng;l.skipped_event(event)?;
   clockite_stages.push((label.into(),point(&mut l,before)));
  }
  let before=l.rng;l.visit_map("Dng_Cty_2_1")?;
  clockite_stages.push(("Roque Company: Factory · entered to deliver Clockite".into(),point(&mut l,before)));
  let before=l.rng;
  l.skipped_event_with_choices("MS_SHO_20_0615",&world::EventChoices::from([(("MS_SHO_20_0615".into(),2),"eDIALOG_YES".into())]))?;
  ensure(l.state.flag(6059)&&l.state.flag(6077)&&l.state.map=="Twn_Cty_2_1_B","Clockite delivery did not finish")?;
  clockite_stages.push(("Clockite delivered to Floyd · scene skipped, back outside".into(),point(&mut l,before)));
  let (clockbank_adjustment,crackridge_return)=search_clockbank_bg(&clockite_stages.last().unwrap().1)?;
  let clockbank_adjustment=Some(clockbank_adjustment);
  return Ok(Prediction {clockbank_adjustment,approach:a,menu,inputs,escape,field_return,license,bomb,scope,western_clockbank,southern_clockbank,clockbank_reach,clockbank,partitio_story,clockite_stages,crackridge_return});
 }
 Err(SimError("Highroad Escape search exhausted".into()))
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn clockbank_mixed_terrain_and_land_padding() {
  let f:serde_json::Value=serde_json::from_str(include_str!("../data/source/retail-inventor-highroad-20261010.json")).unwrap();
  let mut w:world::WorldState=serde_json::from_value(f["world"].clone()).unwrap();
  w.time=2;w.evasive_maneuvers=true;w.encounter_counter=584;
  let r=clockbank_reach(&w).unwrap().unwrap();
  assert_eq!((r.walking_cost,r.running_cost),(547,680));
  assert_eq!((r.run_percent,r.estimated_counter_at_exit),(36.0,1));
  w.encounter_counter=547;assert!(clockbank_reach(&w).unwrap().is_none());
  w.encounter_counter=680;assert!(clockbank_reach(&w).unwrap().unwrap().run_percent<100.0);
  w.encounter_counter=681;assert_eq!(clockbank_reach(&w).unwrap().unwrap().run_percent,100.0);
 }
 #[test]
 fn retail_first_menu_after_corrected_cait_reward() {
  let f:serde_json::Value=serde_json::from_str(include_str!("../data/source/retail-inventor-highroad-20261010.json")).unwrap();
  let mut w:world::WorldState=serde_json::from_value(f["world"].clone()).unwrap();w.sync_progression().unwrap();
  for id in w.party.clone() {let p=&mut w.character_progression.get_mut(&id).unwrap().character;let r=p.resources.as_mut().unwrap();r.hp=p.stats.max_hp;r.sp=p.stats.max_sp;}
  assert_eq!((w.character_progression[&8].level,w.character_progression[&8].character.stats.speed),(14,133));
  let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
  let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
  let ids:Vec<_>=f["enemies"].as_array().unwrap().iter().map(|v|v.as_str().unwrap()).collect();
  let menu=FieldEscapeMenu::with_players(serde_json::from_value(f["battle_entry"].clone()).unwrap(),Situation::Normal,&players,&levels,&ids,&w.enemy_knowledge).unwrap();
  assert_eq!(menu.current_order,vec![0,2,3,6,1,4,5]);
  assert_eq!(menu.next_order,vec![3,2,0,1,6,4,5]);
  assert_eq!(menu.rng.draws+19284735,19287621);
 }
}

#[cfg(test)]
mod gravell_integration_tests {
 use super::*;
 #[test]
 fn full_gravell_continuation_carries_reward_growth_and_absolute_rng() {
  let fixture:serde_json::Value=serde_json::from_str(include_str!("../data/source/gravell-complete-route-fixture.json")).unwrap();
  let b=&fixture["before"];
  let entry=Entry {arrival:serde_json::from_value(b["arrival"].clone()).unwrap(),after:serde_json::from_value(b["after"].clone()).unwrap(),world_after:serde_json::from_value(b["world_after"].clone()).unwrap(),trace:vec![],decisions:vec![]};
  let g=gravell_arrival(&entry).unwrap().unwrap();
  assert_eq!(g.fight.evaluation,ot2_battle_engine::gravell_routine::Evaluation::Accepted);
  assert_eq!(serde_json::to_value(g.field_return.after).unwrap(),fixture["expected_after"]);
  assert_eq!(serde_json::to_value(g.rewards.awarded).unwrap(),fixture["expected_rewards"]);
  for id in &g.field_return.world_after.party {
   let before=&g.night.world_after.character_progression[id];let after=&g.field_return.world_after.character_progression[id];
   assert_eq!(after.experience,before.experience+g.rewards.awarded.exp);
   assert_eq!(after.jp_earned,before.jp_earned+g.rewards.awarded.jp);
   assert!(after.level>before.level);
   assert_eq!(after.character.resources.as_ref().unwrap().hp,after.character.stats.max_hp);
  }
  assert!(g.fight.actions.iter().all(|a|a.before.draws>=g.battle_entry.draws));
  assert_eq!(g.field_return.after.draws,g.rewards.after.draws+2);
 }
}

/// Establish a fast valid plan first, then spend a deterministic amount of
/// work improving its total input cost. A truncated refinement is not a claim
/// that an outcome is impossible: the already validated plan remains available.
fn search_clockbank_bg(entry:&Entry)->Result<(ClockbankAdjustment,HarborReturn)> {
 let mut loader=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 let mut departures=vec![entry.clone()];
 let mut best:Option<(ClockbankAdjustment,HarborReturn)>=None;
 let refinement_work=std::cell::Cell::new(5_000usize);
 for phase in 0..3 {
  if phase==2 {refinement_work.set(refinement_work.get()+5_000);}
  let mut scan=entry.after;if phase==0 {scan.advance(650);}
  loop {
   if phase>0 {
    if refinement_work.get()==0 {break;}
    let max_pairs=best.as_ref().unwrap().0.input_cost.saturating_sub(7)/14;
    while departures.len()<=max_pairs as usize {add_clockbank_cycle(&mut loader,&mut departures)?;}
    if scan.draws>departures[max_pairs as usize].after.draws+1900 {break;}
   }
   let target=scan;scan.advance(1);
   let mut lottery=target;
   if ot2_battle_engine::bewildering_grace::Dance::new(3)?.select(&mut lottery)?.unwrap().ability!="ABI_DAN_MAKA_002" {continue;}
   let minimum_gap=if phase==0 {650}else{0};
   while loader.rng.draws+minimum_gap<target.draws {add_clockbank_cycle(&mut loader,&mut departures)?;}
   let mut trials:Vec<_>=departures.iter().enumerate().filter(|(_,e)|{
    let gap=target.draws.saturating_sub(e.after.draws);(minimum_gap..=1900).contains(&gap)&&e.after.draws<target.draws
   }).collect();
   trials.sort_by_key(|(_,e)|target.draws.saturating_sub(e.after.draws).abs_diff(850));
   let cache=FightCache::default();
   for &(index,departure) in &trials {
    if phase>0&&refinement_work.get()==0 {break;}
    let switch_cost=index as u32*14;
    let local_budget=if phase==0 {400}else{192};
    let ceiling=best.as_ref().map_or(local_budget,|(a,_)|local_budget.min(a.input_cost.saturating_sub(switch_cost+1)));
    if ceiling<6 {continue;}
    let search=BgTarget {rng:target,budget:ceiling,cache:&cache,guard_extra:if phase==2 {12}else{0},
     fight_cap:if phase==0 {100}else{192},action_cap:if phase==0 {74}else{192},work:(phase>0).then_some(&refinement_work)};
    if let Some(route)=field_continuation(departure,0,Some(search))? {
     let input_cost=switch_cost+route_cost(&route);
     if best.as_ref().is_none_or(|(b,_)|input_cost<b.input_cost) {
      best=Some((ClockbankAdjustment {switches:index as u32*2,target,departure:departure.clone(),input_cost},route));
     }
    }
   }
   if phase==0&&best.is_some() {break;}
  }
 }
 // Adaptive rescue: the incumbent TOTAL cost is the only player-input
 // ceiling. Gap estimates order work; they never exclude a departure.
 // Keep a separate computational allowance so expensive prefixes cannot starve
 // every other departure/anchor. Truncated queries are never cached as failures.
 let horizon=best.as_ref().unwrap().0.target.draws;
 let mut targets=Vec::new();let mut scan=entry.after;
 while scan.draws<=horizon {
  let target=scan;scan.advance(1);let mut lottery=target;
  if ot2_battle_engine::bewildering_grace::Dance::new(3)?.select(&mut lottery)?.unwrap().ability=="ABI_DAN_MAKA_002" {targets.push(target);}
 }
 let mut adaptive_work=(best.as_ref().unwrap().0.input_cost as usize).saturating_mul(30).min(90_000);
 'adaptive: for preferred_gap in [1150u64,2200,3800] {
  for &target in &targets {
   let mut target_work=12_000usize;
   let mut trials:Vec<_>=departures.iter().enumerate().filter(|(i,e)|e.after.draws<target.draws&&(*i as u32)*14+6<best.as_ref().unwrap().0.input_cost).collect();
   trials.sort_by_key(|(_,e)|(target.draws-e.after.draws).abs_diff(preferred_gap));
   let cache=FightCache::default();
   for &(index,departure) in &trials {
    if adaptive_work==0 {break 'adaptive;}
    if target_work==0 {break;}
    let switch_cost=index as u32*14;
    let ceiling=best.as_ref().unwrap().0.input_cost.saturating_sub(switch_cost+1);
    if ceiling<6 {continue;}
    let allowance=adaptive_work.min(target_work).min(6000);let work=std::cell::Cell::new(allowance);
    let search=BgTarget {rng:target,budget:ceiling,cache:&cache,guard_extra:12,fight_cap:ceiling as usize,action_cap:ceiling,work:Some(&work)};
    if let Some(route)=field_continuation(departure,0,Some(search))? {
     let input_cost=switch_cost+route_cost(&route);
     if input_cost<best.as_ref().unwrap().0.input_cost {best=Some((ClockbankAdjustment {switches:index as u32*2,target,departure:departure.clone(),input_cost},route));}
    }
    adaptive_work-=allowance-work.get();target_work-=allowance-work.get();
   }
  }
 }
 let (mut adjustment,route)=best.expect("baseline search returns only a valid plan");
 let mut replay=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 for i in 0..adjustment.switches {replay.manual_time_switch(if i%2==0 {4}else{2},"Clockbank Industrial District: BG adjustment")?;}
 ensure(replay.rng==adjustment.departure.after,"Clockbank switch replay diverged")?;
 adjustment.departure=point(&mut replay,entry.after);
 Ok((adjustment,route))
}
fn add_clockbank_cycle(loader:&mut Loader<'_>,departures:&mut Vec<Entry>)->Result<()> {
 let before=loader.rng;
 loader.manual_time_switch(4,"Clockbank Industrial District: BG adjustment night")?;
 loader.manual_time_switch(2,"Clockbank Industrial District: BG adjustment day")?;
 departures.push(point(loader,before));Ok(())
}

fn fight_input_cost(f:&ot2_battle_engine::gravell_routine::Fight)->u32 {
 f.actions.iter().filter(|a|matches!(a.action,ot2_battle_engine::gravell_routine::Action::Guards|ot2_battle_engine::gravell_routine::Action::BewilderingGrace)).map(|a|a.inputs.len() as u32).sum()
}

fn route_cost(r:&HarborReturn)->u32 {
 r.inputs.levels.len() as u32+r.defends.len() as u32*crate::escape_search::DEFEND_INPUT_COST+if r.victory.is_some(){10}else{0}
 +r.southern.as_ref().map_or(0,|s|route_cost(s))
 +r.gravell.as_ref().map_or(0,|g|fight_input_cost(&g.fight))
}

#[cfg(test)]
mod clockbank_switch_benchmark {
 #[test]
 fn refinement_improves_known_plan_and_replays_full_fight() {
  let world=serde_json::from_str(include_str!("../data/source/clockbank-search-party.json")).unwrap();
  let rng=super::CrtRng::new(3221999247);
  let entry=super::Entry {arrival:rng,after:rng,world_after:world,trace:vec![],decisions:vec![]};
  let (adjustment,route)=super::search_clockbank_bg(&entry).unwrap();
  assert!(adjustment.input_cost<293,"Old search needed 293 input equivalents");
  assert_eq!(adjustment.input_cost,adjustment.switches*7+super::route_cost(&route));
  let mut end=&route;while let Some(next)=&end.southern {end=next;}
  let fight=&end.gravell.as_ref().unwrap().fight;
  assert_eq!(fight.evaluation,ot2_battle_engine::gravell_routine::Evaluation::Accepted);
  assert!(fight.state.actors[..4].iter().all(|a|a.alive()));
  assert_eq!(fight.state.grace.as_ref().unwrap().exp_percent(),10000);
  assert!(fight.trace.iter().any(|s|s.reason=="guards_variant_preparation"),"accepted fight must retain full replay trace");
 }

 #[test]
 fn adaptive_search_recovers_earlier_valid_outlier_target() {
  use super::*;
  let world=serde_json::from_str(include_str!("../data/source/clockbank-search-party.json")).unwrap();
  let rng=CrtRng::new(1775700753);
  let entry=Entry {arrival:rng,after:rng,world_after:world,trace:vec![],decisions:vec![]};
  let (adjustment,route)=search_clockbank_bg(&entry).unwrap();
  assert!(adjustment.input_cost<1300);
  assert_eq!(adjustment.target.draws,4280);
  assert_eq!(adjustment.input_cost,adjustment.switches*7+route_cost(&route));
  let mut end=&route;while let Some(next)=&end.southern {end=next;}
  let fight=&end.gravell.as_ref().unwrap().fight;
  assert_eq!(fight.evaluation,ot2_battle_engine::gravell_routine::Evaluation::Accepted);
  assert!(fight.state.actors[..4].iter().all(|a|a.alive()));
  assert!(fight.trace.iter().any(|s|s.reason=="guards_variant_preparation"));
 }

 #[test]
 #[ignore = "isolate search limits on a cheaper outlier witness"]
 fn audit_484_limits() {
  use super::*;
  let world=serde_json::from_str(include_str!("../data/source/clockbank-search-party.json")).unwrap();
  let rng=CrtRng::new(1775700753);
  let mut loader=Loader::new(world::catalog(),world,rng).unwrap();
  let mut departure=point(&mut loader,rng);
  for _ in 0..138 {
   loader.manual_time_switch(4,"audit night").unwrap();
   loader.manual_time_switch(2,"audit day").unwrap();
   departure=point(&mut loader,rng);
  }
  let mut target=rng;target.advance(7504);
  for (budget,guard_extra) in [(192,12),(384,12),(192,384),(384,384)] {
   let cache=FightCache::default();let work=std::cell::Cell::new(100_000);
   let search=BgTarget {rng:target,budget,cache:&cache,guard_extra,fight_cap:budget as usize,action_cap:budget,work:Some(&work)};
   let route=field_continuation(&departure,0,Some(search)).unwrap();
   eprintln!("LIMIT budget={budget} guards={guard_extra} cost={:?} work={} truncated={}",route.as_ref().map(|r|1932+route_cost(r)),100000-work.get(),work.get()==0);
   if let Some(route)=route {
    let mut end=&route;while let Some(next)=&end.southern {end=next;}
    let fight=&end.gravell.as_ref().unwrap().fight;
    assert_eq!(fight.evaluation,ot2_battle_engine::gravell_routine::Evaluation::Accepted);
    assert!(fight.state.actors[..4].iter().all(|a|a.alive()));
    eprintln!("ACTIONS {:?}",fight.actions.iter().map(|a|(&a.action,a.inputs.len())).collect::<Vec<_>>());
    eprintln!("EFFECTS {:?}",fight.trace.iter().filter(|s|s.reason.contains("MAKA")||s.reason.contains("lottery")).collect::<Vec<_>>());
    std::fs::create_dir_all("outputs").unwrap();
    std::fs::write(format!("outputs/clockbank-276-witness-{budget}-{guard_extra}.json"),serde_json::to_vec(&route).unwrap()).unwrap();
   }
  }
 }

 #[test]
 #[ignore = "offline audit of the 484-switch outlier"]
 fn audit_484_earlier_anchors() {
  use super::*;
  let world=serde_json::from_str(include_str!("../data/source/clockbank-search-party.json")).unwrap();
  let rng=CrtRng::new(1775700753);
  let entry=Entry {arrival:rng,after:rng,world_after:world,trace:vec![],decisions:vec![]};
  let mut loader=Loader::new(world::catalog(),entry.world_after.clone(),rng).unwrap();
  let mut departures=vec![entry];
  while loader.rng.draws<7504 {add_clockbank_cycle(&mut loader,&mut departures).unwrap();}
  for offset in [4280,7504] {
   let mut target=rng;target.advance(offset);
   let mut trials:Vec<_>=departures.iter().enumerate().filter(|(_,e)|e.after.draws<offset&&(offset-e.after.draws)<=1900).collect();
   trials.sort_by_key(|(_,e)|(offset-e.after.draws).abs_diff(850));
   let mut best=None;
   let mut tested=0;
   for (index,departure) in trials {
    let work=std::cell::Cell::new(20_000);
    let cache=FightCache::default();
    let search=BgTarget {rng:target,budget:384,cache:&cache,guard_extra:384,fight_cap:384,action_cap:384,work:Some(&work)};
    if let Some(route)=field_continuation(departure,0,Some(search)).unwrap() {
     let cost=index as u32*14+route_cost(&route);
     eprintln!("VALID anchor={offset} switches={} cost={cost}",index*2);
     best=Some(cost);break;
    }
    tested+=1;
    eprintln!("AUDIT anchor={offset} switches={} work={} truncated={}",index*2,20000-work.get(),work.get()==0);
   }
   eprintln!("RESULT anchor={offset} tested={tested} valid={best:?}");
  }
 }

 #[test]
 #[ignore = "explicit 1000-state production-search benchmark"]
 fn sample_switch_counts() {
  use super::super::*;
  use std::io::Write;
  let world:world::WorldState=serde_json::from_str(include_str!("../data/source/clockbank-search-party.json")).unwrap();
  let count=std::env::var("OT2_BENCH_COUNT").ok().and_then(|s|s.parse().ok()).unwrap_or(1000usize);
  let next=std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
  let file=std::sync::Arc::new(std::sync::Mutex::new(std::fs::File::create(std::env::var("OT2_BENCH_OUTPUT").unwrap_or_else(|_|"outputs/clockbank-switch-optimized.jsonl".into())).unwrap()));
  let mut threads=vec![];
  for _ in 0..4 {
   let (next,file,world)=(next.clone(),file.clone(),world.clone());
   threads.push(std::thread::Builder::new().stack_size(16*1024*1024).spawn(move || {
    loop {
     let i=next.fetch_add(1,std::sync::atomic::Ordering::Relaxed);if i>=count {break;}
     let seed=std::env::var("OT2_BENCH_STATE").ok().and_then(|s|s.parse().ok()).unwrap_or_else(||(i as u32).wrapping_mul(2654435761).wrapping_add(0x91e10da5));
     let rng=CrtRng::new(seed);
     let entry=snows_escape::Entry {arrival:rng,after:rng,world_after:world.clone(),trace:vec![],decisions:vec![]};
     let now=std::time::Instant::now();
     let row=match inventor::search_clockbank_bg(&entry) {
      Ok((a,_))=>serde_json::json!({"i":i,"state":seed,"switches":a.switches,"input_cost":a.input_cost,"target_draws":a.target.draws,"seconds":now.elapsed().as_secs_f64()}),
      Err(e)=>serde_json::json!({"i":i,"state":seed,"error":e.to_string(),"seconds":now.elapsed().as_secs_f64()}),
     };
     let mut f=file.lock().unwrap();writeln!(f,"{row}").unwrap();f.flush().unwrap();
    }
   }).unwrap());
  }
  for t in threads {t.join().unwrap();}
 }
}
