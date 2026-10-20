//! Optional useful errands before the boat / Roque JP100 endpoint.
use crate::{Result,ensure,CrtRng,world::{self,Loader},snows_escape::Entry,galdera_coast::{self,SouthernSai,Movement},encounter::{self,PredictionOptions,Boundary}};
use serde::{Serialize,Deserialize};use std::{collections::BTreeMap,sync::OnceLock};
#[derive(Deserialize)]struct Item {id:String,level:u32}
#[derive(Deserialize)]struct Steal {npc:String,items:Vec<Item>}
#[derive(Deserialize)]struct Chest {id:String,item:String,count:u32}
#[derive(Deserialize)]struct Data {steals:BTreeMap<String,Steal>,scholar:String,scholar_placement:String,ambush_level:u32,ambush_before:Vec<String>,ambush_success:String,chests:Vec<Chest>,boat_npc:String}
const TIME_SWITCH_COST:u32=12;
fn tropu_reload_cost(count:u32)->u32 {count.saturating_sub(3)*TIME_SWITCH_COST}
fn roque_reload_cost(count:u32)->u32 {count.saturating_sub(4)*TIME_SWITCH_COST}
const ERRAND_ORDER:[u8;0]=[];
const ROUTE_ORDER:[u8;1]=[1];
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
#[derive(Debug,Clone,Serialize)]pub struct Segment {pub name:String,pub stages:Vec<(String,Entry)>,pub encounters:Vec<SouthernSai>,pub extra_switches:u32,pub escape_boost_inputs:Vec<u8>,pub movement:Option<Movement>}
impl Segment {
 pub(crate) fn new(name:&str)->Self {Self{name:name.into(),stages:vec![],encounters:vec![],extra_switches:0,escape_boost_inputs:vec![],movement:None}}
 pub(crate) fn save(&mut self,l:&mut Loader<'_>,label:&str,before:CrtRng){self.stages.push((label.into(),point(l,before)));}
 pub(crate) fn end(&self)->&Entry {&self.stages.last().unwrap().1}
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
pub(crate) fn before_gravell(entry:&Entry)->Result<Vec<Segment>> {
 let fire=firestarter(entry)?;
 let book=book(fire.end())?;
 Ok(vec![fire,book])
}
fn firestarter(entry:&Entry)->Result<Segment> {firestarter_defends(entry,0)}
fn firestarter_defends(entry:&Entry,defends:u8)->Result<Segment> {
 for pairs in 0..1000 {
  let mut s=Segment::new("Sai / Firestarter");let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
  visit(&mut l,&mut s,"Twn_Dst_2_1_A","Sai arrival")?;
  if pairs>0 {let before=l.rng;for _ in 0..pairs {l.manual_time_switch(4,"Sai anti-ambush adjustment")?;l.manual_time_switch(2,"Sai anti-ambush adjustment")?;}s.extra_switches=pairs*2;s.save(&mut l,"Sai extra switches",before);}
  visit(&mut l,&mut s,"Fld_Dst_2_3","Southern Sai; no encounter")?;
  visit(&mut l,&mut s,"Fld_Dst_2_2","Western Sai")?;
  let Some(fight)=galdera_coast::sai_encounter_defends(s.end().clone(),"ENT_FLD_DST_2_2_LND",defends)? else {continue;};
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
fn to_tropu(entry:&Entry)->Result<Segment> {to_tropu_defends(entry,0,0)}
fn to_tropu_defends(entry:&Entry,beasting:u8,tropu:u8)->Result<Segment> {
 for pairs in 0..1000 {
  let mut s=Segment::new("Boat approach");let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
  if pairs>0 {let before=l.rng;for _ in 0..pairs {l.manual_time_switch(4,"Departure anti-ambush adjustment")?;l.manual_time_switch(2,"Departure anti-ambush adjustment")?;}s.extra_switches=pairs*2;s.save(&mut l,"Departure extra switches",before);}
  visit(&mut l,&mut s,"Fld_Isd_1_3","Beasting Bay: Anchorage")?;
  let Some(mut first)=galdera_coast::sai_encounter_defends(s.end().clone(),"ENT_FLD_ISD_1_3_LND",beasting)? else {continue;};
  let first_updates=first.encounter.counter_updates.unwrap();
  for cycles in 0..1000 {
   let inputs:Vec<_>=(0..cycles).flat_map(|_|[1,0]).collect();let escape=first.menu.clone().attempt(&inputs,first.encounter.encounter_level.unwrap())?;
   ensure(escape.success&&escape.chance_percent==100,"Beasting Escape not guaranteed")?;
   let ret=crate::highroad::restore_field(&first.arrival.world_after,&escape,"Fld_Isd_1_3","Beasting Escape")?;
   let Some(m)=remaining(ret.world_after.encounter_counter,first_updates)? else {continue;};
   let mut l=Loader::new(world::catalog(),ret.world_after.clone(),ret.after)?;
   l.state.encounter_counter=m.estimated_counter_at_exit;l.boundary(format!("After first encounter run {:.1}% then walk; 203 run /217 walk total, +3 leeway",m.run_percent));
   let exit=point(&mut l,ret.after);l.visit_map("Fld_Isd_2_1")?;let arrival=point(&mut l,exit.after);
   let Some(second)=galdera_coast::sai_encounter_defends(arrival,"ENT_FLD_ISD_2_1_LND",tropu)? else {continue;};
   first.escape=escape;first.field_return=Entry{arrival:ret.arrival,after:ret.after,world_after:ret.world_after,trace:ret.trace,decisions:vec![]};
   s.escape_boost_inputs=inputs;s.movement=Some(m);s.stages.push(("Beasting crossing complete".into(),exit));
   let mut l=Loader::new(world::catalog(),second.field_return.world_after.clone(),second.field_return.after)?;
   s.encounters.extend([first,second]);visit(&mut l,&mut s,"Twn_Isd_2_1_A","Tropu’hopu arrival")?;return Ok(s);
  }
 }
 Err(crate::SimError("No safe Beasting / Tropu approach".into()))
}
fn boat(entry:&Entry,switches:u32)->Result<Segment> {boat_period(entry,switches,true)}
fn boat_period(entry:&Entry,switches:u32,night:bool)->Result<Segment> {
 ensure(entry.world_after.time==2&&switches%2==0,"Ship Purchase requires daytime; Tropu switches must finish during day")?;
 let mut s=Segment::new("Boat and Roque Harbor");let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 let before=l.rng;for _ in 0..switches {l.manual_time_switch(if l.state.time==2 {4}else{2},"Tropu’hopu JP100 coarse adjustment")?;}s.extra_switches=switches;s.save(&mut l,"Tropu’hopu time adjustment",before);
 visit(&mut l,&mut s,"Twn_Isd_2_1_C","Tropu’hopu Shipyard")?;
 let before=l.rng;l.skipped_event("MS_SHO_EX1_0100")?;s.save(&mut l,"Shipyard cutscene",before);
 let before=l.rng;l.skipped_event("MS_SHO_EX1_0110")?;s.save(&mut l,"Shipbuilder cutscene",before);
 ensure(l.state.time==2&&l.state.party.contains(&4)&&l.state.existing_npcs.iter().any(|id|l.data.placements[id].resource==data().boat_npc),"Ship Purchase requires daytime, Partitio and the Shipbuilder")?;
 let before=l.rng;l.draw(world::DrawKind::PathActionMenu,&data().boat_npc,"Open Purchase menu",None);
 *l.state.route_items.entry("ITM_TRE_SHO_EX1_0030".into()).or_default()+=1;
 // Purchase invokes the placement's eFC_PURCHASE_ITEM event immediately.
 let choices=world::EventChoices::from([(("MS_SHO_EX1_0600".into(),12),"ED_SHO_EX1_0600_0040".into()),(("MS_SHO_EX1_0600".into(),94),"eDIALOG_YES".into())]);
 l.skipped_event_with_choices("MS_SHO_EX1_0400",&choices)?;
 ensure(l.state.map=="Fld_Ocn_1_1"||l.data.maps[&l.state.map].id==189,"Boat event did not reach sea")?;
 s.save(&mut l,"Boat purchased for 100,000; no decoration; at sea",before);
 visit(&mut l,&mut s,"Fld_Sea_3_1","Roque Island: Anchorage; zero sea encounters")?;
 if night {let before=l.rng;if l.state.time!=4 {l.manual_time_switch(4,"Night immediately upon entering Roque Harbor")?;}s.save(&mut l,"Roque Harbor night",before);}
 else {ensure(l.state.time==2,"Deferred JP route must reach Roque Harbor during daytime")?;}
 Ok(s)
}

/// Inactive candidate for the future automatic JP-location search. It deliberately
/// does not call the Roque JP routine or claim JP100 has been obtained.
#[derive(Debug,Serialize)]
pub struct DeferredRoque {
 pub approach:Segment,pub boat:Segment,pub harbor:SouthernSai,pub town:Segment,
 pub field_return:Entry,pub chest_money:u32,pub shopping_cost:u32,
 pub jp100_complete:bool,
}
pub fn deferred_roque(entry:&Entry,switches:u32)->Result<Option<DeferredRoque>> {deferred_roque_defends(entry,switches,[0;5])}
fn deferred_roque_defends(entry:&Entry,switches:u32,c:[u8;5])->Result<Option<DeferredRoque>> {
 deferred_roque_reloads(entry,switches,c,1)
}
fn deferred_roque_reloads(entry:&Entry,switches:u32,c:[u8;5],reloads:u32)->Result<Option<DeferredRoque>> {
 ensure(entry.world_after.time==2&&switches%2==0,"Daytime Roque branch requires an even number of Tropu switches")?;
 let approach=to_tropu_defends(entry,c[1],c[2])?;
 let boat=boat_period(approach.end(),switches,false)?;
 let Some(harbor)=galdera_coast::sai_encounter_defends(boat.end().clone(),"ENT_FLD_SEA_3_1_LND",c[3])? else {return Ok(None);};
 let town=if reloads==1 {crate::galdera_roque::town(&harbor.field_return)?}else{crate::galdera_roque::town_with_reloads(&harbor.field_return,reloads)?};
 let field_return=town.end().clone();
 Ok(Some(DeferredRoque {approach,boat,harbor,town,field_return,chest_money:39800,shopping_cost:42000,jp100_complete:false}))
}
#[derive(Debug,Serialize)]pub struct Trip {pub order:Vec<u8>,pub segments:Vec<Segment>,pub boat:Segment,pub encounter:encounter::Prediction,pub battle_entry:CrtRng,pub fight:ot2_battle_engine::gravell_routine::Fight,pub rewards:ot2_battle_engine::gravell_routine::Rewards,pub field_return:Entry,pub extra_input_cost:u32,pub remaining_errands:Vec<u8>,pub location:String,pub continuation:Vec<Segment>,pub nameless:Option<crate::galdera_nameless::Trip>,pub nameless_jp:Option<crate::galdera_nameless::JpTrip>,pub shipwreck:Option<crate::galdera_shipwreck::Trip>,pub encounter_arrival:Option<Entry>,pub downstream_draws:u64,pub defend_counts:[u8;5]}
fn finish_roque(entry:&Entry)->Result<Trip> {
 ensure(crate::galdera_jp::can_fund_with(&entry.world_after,12760)?,"Roque JP100 cannot fund the planned skill purchases")?;
 ensure(entry.world_after.character_progression[&5].jobs.learned_abilities[&5].contains("ABI_SET_DAN_070")
  &&entry.world_after.character_progression[&5].jobs.equipped_support.contains(crate::gravell_setup::STEP_AHEAD)
  &&entry.world_after.character_progression[&6].character.boost_start
  &&entry.world_after.character_progression[&4].jobs.learned_abilities.get(&0).is_some_and(|a|a.contains("ABI_SET_MER_050")),"Gravell JP setup or Donate BP missing")?;
 let orders:Vec<Vec<u8>>=vec![ERRAND_ORDER.to_vec()];
 let mut prefixes=vec![];
 for order in orders {
  let mut stages=vec![];let mut current=entry.clone();
  for &n in &order {let s=if n==2 {book(&current)?}else{firestarter(&current)?};current=s.end().clone();stages.push(s);}
  let approach=to_tropu(&current)?;stages.push(approach);prefixes.push((order,stages));
 }
 // Progressive coarse switches and fine boost windows; full battle execution
 // is the acceptance condition. No finite bound declares JP100 impossible.
 let mut best:Option<(usize,Trip)>=None;
 for switches in (0u32..).step_by(2) {
  if best.as_ref().is_some_and(|(_,b)|tropu_reload_cost(switches)+5>b.extra_input_cost){break;}
  for (index,(order,segments)) in prefixes.iter().enumerate() {
   let overhead: u32=segments.iter().map(segment_cost).sum();
   let cost=tropu_reload_cost(switches)+overhead;
   if best.as_ref().is_some_and(|(_,b)|cost+5>b.extra_input_cost){continue;}
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
   if !crate::galdera_jp::sufficient(&ret.world_after)? {continue;}
   for (item,n) in &rewards.drops {*ret.world_after.route_items.entry(item.clone()).or_default()+=n;}
   let downstream_draws=rng.draws-boat.stages[0].1.after.draws;
   best=Some((index,Trip{order:order.iter().copied().chain([1]).collect(),segments:vec![],boat,encounter,battle_entry:rng,fight,rewards,field_return:ret,extra_input_cost:input_cost,remaining_errands:vec![],location:"Roque Island Harbor".into(),continuation:vec![],nameless:None,nameless_jp:None,shipwreck:None,encounter_arrival:None,downstream_draws,defend_counts:[0;5]}));
  }
 }
 let (index,mut b)=best.unwrap();b.segments=prefixes.swap_remove(index).1;Ok(b)
}

// Fixed prerequisites: Firestarter, Crackridge book, then boat/Roque.
const LATER_ORDERS:[[u8;1];1]=[ROUTE_ORDER];
fn defend_cost(f:&SouthernSai)->u32 {f.defends.len()as u32*3+f.escape_round.saturating_sub(1)*8}
fn segment_cost(s:&Segment)->u32 {(if s.name=="Boat and Roque Harbor" {tropu_reload_cost(s.extra_switches)} else {s.extra_switches*TIME_SWITCH_COST})+s.escape_boost_inputs.len()as u32+s.encounters.iter().map(defend_cost).sum::<u32>()}
fn day(entry:&Entry)->Result<Segment> {
 let mut s=Segment::new("Day for remaining errands");let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 if l.state.time!=2 {l.manual_time_switch(2,"Day before remaining errands / Nameless Isle")?;}
 s.save(&mut l,"Daytime departure",entry.after);Ok(s)
}
fn nameless_arrival(entry:&Entry)->Result<Segment> {
 let mut s=day(entry)?;s.name="Beasting Bay to Nameless Isle".into();
 let mut l=Loader::new(world::catalog(),s.end().world_after.clone(),s.end().after)?;
 visit(&mut l,&mut s,"Fld_Isd_1_3","Teleport Beasting Bay: Anchorage")?;
 visit(&mut l,&mut s,"Fld_Ocn_1_1","Board owned boat")?;
 visit(&mut l,&mut s,crate::galdera_nameless::MAP,"Nameless Isle entry; no sea encounters")?;Ok(s)
}
/// Compare complete replays, not a fixed guessed offset from Tropu to BG.
/// Required errands have fixed order; compare JP locations and adjustment costs.
fn finish_without_defends(entry:&Entry)->Result<Trip> {
 let mut best=finish_roque(entry)?;
 for switches in (0u32..).step_by(2) {
  if tropu_reload_cost(switches)+4>=best.extra_input_cost {break;}
  for order in LATER_ORDERS {
   for ship in [false,true] {
   let Some(candidate)=later_candidate(entry,order,switches,best.extra_input_cost,ship)?else{continue;};
   if candidate.extra_input_cost<best.extra_input_cost {best=candidate;}
   }
  }
 }
 Ok(best)
}
pub(crate) fn finish(entry:&Entry)->Result<Trip> {
 // Scan targets before selecting coarse switches. Exact replay, not the
 // estimate, accepts a route. Keep the former search as a quality fallback.
 let targets=crate::bg_target::targets(entry.after,20_000,"ABI_DAN_MAKA_001")?;
 let proposed=target_first(entry,&targets,1024)?;
 let best=if crate::galdera_jp::can_fund_with(&entry.world_after,12760)? {
  let fallback=finish_without_defends(entry)?;
  match proposed {Some(p) if p.extra_input_cost<fallback.extra_input_cost=>p,_=>fallback}
 }else{
  proposed.ok_or_else(||crate::SimError("No JP100 candidate funds the planned skills within the search range".into()))?
 };
 refine_defends(entry,best)
}
fn defend_curve(f:&SouthernSai)->Result<crate::bg_target::Curve>{
 let situation=if f.encounter.situation.as_deref()==Some("Preemptive"){ot2_battle_engine::escape::Situation::Preemptive}else{ot2_battle_engine::escape::Situation::Normal};
 let ids:Vec<_>=f.encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
 crate::bg_target::curve(f.battle_entry,situation,&f.arrival.world_after,&ids)
}
fn roque_probe(e:&Entry)->Result<Option<CrtRng>> {
 let w=&e.world_after;let mut rng=e.after;
 let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions{map:w.map.clone(),zone:"ENT_FLD_SEA_3_1_LND".into(),period:"Night".into(),progress:w.main_progress_points,first_encounter:w.first_encounter,boundary:Boundary::CurrentCounter,current_counter:Some(w.encounter_counter),party_levels:w.party.iter().map(|id|w.character_progression[id].level).collect(),ability_percent:50,dash:true,..Default::default()})?;
 if encounter.enemies.len()!=2||encounter.enemies.iter().any(|e|e.enemy.id!="ENE_NML_OCN_230"){return Ok(None);}
 let situation=match encounter.situation.as_deref(){Some("Normal")=>ot2_battle_engine::escape::Situation::Normal,Some("Preemptive")=>ot2_battle_engine::escape::Situation::Preemptive,_=>return Ok(None)};
 let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
 let b=ot2_battle_engine::field_escape::roque_jp_battle(rng,situation,&players,&w.enemy_knowledge)?;
 Ok(ot2_battle_engine::roque_routine::prefix(b)?.map(|(b,_)|CrtRng{state:b.state().rng.state,draws:rng.draws+b.state().rng.draws}))
}
/// A location-specific forward probe stops at the BG command menu. It does not
/// need a JP100 roll and never awards rewards or publishes a speculative plan.
fn jp_estimate(entry:&Entry,location:u8,switches:u32)->Result<Option<(u64,Vec<crate::bg_target::Curve>,Entry)>> {
 let mut disabled=[u64::MAX;8];disabled[0]=0;
 let mut curves=vec![disabled;5];
 let (menu,approach,boat)=if location==0 {
  let approach=to_tropu(entry)?;let boat=boat(approach.end(),switches)?;
  let Some(menu)=roque_probe(boat.end())?else{return Ok(None);};(menu,approach,boat)
 }else{
  let Some(r)=deferred_roque_reloads(entry,switches,[0;5],1)?else{return Ok(None);};
  curves[3]=defend_curve(&r.harbor)?;
  let island=nameless_arrival(&r.field_return)?;
  let menu=if location==1 {crate::galdera_nameless::jp_probe(island.end())?}else{
   let Some(n)=crate::galdera_nameless::finish_with_defends(island.end(),48,0)?else{return Ok(None);};
   curves[4]=defend_curve(&n.approach.encounter)?;
   crate::galdera_shipwreck::jp_probe(n.loot.end())?
  };
  let Some(menu)=menu else{return Ok(None);};(menu,r.approach,r.boat)
 };
 for (i,f) in approach.encounters.iter().enumerate(){curves[i+1]=defend_curve(f)?;}
 let departure=approach.end().clone();
 let coarse=boat.stages[0].1.after.draws-departure.after.draws;
 // Eight draws allow the mandatory boost/skill setup before the first lottery.
 // BG's later children and changing formations are covered by neighboring trials.
 let natural=menu.draws.saturating_sub(entry.after.draws+coarse)+8;
 Ok(Some((natural,curves,departure)))
}
fn target_first(entry:&Entry,targets:&[CrtRng],limit:u32)->Result<Option<Trip>>{
 let mut trials=vec![];
 for location in 0..3u8 {
  let mut model=None;
  for switches in (0..=32).step_by(2){
   if let Some(m)=jp_estimate(entry,location,switches)? {model=Some(m);break;}
  }
  let Some((natural,curves,departure))=model else{continue;};
  let mut l=Loader::new(world::catalog(),departure.world_after.clone(),departure.after)?;
  let mut departures=vec![];let mut counts=vec![];
  for switches in 0..=limit/12+4 {
   if switches%2==0 {
    departures.push((entry.after.draws+l.rng.draws-departure.after.draws,tropu_reload_cost(switches)));counts.push(switches);
   }
   let period=if l.state.time==2{4}else{2};l.manual_time_switch(period,"JP target estimate")?;
  }
  let mut ranked=crate::bg_target::rank(targets,&departures,natural,&curves,limit,128);ranked.truncate(384);
  trials.extend(ranked.into_iter().map(|c|(location,counts[c.departure],c)));
 }
 trials.sort_by_key(|(_,_,c)|c.score);
 let mut best:Option<Trip>=None;let mut seen=std::collections::BTreeSet::new();
 for (location,switches,c) in trials {
  let controls:[u8;5]=c.counts.try_into().unwrap();
  if !seen.insert((location,switches,controls)){continue;}
  let ceiling=best.as_ref().map_or(limit,|b|b.extra_input_cost);
  if tropu_reload_cost(switches)+controls.iter().map(|&n|controls_cost(n)).sum::<u32>()+4>=ceiling {continue;}
  let candidate=if location==0 {roque_defends(entry,&ERRAND_ORDER,switches,ceiling,controls)?}
   else{later_candidate_defends(entry,ROUTE_ORDER,switches,ceiling,location==2,controls)?};
  if let Some(t)=candidate {if best.as_ref().is_none_or(|b|t.extra_input_cost<b.extra_input_cost){best=Some(t);}}
 }
 Ok(best)
}
fn roque_defends(entry:&Entry,order:&[u8],switches:u32,limit:u32,c:[u8;5])->Result<Option<Trip>> {
 ensure(order==ERRAND_ORDER,"Errands already completed before Gravell")?;
 let mut segments=vec![];let mut current=entry.clone();
 for &n in order {let s=if n==2{book(&current)?}else{firestarter_defends(&current,c[0])?};current=s.end().clone();segments.push(s);}
 let approach=to_tropu_defends(&current,c[1],c[2])?;segments.push(approach);
 let cost=tropu_reload_cost(switches)+segments.iter().map(segment_cost).sum::<u32>();if cost+5>=limit{return Ok(None);}
 let boat=boat(segments.last().unwrap().end(),switches)?;let e=boat.end();let w=&e.world_after;let mut rng=e.after;
 let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions{map:w.map.clone(),zone:"ENT_FLD_SEA_3_1_LND".into(),period:"Night".into(),progress:w.main_progress_points,first_encounter:w.first_encounter,boundary:Boundary::CurrentCounter,current_counter:Some(w.encounter_counter),party_levels:w.party.iter().map(|id|w.character_progression[id].level).collect(),ability_percent:50,dash:true,..Default::default()})?;
 if encounter.enemies.len()!=2||encounter.enemies.iter().any(|e|e.enemy.id!="ENE_NML_OCN_230"){return Ok(None);}
 let situation=match encounter.situation.as_deref(){Some("Normal")=>ot2_battle_engine::escape::Situation::Normal,Some("Preemptive")=>ot2_battle_engine::escape::Situation::Preemptive,_=>return Ok(None)};
 let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
 let b=ot2_battle_engine::field_escape::roque_jp_battle(rng,situation,&players,&w.enemy_knowledge)?;
 let Some(f)=ot2_battle_engine::roque_routine::search(b,(limit-cost).min(96)as usize)?else{return Ok(None);};
 let cost=cost+f.actions.iter().map(|a|a.inputs.len()as u32).sum::<u32>();
 let (fight,rewards,mut ret)=crate::inventor::settle_gravell(e,rng,f)?;
 if !crate::galdera_jp::sufficient(&ret.world_after)? {return Ok(None);}
 for (item,n) in &rewards.drops{*ret.world_after.route_items.entry(item.clone()).or_default()+=n;}
 let downstream_draws=rng.draws-boat.stages[0].1.after.draws;
 Ok(Some(Trip{order:order.iter().copied().chain([1]).collect(),segments,boat,encounter,battle_entry:rng,fight,rewards,field_return:ret,extra_input_cost:cost,remaining_errands:vec![],location:"Roque Island Harbor".into(),continuation:vec![],nameless:None,nameless_jp:None,shipwreck:None,encounter_arrival:None,downstream_draws,defend_counts:c}))
}
/// Bounded refinement retains the accepted no-Defend fallback. Explore all
/// individual counts and uniform multi-encounter counts; never claim global optimum.
fn refine_defends(entry:&Entry,mut best:Trip)->Result<Trip> {
 let mut work=0;
 for _pass in 0..2 {
  let start=best.extra_input_cost;let base=best.defend_counts;let mut profiles=vec![];
  for count in (1..=7).rev(){profiles.push([0,count,count,count,count]);}
  for site in 1..5 {for count in 0..=7{let mut c=base;c[site]=count;if c!=base{profiles.push(c);}}}
  profiles.sort();profiles.dedup();
  for switches in (0u32..).step_by(2) {
   if tropu_reload_cost(switches)+4>=best.extra_input_cost{break;}
   for &c in &profiles {
    for order in [&ERRAND_ORDER[..]] {
     // Ignore controls for encounters absent from this route prefix.
     if c[3]!=base[3]||c[4]!=base[4]||(!order.contains(&3)&&c[0]!=0){continue;}
     let lower=controls_cost(c[1])+controls_cost(c[2])+if order.contains(&3){controls_cost(c[0])}else{0};
     if tropu_reload_cost(switches)+lower+5>=best.extra_input_cost{continue;}
     work+=1;if work>12000{return Ok(best);}
     if let Some(t)=roque_defends(entry,order,switches,best.extra_input_cost,c)?{if t.extra_input_cost<best.extra_input_cost{best=t;}}
    }
    for order in LATER_ORDERS {for ship in [false,true] {
     if !ship&&c[4]!=base[4]{continue;}
     let lower=c[..if ship{5}else{4}].iter().map(|&n|controls_cost(n)).sum::<u32>();
     if tropu_reload_cost(switches)+lower+4>=best.extra_input_cost{continue;}
     work+=1;if work>12000{return Ok(best);}
     if let Some(t)=later_candidate_defends(entry,order,switches,best.extra_input_cost,ship,c)?{if t.extra_input_cost<best.extra_input_cost{best=t;}}
    }}
   }
  }
  if best.extra_input_cost==start{break;}
 }
 Ok(best)
}
fn controls_cost(n:u8)->u32 {u32::from(n)*3+if n>=4{8}else{0}}
fn later_candidate(entry:&Entry,order:[u8;1],switches:u32,limit:u32,ship:bool)->Result<Option<Trip>> {later_candidate_defends(entry,order,switches,limit,ship,[0;5])}
fn later_candidate_defends(entry:&Entry,order:[u8;1],switches:u32,limit:u32,ship:bool,c:[u8;5])->Result<Option<Trip>> {
 let mut best:Option<Trip>=None;
 // Preserve daytime departure: 1 or 3 reloads fit the four-reload free allowance.
 // Paid alternatives use the same 96-input adjustment window as battle search.
 for reloads in (1..).step_by(2) {
  let reload_cost=roque_reload_cost(reloads);
  let budget=best.as_ref().map_or(limit,|t|t.extra_input_cost);
  if reload_cost>96||tropu_reload_cost(switches)+reload_cost+4>=budget {break;}
  if let Some(t)=later_candidate_reloads(entry,order,switches,budget,ship,c,reloads)? {best=Some(t);}
 }
 Ok(best)
}
fn later_candidate_reloads(entry:&Entry,order:[u8;1],switches:u32,limit:u32,ship:bool,c:[u8;5],reloads:u32)->Result<Option<Trip>> {
 ensure(order==ROUTE_ORDER,"Expected Roque after Gravell")?;
 let mut current=entry.clone();let mut before=vec![];let mut after=vec![];let mut purchased=None;let mut cost=0;
 for n in order {
  if n==1 {
   let Some(r)=deferred_roque_reloads(&current,switches,c,reloads)?else{return Ok(None);};
   cost+=segment_cost(&r.approach)+segment_cost(&r.boat)+defend_cost(&r.harbor)+roque_reload_cost(reloads);
   current=r.field_return.clone();before.push(r.approach);
   let mut harbor=Segment::new("Roque Harbor daytime Escape");harbor.stages.push(("Roque Harbor field return".into(),r.harbor.field_return.clone()));harbor.encounters.push(r.harbor);
   after.extend([harbor,r.town]);let d=day(&current)?;current=d.end().clone();after.push(d);purchased=Some(r.boat);
  }else{
   let s=if n==2{book(&current)?}else{firestarter_defends(&current,c[0])?};cost+=segment_cost(&s);current=s.end().clone();
   if purchased.is_some(){after.push(s);}else{before.push(s);}
  }
 }
 if cost+4>=limit{return Ok(None);}
 let arrival=nameless_arrival(&current)?;let island=arrival.end().clone();after.push(arrival);
 let boat=purchased.unwrap();let anchor=boat.stages[0].1.after;
 let budget=(limit-cost).min(96)as usize;

 if !ship {
 if let Some(j)=crate::galdera_nameless::jp_finish(&island,budget)? {
  let inputs=j.fight.actions.iter().map(|a|a.inputs.len()as u32).sum::<u32>();
  return Ok(Some(Trip{order:order.into_iter().chain([4]).collect(),segments:before,boat,encounter:j.encounter.clone(),battle_entry:j.battle_entry,fight:j.fight.clone(),rewards:j.rewards.clone(),field_return:j.field_return.clone(),extra_input_cost:cost+inputs,remaining_errands:vec![],location:"Nameless Isle".into(),continuation:after,nameless:None,encounter_arrival:Some(j.water_entry.clone()),downstream_draws:j.battle_entry.draws-anchor.draws,nameless_jp:Some(j),shipwreck:None,defend_counts:c}));
 }
 return Ok(None);
 }
 // No JP awarded here: flee, finish Gigantes, then carry Georges/loot state.
 let Some(n)=crate::galdera_nameless::finish_with_defends(&island,((limit-cost)/2).min(48),c[4])?else{return Ok(None);};
 cost+=n.approach.escape_inputs.len()as u32+n.finisher_boost_cycles*2+defend_cost(&n.approach.encounter);
 if cost+6>=limit{return Ok(None);}
 let Some(s)=crate::galdera_shipwreck::run(n.loot.end(),true,(limit-cost).min(96)as usize)?else{return Ok(None);};
 let j=s.jp.as_ref().unwrap();cost+=j.fight.actions.iter().map(|a|a.inputs.len()as u32).sum::<u32>();
 Ok(Some(Trip{order:order.into_iter().chain([4,5]).collect(),segments:before,boat,encounter:j.encounter.clone(),battle_entry:j.battle_entry,fight:j.fight.clone(),rewards:j.rewards.clone(),field_return:j.field_return.clone(),extra_input_cost:cost,remaining_errands:vec![],location:"Shipwreck".into(),continuation:after,encounter_arrival:Some(s.travel.end().clone()),downstream_draws:j.battle_entry.draws-anchor.draws,nameless:Some(n),nameless_jp:None,shipwreck:Some(s),defend_counts:c}))
}

#[cfg(test)]mod tests {
 use super::*;
 #[test]fn reload_allowances_only_discount_the_named_sites() {
  assert_eq!((0..=5).map(tropu_reload_cost).collect::<Vec<_>>(),vec![0,0,0,0,12,24]);
  assert_eq!((0..=6).map(roque_reload_cost).collect::<Vec<_>>(),vec![0,0,0,0,0,12,24]);
  let mut boat=Segment::new("Boat and Roque Harbor");boat.extra_switches=3;assert_eq!(segment_cost(&boat),0);
  let mut sai=Segment::new("Sai");sai.extra_switches=3;assert_eq!(segment_cost(&sai),36);
 }
 #[test]#[ignore="Specific screenshot seed audit; full route plus independent location searches"]
 fn audit_seed_05d03c86() {
  let path="target/audit-05d03c86-entry.json";
  let v:serde_json::Value=if let Ok(s)=std::fs::read_to_string(path){serde_json::from_str(&s).unwrap()}else{
   let r=crate::identified_route::simulate_route("SSSSSSSSSSSSSSSNNNNNNSNSNSNSSSSS",false,197,true).unwrap();
   assert_eq!(r["initial_seed"],0x05d03c86u32);
   fn find(v:&serde_json::Value)->Option<&serde_json::Value>{if v["restoration"].is_object()&&v["jam_steal"].is_object(){return Some(&v["jam_steal"]);}v.as_object()?.values().find_map(find)}
   let e=find(&r).expect("Gravell flex entry").clone();std::fs::write(path,serde_json::to_vec(&e).unwrap()).unwrap();e
  };
  let e=Entry{arrival:serde_json::from_value(v["arrival"].clone()).unwrap(),after:serde_json::from_value(v["after"].clone()).unwrap(),world_after:serde_json::from_value(v["world_after"].clone()).unwrap(),trace:vec![],decisions:vec![]};
  fn report(t:&Trip){
   let anchor=t.boat.stages[0].1.after;
   let bg=t.fight.actions.iter().find(|a|matches!(a.action,ot2_battle_engine::gravell_routine::Action::BewilderingGrace)).unwrap();
   eprintln!("AUDIT {} order {:?} switches {} cost {} battle_inputs {} downstream {} BG_before_delta {} anchor_draws {}",t.location,t.order,t.boat.extra_switches,t.extra_input_cost,t.fight.actions.iter().map(|a|a.inputs.len()).sum::<usize>(),t.downstream_draws,bg.before.draws-anchor.draws,anchor.draws);
   std::fs::write(format!("target/audit-05d03c86-{}.json",t.location.replace(' ',"-")),serde_json::to_vec(t).unwrap()).unwrap();
  }
  let roque=finish_roque(&e).unwrap();report(&roque);
  for ship in [false,true] {
   let mut best:Option<Trip>=None;
   for switches in (0u32..1000).step_by(2) {
    if best.as_ref().is_some_and(|t|tropu_reload_cost(switches)+4>=t.extra_input_cost){break;}
    for order in LATER_ORDERS {
     if let Some(t)=later_candidate(&e,order,switches,best.as_ref().map_or(100000,|t|t.extra_input_cost),ship).unwrap(){
      if best.as_ref().is_none_or(|b|t.extra_input_cost<b.extra_input_cost){eprintln!("candidate {} switches {} cost {}",t.location,switches,t.extra_input_cost);best=Some(t);}
     }
    }
   }
   report(&best.expect("Location candidate within 1000 switches"));
  }
  let selected=finish(&e).unwrap();eprintln!("AUTOMATIC {} switches {} cost {}",selected.location,selected.boat.extra_switches,selected.extra_input_cost);
  assert!(selected.extra_input_cost<=196);
 }
 #[test]#[ignore="Specific screenshot seed audit; full route plus independent location searches"]
 fn audit_seed_56147f50() {
  let path="target/audit-56147f50-entry.json";
  let v:serde_json::Value=if let Ok(s)=std::fs::read_to_string(path){serde_json::from_str(&s).unwrap()}else{
   let r=crate::identified_route::simulate_route("SSSSSSNSNSNSNSNSNSNSNSNSNNNN",true,154,true).unwrap();
   assert_eq!(r["initial_seed"],0x56147f50u32);
   fn sands(v:&serde_json::Value)->Option<&serde_json::Value>{if v["lord_sands"].is_object(){return Some(v);}v.as_object()?.values().find_map(sands)}
   let q=sands(&r).unwrap();eprintln!("SANDS ACTIONS {}",q["lord_sands"]["battle"]["actions"].as_array().unwrap().iter().map(|a|format!("T{} actor{} {}",a["turn"],a["actor"],a["name"])).collect::<Vec<_>>().join("; "));
   assert!(q["lord_sands"]["battle"]["actions"].as_array().unwrap().iter().any(|a|a["turn"]==1&&a["actor"]==2&&a["name"]=="Holy Light"));
   fn find(v:&serde_json::Value)->Option<&serde_json::Value>{if v["restoration"].is_object()&&v["jam_steal"].is_object(){return Some(&v["jam_steal"]);}v.as_object()?.values().find_map(find)}
   let e=find(&r).expect("Gravell flex entry").clone();std::fs::write(path,serde_json::to_vec(&e).unwrap()).unwrap();e
  };
  let e=Entry{arrival:serde_json::from_value(v["arrival"].clone()).unwrap(),after:serde_json::from_value(v["after"].clone()).unwrap(),world_after:serde_json::from_value(v["world_after"].clone()).unwrap(),trace:vec![],decisions:vec![]};
  fn report(t:&Trip){
   let anchor=t.boat.stages[0].1.after;
   let bg=t.fight.actions.iter().find(|a|matches!(a.action,ot2_battle_engine::gravell_routine::Action::BewilderingGrace)).unwrap();
   eprintln!("AUDIT {} order {:?} switches {} cost {} battle_inputs {} downstream {} BG_before_delta {} anchor_draws {}",t.location,t.order,t.boat.extra_switches,t.extra_input_cost,t.fight.actions.iter().map(|a|a.inputs.len()).sum::<usize>(),t.downstream_draws,bg.before.draws-anchor.draws,anchor.draws);
   std::fs::write(format!("target/audit-56147f50-{}.json",t.location.replace(' ',"-")),serde_json::to_vec(t).unwrap()).unwrap();
  }
  let roque=finish_roque(&e).unwrap();report(&roque);
  for ship in [false,true] {
   let mut best:Option<Trip>=None;
   for switches in (0u32..1000).step_by(2) {
    if best.as_ref().is_some_and(|t|tropu_reload_cost(switches)+4>=t.extra_input_cost){break;}
    for order in LATER_ORDERS {
     if let Some(t)=later_candidate(&e,order,switches,best.as_ref().map_or(100000,|t|t.extra_input_cost),ship).unwrap(){
      if best.as_ref().is_none_or(|b|t.extra_input_cost<b.extra_input_cost){eprintln!("candidate {} switches {} cost {}",t.location,switches,t.extra_input_cost);best=Some(t);}
     }
    }
   }
   report(&best.expect("Location candidate within 1000 switches"));
  }
  let selected=finish(&e).unwrap();eprintln!("AUTOMATIC {} switches {} cost {} defends {:?}",selected.location,selected.boat.extra_switches,selected.extra_input_cost,selected.defend_counts);
  assert!(selected.extra_input_cost<282);assert!(selected.defend_counts.iter().any(|&n|n>0));
  
 }
 fn fixture()->Entry {
  let v:serde_json::Value=serde_json::from_str(include_str!("../data/source/galdera-flex-entry.json")).unwrap();
  let rng: CrtRng=serde_json::from_value(v["after"].clone()).unwrap();
  let mut world_after:world::WorldState=serde_json::from_value(v["world_after"].clone()).unwrap();world_after.refresh_support_skills();
  Entry{arrival:rng,after:rng,world_after,trace:vec![],decisions:vec![]}
 }
 fn current_fixture()->Entry {
  let e=fixture();let mut l=Loader::new(world::catalog(),e.world_after,e.after).unwrap();
  crate::gravell_setup::learn(&mut l.state,6,0,&["ABI_SET_MER_030"]).unwrap();
  l.state.character_progression.get_mut(&6).unwrap().jobs.learned_abilities.entry(6).or_default().insert("ABI_SET_SCH_060".into());
  crate::gravell_setup::jp_setup(&mut l).unwrap();
  crate::nameless_setup::track_augmentation_for_legacy_fixture(&mut l.state);
  // Only acquisitions before flex missing from the historical fixture.
  for (id,n) in [("ITM_EQP_LNS_220",1),("ITM_EQP_HLM_180",1),("ITM_EQP_ACS_250",1),("ITM_CSM_0610",1),("ITM_CSM_0730",1),("ITM_CSM_0550",1)] {l.state.route_items.insert(id.into(),n);}
  let e=before_gravell(&point(&mut l,e.after)).unwrap().last().unwrap().end().clone();
  let mut l=Loader::new(world::catalog(),e.world_after,e.after).unwrap();
  l.visit_map("Twn_Wld_3_1_A").unwrap();
  crate::nameless_setup::equip_gravell_osvald(&mut l).unwrap();
  point(&mut l,e.after)
 }
 #[test]fn location_probes_stop_before_bg_and_preserve_source_state(){
  let entry=current_fixture();let before=crate::state_key::WorldKey::new(&entry.world_after);
  for location in 0..3 {
   let model=(0..=32).step_by(2).find_map(|n|jp_estimate(&entry,location,n).unwrap()).expect("BG-menu calibration for each location");
   assert!(model.0>0);assert_eq!(model.1.len(),5);
   assert!(model.1[1][7]>model.1[1][0]);
   assert_eq!(crate::state_key::WorldKey::new(&entry.world_after),before);
   eprintln!("JP model location {location}: natural draws {}",model.0);
  }
 }
 #[test]#[ignore="full target-first JP search; run optimized"]
 fn target_first_replays_jp100_before_legacy_fallback(){
  let entry=current_fixture();let started=std::time::Instant::now();
  let targets=crate::bg_target::targets(entry.after,20_000,"ABI_DAN_MAKA_001").unwrap();
  let t=target_first(&entry,&targets,1024).unwrap().expect("Target-first candidate");
  assert_eq!(t.boat.extra_switches%2,0);
  assert_eq!(t.boat.stages[0].1.world_after.time,2);
  assert_eq!(t.fight.state.outcome,ot2_battle_engine::Outcome::Victory);
  assert!(t.fight.state.actors[..4].iter().all(|a|a.alive()));
  assert_eq!(t.fight.state.grace.as_ref().unwrap().jp_percent(),10000);
  assert_eq!(t.field_return.world_after.flag(40003),t.location=="Shipwreck");
  eprintln!("Target-first JP: {}, switches {}, cost {}, Defends {:?}, elapsed {:?}",t.location,t.boat.extra_switches,t.extra_input_cost,t.defend_counts,started.elapsed());
 }
 #[test]fn seven_defends_replay_safe_rounds_and_field_counter() {
  let entry=current_fixture();
  for count in 0..=7 {
   let s=to_tropu_defends(&entry,count,count).unwrap();
   assert!(s.movement.as_ref().unwrap().estimated_counter_at_exit>0);
   for f in &s.encounters {
    assert_eq!(f.defends.len(),count as usize);
    assert_eq!(f.defend_rounds,(0..count).map(|n|if n<4{1}else{2}).collect::<Vec<_>>());
    assert_eq!(f.escape_round,if count<4{1}else{2});
    assert!(f.escape.success);assert_eq!(f.escape.chance_percent,100);
    assert_eq!(defend_cost(f),controls_cost(count));
    for report in &f.defends {assert!(report.actor.0<4);assert!(report.rng_after.draws>report.rng_before.draws);}
    assert!(f.field_return.after.draws>f.escape.after.draws);
   }
  }
 }
 #[test]fn later_locations_replay_prerequisites_and_stop_at_jp_victory() {
  for ship in [false,true] {
   let mut found=None;
   for switches in (0..512).step_by(2) {
    if let Some(t)=later_candidate(&current_fixture(),ROUTE_ORDER,switches,10000,ship).unwrap(){found=Some(t);break;}
   }
   let t=found.expect("Complete later JP candidate");
   assert_eq!(&t.order[..1],&ROUTE_ORDER);assert_eq!(t.order[1],4);
   assert_eq!(t.order.last(),Some(if ship{&5}else{&4}));
   assert!(t.remaining_errands.is_empty());assert!(t.rewards.awarded.jp>=12210);
   assert_eq!(t.fight.state.outcome,ot2_battle_engine::Outcome::Victory);
   assert!(t.fight.state.actors[..4].iter().all(|a|a.alive()));
   assert_eq!(t.field_return.world_after.flag(40003),ship);
   assert!(t.downstream_draws>0);
   eprintln!("{}: switches {}, downstream draws {}, cost {}",t.location,t.boat.extra_switches,t.downstream_draws,t.extra_input_cost);
  }
 }
 #[test]fn automatic_selector_never_costs_more_than_roque_fallback() {
  let entry=current_fixture();let baseline=finish_roque(&entry).unwrap();let selected=finish(&entry).unwrap();
  assert!(selected.extra_input_cost<=baseline.extra_input_cost);
  assert_eq!(&baseline.order[..1],&ROUTE_ORDER);
  assert_eq!(&selected.order[..1],&ROUTE_ORDER);
  eprintln!("Selected {} {:?}, cost {} vs Roque {}",selected.location,selected.order,selected.extra_input_cost,baseline.extra_input_cost);
 }
 #[test]fn selector_can_choose_a_later_location() {
  for seed in 0..16 {
   let mut entry=current_fixture();entry.after=CrtRng::new(seed);entry.arrival=entry.after;
   let selected=finish(&entry).unwrap();
   if selected.location!="Roque Island Harbor" {
    assert_eq!(selected.order[1],4);assert_eq!(selected.remaining_errands.len(),0);
    eprintln!("Seed {seed}: {} {:?}, cost {}",selected.location,selected.order,selected.extra_input_cost);return;
   }
  }
  panic!("Expected at least one later-location winner in deterministic sample");
 }
 #[test]fn fixed_errand_order_keeps_inventory_and_daytime_without_repeating_pickups() {
  {
   let entry=fixture();let segments=before_gravell(&entry).unwrap();
   assert_eq!(segments.iter().map(|s|s.name.as_str()).collect::<Vec<_>>(),vec!["Sai / Firestarter","Crackridge book"]);
   assert_eq!(segments[0].stages[0].1.arrival,entry.after);
   assert_eq!(segments[1].stages[0].1.arrival,segments[0].end().after);
   let e=segments[1].end();
   assert_eq!(e.world_after.time,2);
   for c in &data().chests {assert_eq!(e.world_after.route_items[&c.item],fixture().world_after.route_items.get(&c.item).copied().unwrap_or(0)+1);}
   assert_eq!(e.world_after.route_items["ITM_MRL_REV_0040"],1);
   assert!(e.world_after.swooned_route_npcs.contains(&data().scholar_placement));
  }
 }
 #[test]fn boat_purchase_requires_daytime_before_either_harbor_branch() {
  let s=to_tropu(&fixture()).unwrap();assert_eq!(s.encounters.len(),2);assert!(s.movement.as_ref().unwrap().estimated_counter_at_exit>0);
  for switches in [0,2,4] {for night in [false,true] {
   let b=boat_period(s.end(),switches,night).unwrap();let e=b.end();assert_eq!(e.world_after.map,"Fld_Sea_3_1");assert_eq!(e.world_after.time,if night {4}else{2});
   // Every stage through purchase/sea/harbor arrival stays in daytime.
   for (label,stage) in &b.stages {assert_eq!(stage.world_after.time,if label=="Roque Harbor night" {4}else{2},"{label}");}
   assert!(e.world_after.flag(6208));assert!(!e.world_after.route_items.contains_key("ITM_TRE_SHO_EX1_0030"));
   assert_eq!(e.world_after.route_items["ITM_TRE_SHO_EX1_0010"],1);
   let departure=&b.stages[0].1;
   let mut replay=Loader::new(world::catalog(),s.end().world_after.clone(),s.end().after).unwrap();
   for i in 0..switches {replay.manual_time_switch(if i%2==0 {4}else{2},"test Tropu adjustment").unwrap();}
   assert_eq!(departure.after,replay.rng);
  }}
  for switches in [1,3,5] {for night in [false,true] {assert!(boat_period(s.end(),switches,night).is_err());}}
  let mut night_entry=s.end().clone();night_entry.world_after.time=4;
  assert!(boat(&night_entry,0).is_err());
 }
 #[test]fn complete_jp100_search_requires_the_full_four_alive_fireball_victory() {
  let r=finish_roque(&current_fixture()).unwrap();assert_eq!(r.order,ROUTE_ORDER);assert!(r.remaining_errands.is_empty());assert_eq!(r.rewards.base.jp,116);assert_eq!(r.rewards.awarded.jp,12760);
  assert_eq!(r.boat.extra_switches%2,0);
  assert_eq!(r.boat.stages[0].1.world_after.time,2);
  assert_eq!(r.fight.state.outcome,ot2_battle_engine::Outcome::Victory);assert!(r.fight.state.actors[..4].iter().all(|a|a.alive()));
  assert_eq!(r.fight.actions.len(),6);assert_eq!(r.fight.actions[4].actor.0,2);assert_eq!(r.fight.actions[5].actor.0,1);
  assert_eq!(r.fight.actions[5].action,ot2_battle_engine::gravell_routine::Action::FireballTwo);
  assert_eq!(r.rewards.hired_help_cost,0);
  assert_eq!(r.fight.state.grace.as_ref().unwrap().jp_percent(),10000);
  assert_eq!(r.field_return.world_after.route_items["ITM_CSM_0080"],fixture().world_after.route_items["ITM_CSM_0080"]);
  for id in [7,6,5,4] {assert_eq!(r.field_return.world_after.character_progression[&id].jp_earned-fixture().world_after.character_progression[&id].jp_earned,12760);}
 }
 #[test]fn deferred_daytime_roque_carries_supplies_without_awarding_jp100() {
  let entry=fixture();
  {
   let r=(0..32).map(|n|deferred_roque(&entry,n*2).unwrap()).find_map(|r|r).expect("Safe daytime harbor candidate");
   assert_eq!(r.boat.end().world_after.time,2);
   assert_eq!(r.harbor.escape.chance_percent,100);assert!(r.harbor.escape.success);
   assert_ne!(r.harbor.encounter.situation.as_deref(),Some("Surprise"));
   assert_eq!(r.field_return.world_after.map,"Twn_Sea_3_1_A");
   assert_eq!(r.field_return.world_after.time,2);assert!(!r.jp100_complete);
   for (id,count) in [("ITM_CSM_0080",1),("ITM_CSM_0090",1),("ITM_CSM_0240",1),("ITM_CSM_0111",3),("ITM_CSM_0610",1),("ITM_EQP_SLD_240",0),("ITM_EQP_AMR_300",0),("ITM_CSM_0390",0)] {
    assert_eq!(r.field_return.world_after.route_items.get(id).copied().unwrap_or(0),entry.world_after.route_items.get(id).copied().unwrap_or(0)+count,"{id}");
   }
   for id in [7,6,5,4] {assert_eq!(r.field_return.world_after.character_progression[&id].jp_earned,entry.world_after.character_progression[&id].jp_earned);}
   let steal=&r.town.stages[1].1;assert_eq!(steal.after.draws-steal.arrival.draws,3);
   assert_eq!(r.chest_money,39800);assert_eq!(r.shopping_cost,42000);
   let gear=&r.field_return.world_after.character_progression[&7].jobs.equipped_route_items;
   assert_eq!(gear["Shield"],"ITM_EQP_SLD_240");assert_eq!(gear["Body"],"ITM_EQP_AMR_300");
   let daylight=&r.town.stages[4].1;
   assert_eq!(daylight.world_after.time,2);assert!(daylight.after.draws>daylight.arrival.draws);
   assert_eq!(r.town.stages[5].1.after,r.town.stages[5].1.arrival);
   assert_eq!(r.town.stages[6].1.after,r.town.stages[6].1.arrival);
   let arrival=nameless_arrival(&r.field_return).unwrap();
   assert_eq!(arrival.stages[0].1.after,r.field_return.after);
   assert!(arrival.stages.iter().all(|(_,e)|e.world_after.time==2));
   let adjusted=crate::galdera_roque::town_with_reloads(&r.harbor.field_return,3).unwrap();
   assert_eq!(adjusted.end().world_after.time,2);
   assert_eq!(adjusted.extra_switches,2);
   assert!(adjusted.end().after.draws>r.field_return.after.draws);
   assert_eq!(adjusted.end().world_after.route_items,r.field_return.world_after.route_items);
  }
 }
 #[test]#[ignore="Expanded synthetic RNG-state search"]fn alternate_rng_states_find_complete_jp_routes() {
  for seed in [0,1,12345678] {
   let mut e=fixture();e.after=CrtRng::new(seed);e.arrival=e.after;
   let r=finish(&e).unwrap();assert_eq!(r.rewards.awarded.jp,12760);
   eprintln!("Seed {seed}: order {:?}, switches {}, cost {}",r.order,r.boat.extra_switches,r.extra_input_cost);
  }
 }
}
