//! Return to New Delsta, one daytime Highroad encounter, then Arkar.
use crate::{Result,CrtRng,ensure,SimError,world::{self,Loader},snows_escape::Entry,
 encounter::{self,Boundary,PredictionOptions},manipulation::{BoostInputs,describe_inputs}};
use ot2_battle_engine::{escape::Situation,field_escape::{FieldEscapeMenu,FieldEscapeAttempt}};
use serde::Serialize;
#[derive(Debug,Serialize)]
pub struct Approach {#[serde(skip_serializing_if="Option::is_none")] pub thief_reach:Option<ThiefReach>,#[serde(skip_serializing_if="Option::is_none")] pub highroad_day_switch:Option<Entry>,pub town:Entry,pub agnea_story:Entry,pub highroad:Entry,pub encounter:Option<encounter::Prediction>,pub battle_entry:CrtRng,pub surprise_percent:u32}
#[derive(Debug,Serialize)]
pub struct ThiefReach {pub initial_counter:u32,pub walking_updates:u32,pub running_updates:u32,pub run_percent:f64,pub walk_decrement:u32,pub run_decrement:u32,pub switch_to_walking_counter:u32,pub counter_cost:u32,pub estimated_counter_at_thief:u32}
fn thief_reach(counter:u32,evasive:bool)->Result<Option<ThiefReach>> {
 measured_highroad_reach(counter,evasive,117,124)
}
fn inventor_reach(counter:u32,evasive:bool)->Result<Option<ThiefReach>> {
 measured_highroad_reach(counter,evasive,86,90)
}
fn measured_highroad_reach(counter:u32,evasive:bool,full_run:u32,full_walk:u32)->Result<Option<ThiefReach>> {
 let mut o=PredictionOptions {zone:"ENT_FLD_CTY_1_3_LND".into(),period:"DayTime".into(),ability_percent:if evasive {50}else{100},dash:false,..Default::default()};
 let walk=encounter::catalog().counter_decrement(&o)?;o.dash=true;let run=encounter::catalog().counter_decrement(&o)?;
 // Use the measured endpoints without extra safety updates. Choose
 // whole running updates, then round the remaining walking updates up.
 for fast in (0..=full_run).rev() {
  let slow=(full_walk*(full_run-fast)).div_ceil(full_run);
  let cost=fast*run+slow*walk;
  if cost<counter {return Ok(Some(ThiefReach {initial_counter:counter,walking_updates:slow,running_updates:fast,run_percent:(fast*1000/full_run) as f64/10.0,walk_decrement:walk,run_decrement:run,switch_to_walking_counter:counter-fast*run,counter_cost:cost,estimated_counter_at_thief:counter-cost}));}
 }
 Ok(None)
}
#[derive(Debug,Serialize)]
pub struct ClockbankAdjustment {pub switches:u32,pub target:CrtRng,pub departure:Entry,pub input_cost:u32}
#[derive(Debug,Serialize)]
pub struct Thief {pub night:Entry,pub introduction:Entry,pub battle:ot2_battle_engine::field_escape::FieldRareVictory,pub field_return:Entry,pub bag:Entry,pub day:Entry}
fn thief(entry:&Entry)->Result<Thief> {
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 l.manual_time_switch(4,"Traveler's Bag: switch to night at the thief")?;let night=point(&mut l,entry.after);
 let request=l.skipped_event_until_battle("SC_SP_Tutorial_0200",&world::EventChoices::from([(("SC_SP_Tutorial_0200".into(),4),"eDIALOG_YES".into())]))?;
 ensure(request.volume=="EVM_EVE_SUB_OTR_100","Unexpected Traveler's Bag battle")?;
 for id in l.state.party.clone() {l.draw(world::DrawKind::BattleTiming,&id.to_string(),"Traveler's Bag: BattlePlayerBP.Initialize timing",None);}
 let introduction=point(&mut l,night.after);let w=&introduction.world_after;
 let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
 let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
 let battle=ot2_battle_engine::field_escape::carried_soulstone(introduction.after,Situation::Normal,&players,&levels,&["ENE_EVE_SUB_OTR_100"],true,&w.enemy_knowledge,&[],"shadow_soulstone_m")?.ok_or_else(||SimError("Medium Soulstone did not defeat the thief".into()))?;
 let mut state=w.clone();crate::conning::apply_item_victory(&mut state,&battle)?;
 let mut l=Loader::new(world::catalog(),state,battle.after)?;l.restore_field_after_battle("Traveler's Bag thief: turn-one Shadow Soulstone M victory")?;
 let field_return=point(&mut l,battle.after);
 l.skipped_event(&request.victory_event)?;let bag=point(&mut l,field_return.after);
 ensure(l.state.flag(20510),"Traveler's Bag was not recovered")?;
 l.manual_time_switch(2,"Traveler's Bag: return to day before Arkar")?;let day=point(&mut l,bag.after);
 Ok(Thief {night,introduction,battle,field_return,bag,day})
}
#[derive(Debug,Serialize)]
pub struct Prediction {#[serde(skip_serializing_if="Option::is_none")] pub inventor_reach:Option<ThiefReach>,#[serde(skip_serializing_if="Option::is_none")] pub inventor_arrival:Option<Entry>,#[serde(skip_serializing_if="Option::is_none")] pub new_delsta_adjustment:Option<ClockbankAdjustment>,pub clockbank_adjustment:Option<ClockbankAdjustment>,pub approach:Approach,pub menu:Option<FieldEscapeMenu>,pub inputs:BoostInputs,pub escape:Option<FieldEscapeAttempt>,pub field_return:Entry,pub license:Entry,pub bomb:Entry,pub scope:Entry,#[serde(skip_serializing_if="Option::is_none")] pub thief:Option<Thief>,pub western_clockbank:Option<Entry>,pub southern_clockbank:Option<Entry>,pub clockbank_reach:Option<crate::winter_voyage::CropdaleReach>,pub clockbank:Option<Entry>,pub partitio_story:Option<Entry>,pub clockite_stages:Vec<(String,Entry)>,pub crackridge_return:HarborReturn}
#[derive(Debug,Serialize)]
pub struct HarborReturn {#[serde(skip_serializing_if="Vec::is_empty")] pub defend_rounds:Vec<u32>,#[serde(skip_serializing_if="is_first_round")] pub escape_round:u32,#[serde(skip_serializing_if="Option::is_none")] pub bg_adjustment:Option<ClockbankAdjustment>,pub defends:Vec<ot2_battle_engine::StepReport>,pub arrival:Entry,pub encounter:encounter::Prediction,pub battle_entry:CrtRng,pub menu:FieldEscapeMenu,pub inputs:BoostInputs,pub escape:Option<FieldEscapeAttempt>,pub victory:Option<ot2_battle_engine::field_escape::FieldRareVictory>,pub field_return:Entry,pub southern:Option<Box<HarborReturn>>,pub approach_stages:Vec<(String,Entry)>,pub gravell:Option<GravellArrival>}
#[derive(Debug,Serialize)]
pub struct GravellArrival {pub license:Entry,pub setup:Entry,pub arrival:Entry,pub night:Entry,pub encounter:encounter::Prediction,pub battle_entry:CrtRng,pub fight:ot2_battle_engine::gravell_routine::Fight,pub rewards:ot2_battle_engine::gravell_routine::Rewards,pub field_return:Entry,pub town:Entry,#[serde(skip_serializing_if="Option::is_none")] pub coast:Option<crate::galdera_coast::Continuation>}
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
 let Some(fight)=fight else {return Ok(None);};
 let (fight,rewards,field_return)=settle_gravell(&night,rng,fight)?;
 let mut l=Loader::new(world::catalog(),field_return.world_after.clone(),field_return.after)?;
 l.visit_map("Twn_Wld_3_1_A")?;
 let town=point(&mut l,field_return.after);
 Ok(Some(GravellArrival {license,setup,arrival,night,encounter,battle_entry:rng,fight,rewards,field_return,town,coast:None}))
}
pub(crate) fn settle_gravell(night:&Entry,rng:CrtRng,mut fight:ot2_battle_engine::gravell_routine::Fight)->Result<(ot2_battle_engine::gravell_routine::Fight,ot2_battle_engine::gravell_routine::Rewards,Entry)> {
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
 Ok((fight,rewards,field_return))
}
fn field_continuation(entry:&Entry,stage:usize,target:Option<BgTarget>)->Result<Option<HarborReturn>> {
 if target.is_some_and(|t|entry.after.draws>=t.rng.draws||t.budget<6||t.work.is_some_and(|w|w.get()==0)){return Ok(None);}
 let (map,zone,label)=[("Fld_Wld_1_2","ENT_FLD_WLD_1_2_LND","Crackridge Harbor return"),("Fld_Wld_2_2","ENT_FLD_WLD_2_2_LND","Southern Crackridge Wilds"),("Fld_Wld_2_1","ENT_FLD_WLD_2_1_LND","Western Crackridge Wilds first visit"),("Fld_Wld_2_1","ENT_FLD_WLD_2_1_LND","Western Crackridge Wilds after town")][stage];
 // Fast travel is user-authorized to the Harbor tagged earlier in this route.
 // Older continuation fixtures do not retain the complete visited-map ledger.
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 let mut approach_stages=vec![];
 if stage==3 && l.state.map!="Twn_Wld_2_1_A" {l.visit_map("Twn_Wld_2_1_A")?;approach_stages.push(("Crackridge · loaded".into(),point(&mut l,entry.after)));}
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
 let mut menus=vec![(menu.clone(),vec![],vec![],0u32)];
 if target.is_some() {for _ in 0..if w.galdera_route {7} else {3} {
  let (m,actions,rounds,cost)=menus.last().unwrap();
  let defend=if w.galdera_route {m.safe_step_ahead_defend_through_next_round()?} else {m.safe_step_ahead_defend()?};
  let Some((next,report))=defend else {break};
  let next_cost=*cost+crate::escape_search::DEFEND_INPUT_COST+crate::escape_search::elapsed_round_cost(m.round(),next.round());
  let mut rounds=rounds.clone();rounds.push(m.round());
  let mut actions=actions.clone();actions.push(report);menus.push((next,actions,rounds,next_cost));
 }}
 let mut escape_cursors:Vec<_>=menus.iter().map(|(m,_,_,_)|(m.clone(),0u32)).collect();
 let mut best:Option<HarborReturn>=None;
 for cost in 0..=target.map_or(u32::MAX/2,|t|t.budget.min(t.action_cap)) {
 if target.is_some_and(|t|t.work.is_some_and(|w|w.get()==0)){return Ok(best);}
 for (menu_index,(menu,defends,defend_rounds,defend_cost)) in menus.iter().enumerate() {
 for kill in [false,true] {
  if kill&&(stage!=3||(!w.galdera_route&&!defends.is_empty())) {continue;}
  let action_cost=if kill {10}else{0}+*defend_cost;
  if cost<action_cost||(cost-action_cost)%2!=0 {continue;}
  let n=(cost-action_cost)/2;
  if let Some(work)=target.and_then(|t|t.work) {
   let units=if kill {5}else{1};
   if work.get()<units {work.set(0);return Ok(best);}
   work.set(work.get()-units);
  }
  let inputs=describe_inputs(n,0,crate::conning::path(n))?;
  let (escape,victory,field_return)=if kill {
   let Some(v)=ot2_battle_engine::field_escape::carried_soulstone_after_defends(rng,situation,&players,&levels,&ids,w.time!=2,&w.enemy_knowledge,&inputs.levels,"wind_soulstone_l",step_ahead,defends.len())? else {continue;};
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
   let Some(south)=field_continuation(&field_return,stage+1,target.map(|t|BgTarget {budget:t.budget-cost,..t}))? else {
    if target.is_some() && !(w.galdera_route && target.is_some_and(|t|t.work.is_some())) {return Ok(None);}
    continue;
   };
   Some(Box::new(south))
  } else {None};
  let gravell=if stage==3 {let Some(g)=gravell_target(&field_return,target.map(|t|BgTarget {budget:best.as_ref().map_or(t.budget,|b|t.budget.min(route_cost(b).saturating_sub(1))).saturating_sub(cost),..t}))? else {continue;};Some(g)}else{None};
  let candidate=HarborReturn {defend_rounds:if w.galdera_route {defend_rounds.clone()} else {vec![]},escape_round:menu.round(),bg_adjustment:None,defends:defends.clone(),arrival:arrival.clone(),encounter:encounter.clone(),battle_entry:rng,menu:menu.clone(),inputs,escape,victory,field_return,southern,approach_stages:approach_stages.clone(),gravell};
  if stage<3 && !(w.galdera_route && target.is_some_and(|t|t.work.is_some())) {return Ok(Some(candidate));}
  if best.as_ref().is_none_or(|b|route_cost(&candidate)<route_cost(b)) {best=Some(candidate);}
 }
 }
 if best.as_ref().is_some_and(|b|cost>=route_cost(b)) {break;}
 }
 if best.is_some()||target.is_some(){return Ok(best);}
 Err(SimError("Crackridge return Escape search exhausted".into()))
}
fn is_first_round(round:&u32)->bool {*round==1}
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
 approach_from_town(&town)
}
fn approach_from_town(town:&Entry)->Result<Approach> {
 let town=town.clone();
 let mut l=Loader::new(world::catalog(),town.world_after.clone(),town.after)?;
 if !l.state.galdera_route {
  l.skipped_event_with_choices("MapInStorySelect",&world::EventChoices::from([(("MapInStorySelect".into(),1),"MS_ODO_01".into())]))?;
  ensure(l.state.active_scenarios.contains(&801) && l.state.flag(6550),"Agnea Chapter 2 opening did not finish")?;
 } else {
  l.boundary("Galdera: leave Agnea Chapter 2 inactive; depart New Delsta without story selection");
 }
 let agnea_story=point(&mut l,town.after);
 l.visit_map("Fld_Cty_1_3")?;let highroad=point(&mut l,agnea_story.after);
 let highroad_day_switch=if l.state.galdera_route && l.state.time==4 {
  l.manual_time_switch(2,"Eastern New Delsta Highroad: switch to day immediately after loading")?;
  Some(point(&mut l,highroad.after))
 } else {None};
 if l.state.galdera_route {
  let reach=thief_reach(l.state.encounter_counter,l.state.evasive_maneuvers)?;
  return Ok(Approach {thief_reach:reach,highroad_day_switch,town,agnea_story,highroad,encounter:None,battle_entry:l.rng,surprise_percent:0});
 }
 let levels:Vec<_>=l.state.party.iter().map(|id|l.state.character_progression[id].level).collect();
 let mut rng=l.rng;
 let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions {map:l.state.map.clone(),zone:"ENT_FLD_CTY_1_3_LND".into(),period:"DayTime".into(),progress:l.state.main_progress_points,first_encounter:l.state.first_encounter,boundary:Boundary::CurrentCounter,current_counter:Some(l.state.encounter_counter),party_levels:levels.clone(),ability_percent:if l.state.evasive_maneuvers {50}else{100},dash:true,..Default::default()})?;
 let enemy=encounter.enemies.iter().map(|e|e.enemy.level as i32).sum::<i32>()/encounter.enemies.len() as i32;
 let party=levels.iter().sum::<u32>() as i32/levels.len() as i32;
 Ok(Approach {thief_reach:None,highroad_day_switch,town,agnea_story,highroad,encounter:Some(encounter),battle_entry:rng,surprise_percent:(5+4*(enemy-party).clamp(-1,6)) as u32})
}
pub fn safe(a:&Approach)->bool {if a.highroad.world_after.galdera_route {a.thief_reach.is_some()}else{a.encounter.as_ref().is_some_and(|e|matches!(e.situation.as_deref(),Some("Normal"|"Preemptive")))}}
pub fn finish(a:Approach)->Result<Prediction> {
 if a.highroad.world_after.galdera_route {
  let town=a.town.clone();
  let (adjustment,mut prediction)=search_bg(&town,|departure,target| {
   let a=approach_from_town(departure)?;
   if !safe(&a) {return Ok(None);}
   let day_cost=14+if a.highroad_day_switch.is_some() {7}else{0};
   if target.budget<day_cost {return Ok(None);}
   finish_target(a,Some(BgTarget {budget:target.budget-day_cost,..target}))
  },|p:&Prediction|14+p.inputs.levels.len() as u32+route_cost(&p.crackridge_return)+if p.approach.highroad_day_switch.is_some() {7}else{0})?;
  prediction.approach.town=town;
  prediction.new_delsta_adjustment=Some(adjustment);
  // Freeze the completed EXP100 search before optimizing the new coast leg.
  let mut end=&mut prediction.crackridge_return;
  while let Some(ref mut next)=end.southern {end=next;}
  crate::galdera_coast::extend(end.gravell.as_mut().ok_or_else(||SimError("Galdera route did not reach the Chubby victory".into()))?)?;
  return Ok(prediction);
 }
 finish_target(a,None)?.ok_or_else(||SimError("Inventor route exhausted".into()))
}
fn finish_target(a:Approach,target:Option<BgTarget>)->Result<Option<Prediction>> {
 if a.highroad.world_after.galdera_route {
  let Some(reach)=a.thief_reach.as_ref() else {return Ok(None);};
  let start=a.highroad_day_switch.as_ref().unwrap_or(&a.highroad);
  let mut l=Loader::new(world::catalog(),start.world_after.clone(),start.after)?;
  l.state.encounter_counter=reach.estimated_counter_at_thief;
  l.boundary(format!("Highroad to thief: run {} updates (counter {} -> {}), walk {} updates (counter -> {}); measured 117/124 endpoints, no added leeway",reach.running_updates,reach.initial_counter,reach.switch_to_walking_counter,reach.walking_updates,reach.estimated_counter_at_thief));
  let field_return=point(&mut l,start.after);
  let thief=thief(&field_return)?;
  let mut l=Loader::new(world::catalog(),thief.day.world_after.clone(),thief.day.after)?;
  let Some(inventor_reach)=inventor_reach(l.state.encounter_counter,l.state.evasive_maneuvers)? else {return Ok(None);};
  l.state.encounter_counter=inventor_reach.estimated_counter_at_thief;
  l.boundary(format!("Thief to Inventor house: run {} updates ({:.1}% of route; counter {} -> {}), walk {} updates (counter -> {}); measured 86/90 endpoints, no leeway",inventor_reach.running_updates,inventor_reach.run_percent,inventor_reach.initial_counter,inventor_reach.switch_to_walking_counter,inventor_reach.walking_updates,inventor_reach.estimated_counter_at_thief));
  let inventor_arrival=point(&mut l,thief.day.after);
  l.skipped_event("SYS_INV_GUILD_0010")?;let license=point(&mut l,inventor_arrival.after);
  l.skipped_event("SYS_INV_GUILD_0D10")?;let bomb=point(&mut l,license.after);
  l.skipped_event("SYS_INV_GUILD_0C10")?;crate::gravell_setup::equip_step_ahead(&mut l)?;let scope=point(&mut l,bomb.after);
  let Some(crackridge_return)=field_continuation(&scope,0,target)? else {return Ok(None);};
  return Ok(Some(Prediction {inventor_reach:Some(inventor_reach),inventor_arrival:Some(inventor_arrival),new_delsta_adjustment:None,clockbank_adjustment:None,approach:a,menu:None,inputs:describe_inputs(0,0,vec![])?,escape:None,field_return,license,bomb,scope,thief:Some(thief),western_clockbank:None,southern_clockbank:None,clockbank_reach:None,clockbank:None,partitio_story:None,clockite_stages:vec![],crackridge_return}));
 }
 ensure(safe(&a),"Inventor Highroad encounter must not surprise the party")?;
 let w=&a.highroad_day_switch.as_ref().unwrap_or(&a.highroad).world_after;
 let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
 let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
 let encounter=a.encounter.as_ref().unwrap();
 let ids:Vec<_>=encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
 let menu=FieldEscapeMenu::with_players(a.battle_entry,if encounter.situation.as_deref()==Some("Preemptive"){Situation::Preemptive}else{Situation::Normal},&players,&levels,&ids,&w.enemy_knowledge)?;
 for n in 0..u32::MAX/2 {
  if target.is_some_and(|t|2*n>t.budget || 2*n>t.action_cap || t.work.is_some_and(|w|w.get()==0)) {return Ok(None);}
  if let Some(work)=target.and_then(|t|t.work) {work.set(work.get()-1);}
  let inputs=describe_inputs(n,0,crate::conning::path(n))?;
  let escape=menu.clone().attempt(&inputs.levels,encounter.encounter_level.unwrap())?;
  if !escape.success {continue;}
  let ret=crate::highroad::restore_field(w,&escape,"Fld_Cty_1_3","Inventor Highroad Escape")?;
  let field_return=Entry {arrival:ret.arrival,after:ret.after,world_after:ret.world_after,trace:ret.trace,decisions:vec![]};
  let thief=if w.galdera_route {Some(thief(&field_return)?)}else{None};
  let craft_entry=thief.as_ref().map_or(&field_return,|t|&t.day);
  let mut l=Loader::new(world::catalog(),craft_entry.world_after.clone(),craft_entry.after)?;
  l.skipped_event("SYS_INV_GUILD_0010")?;let license=point(&mut l,craft_entry.after);
  l.skipped_event("SYS_INV_GUILD_0D10")?;let bomb=point(&mut l,license.after);
  l.skipped_event("SYS_INV_GUILD_0C10")?;crate::gravell_setup::equip_step_ahead(&mut l)?;let scope=point(&mut l,bomb.after);
  if scope.world_after.galdera_route {
   let Some(crackridge_return)=field_continuation(&scope,0,target.map(|t|BgTarget {budget:t.budget-2*n,..t}))? else {continue;};
   return Ok(Some(Prediction {inventor_reach:None,inventor_arrival:None,new_delsta_adjustment:None,clockbank_adjustment:None,approach:a,menu:Some(menu),inputs,escape:Some(escape),field_return,license,bomb,scope,thief,
    western_clockbank:None,southern_clockbank:None,clockbank_reach:None,clockbank:None,partitio_story:None,
    clockite_stages:vec![],crackridge_return}));
  }
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
  return Ok(Some(Prediction {inventor_reach:None,inventor_arrival:None,new_delsta_adjustment:None,clockbank_adjustment,approach:a,menu:Some(menu),inputs,escape:Some(escape),field_return,license,bomb,scope,thief,western_clockbank:Some(western_clockbank),southern_clockbank:Some(southern_clockbank),clockbank_reach,clockbank:Some(clockbank),partitio_story:Some(partitio_story),clockite_stages,crackridge_return}));
 }
 Err(SimError("Highroad Escape search exhausted".into()))
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 #[ignore = "Manual source audit; requires OT2_THIEF_TEST_ENTRY"]
 fn manual_highroad_inventor_progress_load_audit() {
  let input=std::env::var("OT2_THIEF_TEST_ENTRY").unwrap();
  let w:world::WorldState=serde_json::from_str(&std::fs::read_to_string(input).unwrap()).unwrap();
  let mut before=CrtRng::new(0);before.advance(3_093_251_025);
  let mut variants=vec![];
  for unlocked in [false,true] {
   let mut state=w.clone();state.set_flag(2408,unlocked);
   let mut l=Loader::new(world::catalog(),state,before).unwrap();l.visit_map("Fld_Cty_1_3").unwrap();
   variants.push(serde_json::json!({"inventor_unlocked":unlocked,"after":l.rng,"trace":l.trace}));
  }
  assert_eq!(variants[0]["after"]["draws"],3_093_251_044u64);
  assert_eq!(variants[1]["after"]["draws"],3_093_251_049u64);
  let output=std::env::var("OT2_THIEF_TEST_OUTPUT").unwrap();std::fs::write(output,serde_json::to_string_pretty(&variants).unwrap()).unwrap();
  println!("{}",serde_json::json!({"before":before,"without_inventor":variants[0]["after"],"with_inventor":variants[1]["after"]}));
 }
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
 #[ignore = "Manual seed/position replay; requires OT2_THIEF_TEST_ENTRY"]
 fn manual_seed_zero_highroad_thief_preview() {
  let path=std::env::var("OT2_THIEF_TEST_ENTRY").unwrap();
  let w:world::WorldState=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
  let mut before=CrtRng::new(0);before.advance(3_093_251_025);
  let mut l=Loader::new(world::catalog(),w,before).unwrap();let town=point(&mut l,before);
  let a=approach_from_town(&town).unwrap();let reach=a.thief_reach.as_ref().unwrap();
  let start=a.highroad_day_switch.as_ref().unwrap_or(&a.highroad);
  let mut l=Loader::new(world::catalog(),start.world_after.clone(),start.after).unwrap();l.state.encounter_counter=reach.estimated_counter_at_thief;
  let at_thief=point(&mut l,start.after);let t=thief(&at_thief).unwrap();
  let house=inventor_reach(t.day.world_after.encounter_counter,t.day.world_after.evasive_maneuvers).unwrap();
  let result=serde_json::json!({"seed":0,"before_highroad":before,"party":town.world_after.party,"party_levels":town.world_after.party.iter().map(|id|town.world_after.character_progression[id].level).collect::<Vec<_>>(),"approach":a,"at_thief":at_thief,"thief":t,"inventor_reach":house});
  let output=std::env::var("OT2_THIEF_TEST_OUTPUT").unwrap();std::fs::write(output,serde_json::to_string_pretty(&result).unwrap()).unwrap();
  println!("{}",serde_json::json!({"before":result["before_highroad"],"highroad":result["approach"]["highroad"]["after"],"movement":result["approach"]["thief_reach"],"night":result["thief"]["night"]["after"],"battle_actor":result["thief"]["battle"]["actor"],"battle":result["thief"]["battle"]["after"],"field_return":result["thief"]["field_return"]["after"],"cutscenes":result["thief"]["bag"]["after"],"back_to_day":result["thief"]["day"]["after"],"inventor_reach":result["inventor_reach"]}));
 }
 #[test]
 fn thief_to_inventor_uses_exact_post_bag_counter() {
  let low=inventor_reach(500,true).unwrap().unwrap();
  assert_eq!((low.running_updates,low.walking_updates,low.counter_cost),(75,12,498));assert_eq!(low.run_percent,87.2);
  assert!(inventor_reach(516,true).unwrap().unwrap().run_percent<100.0);
  let fast=inventor_reach(517,true).unwrap().unwrap();assert_eq!((fast.running_updates,fast.walking_updates,fast.run_percent),(86,0,100.0));
  for counter in 500..=899 {let r=inventor_reach(counter,true).unwrap().unwrap();assert!(r.counter_cost<counter);assert_eq!(r.counter_cost,6*r.running_updates+4*r.walking_updates);}
 }
 #[test]
 fn highroad_movement_uses_exact_counter_and_whole_updates() {
  assert!(thief_reach(496,true).unwrap().is_none());
  let slow=thief_reach(497,true).unwrap().unwrap();assert_eq!((slow.running_updates,slow.walking_updates,slow.counter_cost),(0,124,496));
  let low=thief_reach(500,true).unwrap().unwrap();assert_eq!((low.running_updates,low.walking_updates,low.switch_to_walking_counter,low.estimated_counter_at_thief),(1,123,494,2));
  assert!(thief_reach(702,true).unwrap().unwrap().running_updates<117);
  assert_eq!(thief_reach(703,true).unwrap().unwrap().running_updates,117);
  for counter in 500..=899 {let r=thief_reach(counter,true).unwrap().unwrap();assert!(r.counter_cost<counter);assert_eq!(r.counter_cost,6*r.running_updates+4*r.walking_updates);assert_eq!((r.walk_decrement,r.run_decrement),(4,6));}
 }
 #[test]
 fn galdera_thief_medium_soulstone_carries_rewards_and_bag() {
  let fixture:serde_json::Value=serde_json::from_str(include_str!("../data/source/retail-inventor-highroad-20261010.json")).unwrap();
  let mut w:world::WorldState=serde_json::from_value(fixture["world"].clone()).unwrap();w.galdera_route=true;w.set_flag(20505,true);w.time=2;
  w.party=vec![7,6,8,4];w.party_experience=w.party.iter().map(|id|w.character_progression[id].experience).collect();w.sync_progression().unwrap();
  for id in w.party.clone(){let c=&mut w.character_progression.get_mut(&id).unwrap().character;let r=c.resources.as_mut().unwrap();r.hp=r.hp.min(c.stats.max_hp);r.sp=r.sp.min(c.stats.max_sp);}
  let rng=CrtRng::new(77);let mut l=Loader::new(world::catalog(),w,rng).unwrap();l.visit_map("Fld_Cty_1_3").unwrap();
  let entry=point(&mut l,rng);let p=thief(&entry).unwrap();
  assert_eq!(p.battle.base_jp,20);assert_eq!(p.battle.base_exp,20);assert_eq!(p.day.world_after.time,2);
  assert!(p.bag.world_after.flag(20510));assert!(!p.bag.world_after.flag(20511));
  for id in &entry.world_after.party {assert!(p.day.world_after.character_progression[id].jp_earned>entry.world_after.character_progression[id].jp_earned);}
  assert!(p.battle.report.events.iter().any(|e|matches!(e,ot2_battle_engine::Event::Damage {amount,..} if *amount>=280)));
 }
 #[test]
 fn galdera_new_delsta_night_departure_restores_day_on_highroad() {
  let fixture:serde_json::Value=serde_json::from_str(include_str!("../data/source/retail-inventor-highroad-20261010.json")).unwrap();
  let mut w:world::WorldState=serde_json::from_value(fixture["world"].clone()).unwrap();
  w.galdera_route=true;w.time=2;
  let rng=CrtRng::new(77);let mut l=Loader::new(world::catalog(),w,rng).unwrap();
  l.visit_map("Twn_Cty_1_1_A").unwrap();
  l.manual_time_switch(4,"test night departure").unwrap();
  let town=point(&mut l,rng);
  let a=approach_from_town(&town).unwrap();
  assert_eq!(a.town.world_after.time,4);assert_eq!(a.highroad.world_after.time,4);
  let day=a.highroad_day_switch.as_ref().unwrap();
  assert_eq!(day.world_after.time,2);assert_eq!(day.arrival,a.highroad.after);
  assert_eq!(day.world_after.encounter_counter,a.highroad.world_after.encounter_counter);
  let mut replay=Loader::new(world::catalog(),town.world_after.clone(),town.after).unwrap();
  replay.visit_map("Fld_Cty_1_3").unwrap();assert_eq!(replay.rng,a.highroad.after);
  replay.manual_time_switch(2,"test required highroad switch").unwrap();assert_eq!(replay.rng,day.after);
 }
 #[test]
 fn galdera_direct_inventor_return_uses_new_delsta_switches_and_replays_exp100() {
  let fixture:serde_json::Value=serde_json::from_str(include_str!("../data/source/retail-inventor-highroad-20261010.json")).unwrap();
  let mut w:world::WorldState=serde_json::from_value(fixture["world"].clone()).unwrap();
  w.sync_progression().unwrap();w.galdera_route=true;
  let later_jp=w.character_progression[&1].jp_earned;
  w.character_progression.get_mut(&6).unwrap().jp_earned+=later_jp;
  w.party=vec![7,6,8,4];w.bench.retain(|&id|id!=6);w.character_progression.remove(&1);
  w.party_experience=w.party.iter().map(|id|w.character_progression[id].experience).collect();
  // The retail fixture predates the corrected coast EXP; clamp its stored
  // resources to the recomputed caps before constructing this test battle.
  for id in w.party.clone() {
   let p=&mut w.character_progression.get_mut(&id).unwrap().character;
   let r=p.resources.as_mut().unwrap();r.hp=r.hp.min(p.stats.max_hp);r.sp=r.sp.min(p.stats.max_sp);
  }
  let throne=w.character_progression.get_mut(&7).unwrap();
  ot2_battle_engine::jobs::change_secondary_job(&mut throne.character,7,throne.level,throne.jobs.secondary_job,None).unwrap();
  throne.jobs.secondary_job=None;throne.jobs.equipped_support.remove(crate::scholar::EVASIVE);
  for scenario in [801,401] {w.active_scenarios.remove(&scenario);w.confirmed_scenarios.remove(&scenario);}
  for flag in [6550,34167,6076,6059,6077] {w.set_flag(flag,false);}
  w.refresh_support_skills();
  let rng=CrtRng::new(77);let entry=Entry {arrival:rng,after:rng,world_after:w,trace:vec![],decisions:vec![]};
  let a=approach(&entry).unwrap();assert!(safe(&a));
  let p=finish(a).unwrap();
  assert!(p.menu.is_none()&&p.escape.is_none()&&p.approach.encounter.is_none());
  let leg=p.inventor_reach.as_ref().unwrap();let arrival=p.inventor_arrival.as_ref().unwrap();
  assert_eq!(leg.initial_counter,p.thief.as_ref().unwrap().day.world_after.encounter_counter);
  assert_eq!(arrival.after,p.thief.as_ref().unwrap().day.after);
  assert_eq!(arrival.world_after.encounter_counter,leg.estimated_counter_at_thief);
  assert_eq!(p.field_return.after,p.approach.battle_entry);
  assert_eq!(p.field_return.world_after.encounter_counter,p.approach.thief_reach.as_ref().unwrap().estimated_counter_at_thief);
  assert!(p.clockbank.is_none()&&p.western_clockbank.is_none()&&p.southern_clockbank.is_none()&&p.partitio_story.is_none());
  assert!(p.clockite_stages.is_empty()&&p.clockbank_adjustment.is_none());
  assert_eq!(p.crackridge_return.arrival.arrival,p.scope.after);
  let west=p.crackridge_return.southern.as_ref().unwrap().southern.as_ref().unwrap().southern.as_ref().unwrap();
  let adjustment=p.new_delsta_adjustment.as_ref().unwrap();
  assert!(west.bg_adjustment.is_none());
  assert_eq!(adjustment.departure.world_after.map,"Twn_Cty_1_1_A");
  let town=&p.approach.town;
  let mut replay=Loader::new(world::catalog(),town.world_after.clone(),town.after).unwrap();
  for i in 0..adjustment.switches {replay.manual_time_switch(if i%2==0 {4}else{2},"test New Delsta adjustment").unwrap();}
  assert_eq!(replay.rng,adjustment.departure.after);
  assert_eq!(p.approach.agnea_story.arrival,adjustment.departure.after);
  assert_eq!(adjustment.input_cost,14+adjustment.switches*7+p.inputs.levels.len() as u32+route_cost(&p.crackridge_return)+if p.approach.highroad_day_switch.is_some() {7}else{0});
  let mut legs=vec![];let mut next=Some(&p.crackridge_return);
  while let Some(leg)=next {
   assert!(leg.defends.len()<=7);
   assert_eq!(leg.defend_rounds.len(),leg.defends.len());
   assert_eq!(leg.escape_round,if leg.defends.len()<4 {1}else{2});
   let w=&leg.arrival.world_after;
   let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
   let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
   let ids:Vec<_>=leg.encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
   let situation=if leg.encounter.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal};
   let mut menu=FieldEscapeMenu::with_step_ahead(leg.battle_entry,situation,&players,&levels,&ids,&w.enemy_knowledge).unwrap();
   for (i,report) in leg.defends.iter().enumerate() {
    assert_eq!(leg.defend_rounds[i],menu.round());
    let (after,replayed)=menu.safe_step_ahead_defend_through_next_round().unwrap().unwrap();
    assert_eq!(replayed.rng_before,report.rng_before);assert_eq!(replayed.rng_after,report.rng_after);
    menu=after;
   }
   assert_eq!(menu.rng,leg.menu.rng);
   if let Some(escape)=&leg.escape {
    assert_eq!(menu.attempt(&leg.inputs.levels,leg.encounter.encounter_level.unwrap()).unwrap().after,escape.after);
   }
   legs.push(leg.defends.len());next=leg.southern.as_deref();
  }
  assert_eq!(legs.len(),4);
  println!("New Delsta plan: {} switches; Defends per encounter {:?}; total cost {}",adjustment.switches,legs,adjustment.input_cost);
  let g=west.gravell.as_ref().unwrap();
  assert_eq!(g.fight.evaluation,ot2_battle_engine::gravell_routine::Evaluation::Accepted);
  assert_eq!(g.field_return.world_after.party,[7,6,8,4]);
  assert!(!g.field_return.world_after.active_scenarios.contains(&401));
  assert!(!g.field_return.world_after.flag(6076));
  assert!(!g.field_return.world_after.visited_maps.contains("Twn_Cty_2_1_A"));
  for (name,map) in [("Inventor Highroad","Fld_Cty_1_3"),("New Delsta","Twn_Cty_1_1_A"),("Crackridge Harbor","Fld_Wld_1_2"),("Crackridge","Twn_Wld_2_1_A")] {
   let mut l=Loader::new(world::catalog(),p.scope.world_after.clone(),p.scope.after).unwrap();
   if l.state.map!=map {l.visit_map(map).unwrap();}
   let mut draws=vec![];
   for _ in 0..8 {let before=l.rng;l.manual_time_switch(4,"benchmark night").unwrap();l.manual_time_switch(2,"benchmark day").unwrap();draws.push(l.rng.draws-before.draws);}
   println!("{name}: CRT draws per night/day pair {draws:?}");
  }
 }
 #[test]
 fn galdera_does_not_activate_agnea_chapter_two_or_consume_story_rng() {
  let fixture:serde_json::Value=serde_json::from_str(include_str!("../data/source/gravell-complete-route-fixture.json")).unwrap();
  let b=&fixture["before"];
  let mut entry=Entry {arrival:serde_json::from_value(b["arrival"].clone()).unwrap(),after:serde_json::from_value(b["after"].clone()).unwrap(),world_after:serde_json::from_value(b["world_after"].clone()).unwrap(),trace:vec![],decisions:vec![]};
  entry.world_after.galdera_route=true;
  entry.world_after.time=2;
  entry.world_after.active_scenarios.remove(&801);
  entry.world_after.confirmed_scenarios.remove(&801);
  entry.world_after.completed_scenarios.remove(&801);
  for flag in [6550,34167] {entry.world_after.set_flag(flag,false);}
  let result=approach(&entry).unwrap();
  assert_eq!(result.agnea_story.after,result.town.after);
  assert_eq!(result.agnea_story.world_after.main_progress_points,result.town.world_after.main_progress_points);
  for state in [&result.town.world_after,&result.agnea_story.world_after,&result.highroad.world_after] {
   assert!(!state.active_scenarios.contains(&801));
   assert!(!state.flag(6550));assert!(!state.flag(34167));
  }
  assert!(result.agnea_story.trace.iter().all(|t|t.before==t.after));
  assert_eq!(result.highroad.arrival,result.town.after);
  entry.world_after.galdera_route=false;
  let hikari=approach(&entry).unwrap();
  assert!(hikari.agnea_story.world_after.active_scenarios.contains(&801));
  assert!(hikari.agnea_story.world_after.flag(6550));
 }
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
  assert_eq!(g.town.arrival,g.field_return.after);
  assert_eq!(g.town.world_after.map,"Twn_Wld_3_1_A");
  assert_eq!(g.town.world_after.time,4);
  assert_eq!(serde_json::to_value(&g.town.world_after.character_progression).unwrap(),serde_json::to_value(&g.field_return.world_after.character_progression).unwrap());
  let mut replay=Loader::new(world::catalog(),g.field_return.world_after.clone(),g.field_return.after).unwrap();
  replay.visit_map("Twn_Wld_3_1_A").unwrap();
  assert_eq!(replay.rng,g.town.after);
 }
 #[test]
 fn coast_extension_freezes_exp100_and_replays_the_adjusted_finisher() {
  let fixture:serde_json::Value=serde_json::from_str(include_str!("../data/source/gravell-complete-route-fixture.json")).unwrap();
  let b=&fixture["before"];
  let entry=Entry {arrival:serde_json::from_value(b["arrival"].clone()).unwrap(),after:serde_json::from_value(b["after"].clone()).unwrap(),world_after:serde_json::from_value(b["world_after"].clone()).unwrap(),trace:vec![],decisions:vec![]};
  let mut g=gravell_arrival(&entry).unwrap().unwrap();
  let prefix=serde_json::to_value(&g.fight.actions[..g.fight.actions.len()-1]).unwrap();
  let reward=g.rewards.awarded;
  crate::galdera_coast::extend(&mut g).unwrap();
  assert_eq!(serde_json::to_value(&g.fight.actions[..g.fight.actions.len()-1]).unwrap(),prefix);
  assert_eq!(g.rewards.awarded,reward);
  let c=g.coast.as_ref().unwrap();
  assert_eq!(c.day_before_gravell.world_after.time,2);
  assert_eq!(g.town.world_after.time,2);
  assert!(g.town.world_after.visited_maps.contains("Twn_Wld_3_1_A"));
  assert_eq!(c.eastern_sai.world_after.map,"Fld_Dst_2_4");
  let s=&c.southern_sai;
  assert_eq!(s.arrival.arrival,c.eastern_sai_encounter.field_return.after);
  assert!(c.eastern_sai_encounter.escape.success);
  assert_eq!(c.eastern_sai_encounter.escape.chance_percent,100);
  assert_ne!(c.eastern_sai_encounter.encounter.situation.as_deref(),Some("Surprise"));
  assert_eq!(c.sai.world_after.map,"Twn_Dst_2_1_A");
  assert_eq!(c.sai.world_after.active_scenarios,s.field_return.world_after.active_scenarios);
  let supplies=c.oresrush_supplies.as_ref().unwrap();
  assert_eq!(supplies.warrior.after,c.sai.after);
  assert_eq!(supplies.warrior.world_after.job_licenses.get(&2),Some(&1));
  assert_eq!(supplies.hire.after.draws,supplies.night.after.draws+2);
  assert_eq!(supplies.pickaxe.after.draws,supplies.day.after.draws+2);
  assert!(supplies.steal_event.is_none());
  assert!(!supplies.pickaxe.trace.iter().any(|t|matches!(t.kind,world::DrawKind::DialogueLottery)));
  assert_eq!(supplies.steal_chance,100);
  assert_eq!(supplies.pickaxe.world_after.route_items.get("ITM_TRE_SS_TSn21_0200_0010"),Some(&1));
  let hired=supplies.pickaxe.world_after.hired_merchant.as_ref().unwrap();
  assert_eq!(hired.talent,"SPACT_MERCHANT_014");assert_eq!(hired.price,1600);
  assert!(!supplies.pickaxe.world_after.existing_npcs.iter().any(|id|world::catalog().placements[id].resource==hired.npc));
  let mut refreshed=Loader::new(world::catalog(),supplies.pickaxe.world_after.clone(),supplies.pickaxe.after).unwrap();
  refreshed.refresh_npcs("test hired follower persistence").unwrap();
  assert_eq!(refreshed.rng,supplies.pickaxe.after);
  assert!(matches!(s.encounter.situation.as_deref(),Some("Normal"|"Preemptive")));
  assert_eq!(s.escape.chance_percent,100);assert!(s.escape.success);
  assert_eq!(s.menu.party_levels,s.arrival.world_after.party.iter().map(|id|s.arrival.world_after.character_progression[id].level).collect::<Vec<_>>());
  assert_eq!(s.field_return.world_after.map,"Fld_Dst_2_3");
  assert_eq!(s.field_return.after.draws,s.escape.after.draws+2);
  assert_eq!(serde_json::to_value(&s.arrival.world_after.character_progression).unwrap(),serde_json::to_value(&s.field_return.world_after.character_progression).unwrap());
  assert_eq!(c.coast_exit.after,c.coast_day.as_ref().unwrap_or(&c.coast_entry).after);
  assert!(c.movement.estimated_counter_at_exit>0);
  assert_eq!(c.movement.initial_counter-c.movement.counter_cost,c.coast_exit.world_after.encounter_counter);
  let w=&g.night.world_after;
  let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
  let ids:Vec<_>=g.encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
  let situation=if g.encounter.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal};
  let battle=ot2_battle_engine::field_escape::carried_step_ahead(g.battle_entry,situation,&players,&ids,true,&w.enemy_knowledge).unwrap();
  let fight=ot2_battle_engine::gravell_routine::replay_finisher(battle,&g.fight,&g.fight.actions.last().unwrap().inputs).unwrap();
  let (fight,rewards,ret)=settle_gravell(&g.night,g.battle_entry,fight).unwrap();
  assert_eq!(fight.after,g.fight.after);
  assert_eq!(rewards.after,g.rewards.after);
  assert_eq!(ret.after,g.field_return.after);
 }
}

/// Establish a fast valid plan first, then spend a deterministic amount of
/// work improving its total input cost. A truncated refinement is not a claim
/// that an outcome is impossible: the already validated plan remains available.
fn search_clockbank_bg(entry:&Entry)->Result<(ClockbankAdjustment,HarborReturn)> {search_bg(entry,|departure,target|field_continuation(departure,0,Some(target)),route_cost)}
fn search_bg<T>(entry:&Entry,mut project:impl FnMut(&Entry,BgTarget<'_>)->Result<Option<T>>,cost:impl Fn(&T)->u32)->Result<(ClockbankAdjustment,T)> {
 let stride=if entry.world_after.galdera_route {1}else{2};
 let mut loader=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 let mut departures=vec![entry.clone()];
 let mut best:Option<(ClockbankAdjustment,T)>=None;
 let refinement_work=std::cell::Cell::new(5_000usize);
 for phase in 0..3 {
  if phase==2 {refinement_work.set(refinement_work.get()+5_000);}
  let mut scan=entry.after;if phase==0 {scan.advance(650);}
  loop {
   if phase>0 {
    if refinement_work.get()==0 {break;}
    let max_pairs=best.as_ref().unwrap().0.input_cost.saturating_sub(7)/(7*stride);
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
    let switch_cost=index as u32*7*stride;
    let local_budget=if phase==0 {400}else{192};
    let ceiling=best.as_ref().map_or(local_budget,|(a,_)|local_budget.min(a.input_cost.saturating_sub(switch_cost+1)));
    if ceiling<6 {continue;}
    let search=BgTarget {rng:target,budget:ceiling,cache:&cache,guard_extra:if phase==2 {12}else{0},
     fight_cap:if phase==0 {100}else{192},action_cap:if phase==0 {74}else{192},work:(phase>0).then_some(&refinement_work)};
    if let Some(route)=project(departure,search)? {
     let input_cost=switch_cost+cost(&route);
     if best.as_ref().is_none_or(|(b,_)|input_cost<b.input_cost) {
      best=Some((ClockbankAdjustment {switches:index as u32*stride,target,departure:departure.clone(),input_cost},route));
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
   let mut trials:Vec<_>=departures.iter().enumerate().filter(|(i,e)|e.after.draws<target.draws&&(*i as u32)*7*stride+6<best.as_ref().unwrap().0.input_cost).collect();
   trials.sort_by_key(|(_,e)|(target.draws-e.after.draws).abs_diff(preferred_gap));
   let cache=FightCache::default();
   for &(index,departure) in &trials {
    if adaptive_work==0 {break 'adaptive;}
    if target_work==0 {break;}
    let switch_cost=index as u32*7*stride;
    let ceiling=best.as_ref().unwrap().0.input_cost.saturating_sub(switch_cost+1);
    if ceiling<6 {continue;}
    let allowance=adaptive_work.min(target_work).min(6000);let work=std::cell::Cell::new(allowance);
    let search=BgTarget {rng:target,budget:ceiling,cache:&cache,guard_extra:12,fight_cap:ceiling as usize,action_cap:ceiling,work:Some(&work)};
    if let Some(route)=project(departure,search)? {
     let input_cost=switch_cost+cost(&route);
     if input_cost<best.as_ref().unwrap().0.input_cost {best=Some((ClockbankAdjustment {switches:index as u32*stride,target,departure:departure.clone(),input_cost},route));}
    }
    adaptive_work-=allowance-work.get();target_work-=allowance-work.get();
   }
  }
 }
 let (mut adjustment,route)=best.expect("baseline search returns only a valid plan");
 let mut replay=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 for i in 0..adjustment.switches {replay.manual_time_switch(if i%2==0 {4}else{2},if entry.world_after.galdera_route {"New Delsta: BG adjustment"} else {"Clockbank Industrial District: BG adjustment"})?;}
 ensure(replay.rng==adjustment.departure.after,"Clockbank switch replay diverged")?;
 adjustment.departure=point(&mut replay,entry.after);
 Ok((adjustment,route))
}
fn add_clockbank_cycle(loader:&mut Loader<'_>,departures:&mut Vec<Entry>)->Result<()> {
 let before=loader.rng;
 loader.manual_time_switch(if loader.state.time==2 {4}else{2},if loader.state.galdera_route {"New Delsta: BG time switch"} else {"Clockbank Industrial District: BG adjustment night"})?;
 if !loader.state.galdera_route {
  loader.manual_time_switch(2,"Clockbank Industrial District: BG adjustment day")?;
 }
 departures.push(point(loader,before));Ok(())
}

fn fight_input_cost(f:&ot2_battle_engine::gravell_routine::Fight)->u32 {
 f.actions.iter().filter(|a|matches!(a.action,ot2_battle_engine::gravell_routine::Action::Guards|ot2_battle_engine::gravell_routine::Action::BewilderingGrace)).map(|a|a.inputs.len() as u32).sum()
}

fn route_cost(r:&HarborReturn)->u32 {
 r.inputs.levels.len() as u32+r.defends.len() as u32*crate::escape_search::DEFEND_INPUT_COST+crate::escape_search::elapsed_round_cost(1,r.escape_round)+if r.victory.is_some(){10}else{0}
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
