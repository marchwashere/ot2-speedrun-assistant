//! Optional useful errands before the boat / Roque JP100 endpoint.
use crate::{Result,ensure,CrtRng,world::{self,Loader},snows_escape::Entry,galdera_coast::{self,SouthernSai,Movement},encounter::{self,PredictionOptions,Boundary}};
use serde::{Serialize,Deserialize};use std::{collections::BTreeMap,sync::OnceLock};
#[derive(Deserialize)]struct Item {id:String,level:u32}
#[derive(Deserialize)]struct Steal {npc:String,items:Vec<Item>}
#[derive(Deserialize)]struct Chest {id:String,item:String,count:u32}
#[derive(Deserialize)]struct Data {steals:BTreeMap<String,Steal>,scholar:String,scholar_placement:String,ambush_level:u32,ambush_before:Vec<String>,ambush_success:String,chests:Vec<Chest>,boat_npc:String}
fn data()->&'static Data {static D:OnceLock<Data>=OnceLock::new();D.get_or_init(||serde_json::from_str(include_str!("../data/galdera-flex.json")).unwrap())}
pub(crate) fn point(l:&mut Loader<'_>,before:CrtRng)->Entry {Entry {arrival:before,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}}
pub(crate) fn steal(l:&mut Loader<'_>,key:&str)->Result<()> {
 let d=&data().steals[key];ensure(l.state.time==2&&l.state.existing_npcs.iter().any(|id|l.data.placements[id].resource==d.npc),format!("{key} daytime NPC absent"))?;
 l.draw(world::DrawKind::PathActionMenu,&d.npc,"Open Steal menu: action timer",None);
 for item in &d.items {
  ensure(l.state.character_progression[&7].level>=item.level,"Steal not guaranteed")?;
  l.draw(world::DrawKind::StealCheck,&item.id,"Guaranteed Steal",Some(100));
  *l.state.route_items.entry(item.id.clone()).or_default()+=1;
 }
 l.boundary(format!("{key}: items stolen; switch path action and close, no dialogue lottery"));Ok(())
}
fn chest(l:&mut Loader<'_>,index:usize)->Result<()> {
 let c=&data().chests[index];ensure(l.state.opened_route_chests.insert(c.id.clone()),"Chest already opened")?;
 *l.state.route_items.entry(c.item.clone()).or_default()+=c.count;l.boundary(format!("Open {}: {}",c.id,c.item));Ok(())
}
#[derive(Debug,Serialize)]pub struct Segment {pub name:String,pub stages:Vec<(String,Entry)>,pub encounters:Vec<SouthernSai>,pub extra_switches:u32,pub escape_boost_inputs:Vec<u8>,pub movement:Option<Movement>}
impl Segment {
 fn new(name:&str)->Self {Self{name:name.into(),stages:vec![],encounters:vec![],extra_switches:0,escape_boost_inputs:vec![],movement:None}}
 fn save(&mut self,l:&mut Loader<'_>,label:&str,before:CrtRng){self.stages.push((label.into(),point(l,before)));}
 fn end(&self)->&Entry {&self.stages.last().unwrap().1}
}
fn visit(l:&mut Loader<'_>,s:&mut Segment,map:&str,label:&str)->Result<()> {let before=l.rng;l.visit_map(map)?;if map.starts_with("Twn_"){l.skipped_event("MapInStorySelect")?;}s.save(l,label,before);Ok(())}
fn book(entry:&Entry)->Result<Segment> {
 let mut s=Segment::new("Crackridge book");let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 visit(&mut l,&mut s,"Twn_Wld_2_1_A","Crackridge arrival")?;
 let before=l.rng;steal(&mut l,"mason")?;s.save(&mut l,"Mighty Leaf stolen; switch path action and close",before);
 let before=l.rng;l.manual_time_switch(4,"Night for Scholar Ambush")?;s.save(&mut l,"Crackridge night",before);
 let d=data();ensure(l.state.character_progression[&7].level>=d.ambush_level&&l.state.existing_npcs.iter().any(|id|l.data.placements[id].resource==d.scholar),"Scholar Ambush unavailable")?;
 let before=l.rng;l.draw(world::DrawKind::PathActionMenu,&d.scholar,"Ambush preview",None);let roll=l.draw(world::DrawKind::DialogueLottery,&d.scholar,"Ambush before dialogue",None);
 l.skipped_event(&d.ambush_before[usize::from(roll>16383)])?;l.state.swooned_route_npcs.insert(d.scholar_placement.clone());l.skipped_event(&d.ambush_success)?;
 s.save(&mut l,"Scholar ambushed",before);let before=l.rng;l.manual_time_switch(2,"Day immediately after Scholar Ambush")?;chest(&mut l,0)?;s.save(&mut l,"Day; From the Far Reaches of Hell chest",before);Ok(s)
}
fn firestarter(entry:&Entry)->Result<Segment> {
 for pairs in 0..1000 {
  let mut s=Segment::new("Sai / Firestarter");let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
  visit(&mut l,&mut s,"Twn_Dst_2_1_A","Sai arrival")?;
  if pairs>0 {let before=l.rng;for _ in 0..pairs {l.manual_time_switch(4,"Sai anti-ambush adjustment")?;l.manual_time_switch(2,"Sai anti-ambush adjustment")?;}s.extra_switches=pairs*2;s.save(&mut l,"Sai extra switches",before);}
  visit(&mut l,&mut s,"Fld_Dst_2_3","Southern Sai; no encounter")?;
  visit(&mut l,&mut s,"Fld_Dst_2_2","Western Sai")?;
  let Some(fight)=galdera_coast::sai_encounter(s.end().clone(),"ENT_FLD_DST_2_2_LND")? else {continue;};
  let mut l=Loader::new(world::catalog(),fight.field_return.world_after.clone(),fight.field_return.after)?;
  let before=l.rng;chest(&mut l,1)?;s.save(&mut l,"Reinforcing Jam chest",before);s.encounters.push(fight);
  visit(&mut l,&mut s,"Fld_Dst_3_2","Eastern Ku Sands; no encounter")?;
  let before=l.rng;chest(&mut l,2)?;s.save(&mut l,"Firestarter chest",before);return Ok(s);
 }
 Err(crate::SimError("No safe Western Sai opening".into()))
}
fn remaining(counter:u32,first:u32)->Result<Option<Movement>> {
 ensure(first<203,"Beasting encounter occurs past the measured crossing")?;
 let run=203-first+3;let walk=(217*(203-first)).div_ceil(203)+3;
 let mut o=PredictionOptions{zone:"ENT_FLD_ISD_1_3_LND".into(),period:"DayTime".into(),ability_percent:50,dash:true,..Default::default()};
 let rd=encounter::catalog().counter_decrement(&o)?;o.dash=false;let wd=encounter::catalog().counter_decrement(&o)?;
 for fast in (0..=run).rev(){let slow=(walk*(run-fast)).div_ceil(run);let cost=fast*rd+slow*wd;if cost<counter {return Ok(Some(Movement{initial_counter:counter,walking_updates:slow,running_updates:fast,walk_decrement:wd,run_decrement:rd,run_percent:(fast*1000/run)as f64/10.,switch_to_walking_counter:counter-fast*rd,counter_cost:cost,estimated_counter_at_exit:counter-cost}));}}
 Ok(None)
}
fn to_tropu(entry:&Entry)->Result<Segment> {
 for pairs in 0..1000 {
  let mut s=Segment::new("Boat approach");let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
  if pairs>0 {let before=l.rng;for _ in 0..pairs {l.manual_time_switch(4,"Departure anti-ambush adjustment")?;l.manual_time_switch(2,"Departure anti-ambush adjustment")?;}s.extra_switches=pairs*2;s.save(&mut l,"Departure extra switches",before);}
  visit(&mut l,&mut s,"Fld_Isd_1_3","Beasting Bay: Anchorage")?;
  let Some(mut first)=galdera_coast::sai_encounter(s.end().clone(),"ENT_FLD_ISD_1_3_LND")? else {continue;};
  let first_updates=first.encounter.counter_updates.unwrap();
  for cycles in 0..1000 {
   let inputs:Vec<_>=(0..cycles).flat_map(|_|[1,0]).collect();let escape=first.menu.clone().attempt(&inputs,first.encounter.encounter_level.unwrap())?;
   ensure(escape.success&&escape.chance_percent==100,"Beasting Escape not guaranteed")?;
   let ret=crate::highroad::restore_field(&first.arrival.world_after,&escape,"Fld_Isd_1_3","Beasting Escape")?;
   let Some(m)=remaining(ret.world_after.encounter_counter,first_updates)? else {continue;};
   let mut l=Loader::new(world::catalog(),ret.world_after.clone(),ret.after)?;
   l.state.encounter_counter=m.estimated_counter_at_exit;l.boundary(format!("After first encounter run {:.1}% then walk; 203 run /217 walk total, +3 leeway",m.run_percent));
   let exit=point(&mut l,ret.after);l.visit_map("Fld_Isd_2_1")?;let arrival=point(&mut l,exit.after);
   let Some(second)=galdera_coast::sai_encounter(arrival,"ENT_FLD_ISD_2_1_LND")? else {continue;};
   first.escape=escape;first.field_return=Entry{arrival:ret.arrival,after:ret.after,world_after:ret.world_after,trace:ret.trace,decisions:vec![]};
   s.escape_boost_inputs=inputs;s.movement=Some(m);s.stages.push(("Beasting crossing complete".into(),exit));
   let mut l=Loader::new(world::catalog(),second.field_return.world_after.clone(),second.field_return.after)?;
   s.encounters.extend([first,second]);visit(&mut l,&mut s,"Twn_Isd_2_1_A","Tropu’hopu arrival")?;return Ok(s);
  }
 }
 Err(crate::SimError("No safe Beasting / Tropu approach".into()))
}
fn boat(entry:&Entry,switches:u32)->Result<Segment> {
 let mut s=Segment::new("Boat and Roque Harbor");let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 let before=l.rng;for _ in 0..switches {l.manual_time_switch(if l.state.time==2 {4}else{2},"Tropu’hopu JP100 coarse adjustment")?;}s.extra_switches=switches;s.save(&mut l,"Tropu’hopu time adjustment",before);
 visit(&mut l,&mut s,"Twn_Isd_2_1_C","Tropu’hopu Shipyard")?;
 let before=l.rng;l.skipped_event("MS_SHO_EX1_0100")?;s.save(&mut l,"Shipyard cutscene",before);
 let before=l.rng;l.skipped_event("MS_SHO_EX1_0110")?;s.save(&mut l,"Shipbuilder cutscene",before);
 let before=l.rng;l.draw(world::DrawKind::PathActionMenu,&data().boat_npc,"Open Purchase menu",None);
 *l.state.route_items.entry("ITM_TRE_SHO_EX1_0030".into()).or_default()+=1;
 // Purchase invokes the placement's eFC_PURCHASE_ITEM event immediately.
 let choices=world::EventChoices::from([(("MS_SHO_EX1_0600".into(),12),"ED_SHO_EX1_0600_0040".into()),(("MS_SHO_EX1_0600".into(),94),"eDIALOG_YES".into())]);
 l.skipped_event_with_choices("MS_SHO_EX1_0400",&choices)?;
 ensure(l.state.map=="Fld_Ocn_1_1"||l.data.maps[&l.state.map].id==189,"Boat event did not reach sea")?;
 s.save(&mut l,"Boat purchased for 100,000; no decoration; at sea",before);
 visit(&mut l,&mut s,"Fld_Sea_3_1","Roque Island: Anchorage; zero sea encounters")?;
 let before=l.rng;if l.state.time!=4 {l.manual_time_switch(4,"Night immediately upon entering Roque Harbor")?;}s.save(&mut l,"Roque Harbor night",before);Ok(s)
}
#[derive(Debug,Serialize)]pub struct Trip {pub order:Vec<u8>,pub segments:Vec<Segment>,pub boat:Segment,pub encounter:encounter::Prediction,pub battle_entry:CrtRng,pub fight:ot2_battle_engine::gravell_routine::Fight,pub rewards:ot2_battle_engine::gravell_routine::Rewards,pub field_return:Entry,pub extra_input_cost:u32,pub remaining_errands:Vec<u8>}
pub(crate) fn finish(entry:&Entry)->Result<Trip> {
 ensure(entry.world_after.character_progression[&5].jobs.learned_abilities[&5].contains("ABI_SET_DAN_070")
  &&entry.world_after.character_progression[&5].jobs.equipped_support.contains(crate::gravell_setup::STEP_AHEAD)
  &&entry.world_after.character_progression[&4].jobs.equipped_support.contains("ABI_MER_SUP_02")
  &&entry.world_after.route_items.get("ITM_CSM_0070").copied().unwrap_or(0)>0,"Gravell JP setup or small Pom missing")?;
 let orders:Vec<Vec<u8>>=vec![vec![],vec![2],vec![3],vec![2,3],vec![3,2]];
 let mut prefixes=vec![];
 for order in orders {
  let mut stages=vec![];let mut current=entry.clone();
  for &n in &order {let s=if n==2 {book(&current)?}else{firestarter(&current)?};current=s.end().clone();stages.push(s);}
  let approach=to_tropu(&current)?;stages.push(approach);prefixes.push((order,stages));
 }
 // Progressive coarse switches and fine boost windows; full battle execution
 // is the acceptance condition. No finite bound declares JP100 impossible.
 let mut best:Option<(usize,Trip)>=None;
 for switches in 0.. {
  if best.as_ref().is_some_and(|(_,b)|switches*7+6>b.extra_input_cost){break;}
  for (index,(order,segments)) in prefixes.iter().enumerate() {
   let overhead: u32=segments.iter().map(|s|s.extra_switches*7+s.escape_boost_inputs.len()as u32).sum();
   let cost=switches*7+overhead;
   if best.as_ref().is_some_and(|(_,b)|cost+6>b.extra_input_cost){continue;}
   let boat=boat(segments.last().unwrap().end(),switches)?;let e=boat.end();let w=&e.world_after;let mut rng=e.after;
   let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions{map:w.map.clone(),zone:"ENT_FLD_SEA_3_1_LND".into(),period:"Night".into(),progress:w.main_progress_points,first_encounter:w.first_encounter,boundary:Boundary::CurrentCounter,current_counter:Some(w.encounter_counter),party_levels:w.party.iter().map(|id|w.character_progression[id].level).collect(),ability_percent:50,dash:true,..Default::default()})?;
   if encounter.enemies.len()!=2||encounter.enemies.iter().any(|e|e.enemy.id!="ENE_NML_OCN_230") {continue;}
   let situation=match encounter.situation.as_deref(){Some("Normal")=>ot2_battle_engine::escape::Situation::Normal,Some("Preemptive")=>ot2_battle_engine::escape::Situation::Preemptive,_=>continue};
   let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
   let b=ot2_battle_engine::field_escape::roque_jp_battle(rng,situation,&players,&w.enemy_knowledge)?;
   let budget=best.as_ref().map_or(96,|(_,b)|b.extra_input_cost.saturating_sub(cost).min(96))as usize;
   let Some(f)=ot2_battle_engine::roque_routine::search(b,budget)? else {continue;};
   let input_cost=cost+f.actions.iter().map(|a|a.inputs.len()as u32).sum::<u32>();
   let (fight,rewards,mut ret)=crate::inventor::settle_gravell(e,rng,f)?;
   if let Some(n)=ret.world_after.route_items.get_mut("ITM_CSM_0070"){ensure(*n>0,"Missing small Pom")?;*n-=1;if *n==0{ret.world_after.route_items.remove("ITM_CSM_0070");}}else{return Err(crate::SimError("Missing small Pom".into()));}
   for (item,n) in &rewards.drops {*ret.world_after.route_items.entry(item.clone()).or_default()+=n;}
   best=Some((index,Trip{order:order.iter().copied().chain([1]).collect(),segments:vec![],boat,encounter,battle_entry:rng,fight,rewards,field_return:ret,extra_input_cost:input_cost,remaining_errands:[2,3].into_iter().filter(|n|!order.contains(n)).collect()}));
  }
 }
 let (index,mut b)=best.unwrap();b.segments=prefixes.swap_remove(index).1;Ok(b)
}

#[cfg(test)]mod tests {
 use super::*;
 fn fixture()->Entry {
  let v:serde_json::Value=serde_json::from_str(include_str!("../data/source/galdera-flex-entry.json")).unwrap();
  let rng: CrtRng=serde_json::from_value(v["after"].clone()).unwrap();
  let mut world_after:world::WorldState=serde_json::from_value(v["world_after"].clone()).unwrap();world_after.refresh_support_skills();
  Entry{arrival:rng,after:rng,world_after,trace:vec![],decisions:vec![]}
 }
 #[test]fn both_errand_orders_keep_inventory_and_daytime_without_repeating_pickups() {
  for order in [[2,3],[3,2]] {
   let mut e=fixture();
   for n in order {e=if n==2{book(&e).unwrap()}else{firestarter(&e).unwrap()}.end().clone();}
   assert_eq!(e.world_after.time,2);
   for c in &data().chests {assert_eq!(e.world_after.route_items[&c.item],fixture().world_after.route_items.get(&c.item).copied().unwrap_or(0)+1);}
   assert_eq!(e.world_after.route_items["ITM_MRL_REV_0040"],1);
   assert!(e.world_after.swooned_route_npcs.contains(&data().scholar_placement));
  }
 }
 #[test]fn boat_events_and_tropu_switches_preserve_native_rng_and_end_at_night_harbor() {
  let s=to_tropu(&fixture()).unwrap();assert_eq!(s.encounters.len(),2);assert!(s.movement.as_ref().unwrap().estimated_counter_at_exit>0);
  for switches in 0..=2 {
   let b=boat(s.end(),switches).unwrap();let e=b.end();assert_eq!(e.world_after.map,"Fld_Sea_3_1");assert_eq!(e.world_after.time,4);
   assert!(e.world_after.flag(6208));assert!(!e.world_after.route_items.contains_key("ITM_TRE_SHO_EX1_0030"));
   assert_eq!(e.world_after.route_items["ITM_TRE_SHO_EX1_0010"],1);
   let adjustment=&b.stages[0].1;eprintln!("Tropu switches {switches}: {} CRT draws",adjustment.after.draws-adjustment.arrival.draws);
  }
 }
 #[test]fn complete_jp100_search_requires_the_full_four_alive_guards_victory() {
  let r=finish(&fixture()).unwrap();assert_eq!(r.rewards.base.jp,116);assert_eq!(r.rewards.awarded.jp,12760);
  assert_eq!(r.fight.state.outcome,ot2_battle_engine::Outcome::Victory);assert!(r.fight.state.actors[..4].iter().all(|a|a.alive()));
  assert_eq!(r.fight.actions.len(),6);assert_eq!(r.fight.actions[4].actor.0,2);assert_eq!(r.fight.actions[5].actor.0,3);
  assert_eq!(r.fight.state.grace.as_ref().unwrap().jp_percent(),10000);
  assert!(!r.field_return.world_after.route_items.contains_key("ITM_CSM_0070"));
  for id in [7,6,5,4] {assert_eq!(r.field_return.world_after.character_progression[&id].jp_earned-fixture().world_after.character_progression[&id].jp_earned,12760);}
 }
 #[test]#[ignore="Expanded synthetic RNG-state search"]fn alternate_rng_states_find_complete_jp_routes() {
  for seed in [0,1,12345678] {
   let mut e=fixture();e.after=CrtRng::new(seed);e.arrival=e.after;
   let r=finish(&e).unwrap();assert_eq!(r.rewards.awarded.jp,12760);
   eprintln!("Seed {seed}: order {:?}, switches {}, cost {}",r.order,r.boat.extra_switches,r.extra_input_cost);
  }
 }
}
