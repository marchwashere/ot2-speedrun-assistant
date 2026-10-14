//! Daytime supplies and nighttime loot; inactive until later JP targets exist.
use crate::{Result,ensure,world::{self,Loader},snows_escape::Entry,galdera_flex::Segment};
use serde::Deserialize;
#[derive(Deserialize)]struct Item {id:String,level:u32}
#[derive(Deserialize)]struct Chest {id:String,item:String,count:u32,money:bool}
#[derive(Deserialize)]struct Purchase {id:String,price:u32}
#[derive(Deserialize)]struct Data {worker:String,steals:Vec<Item>,guard:String,guard_placements:Vec<String>,ambush_level:u32,before:Vec<String>,success:String,chests:Vec<Chest>,shopping:Vec<Purchase>}
pub(crate) fn town(entry:&Entry,wind_m:bool)->Result<Segment> {
 let d:Data=serde_json::from_str(include_str!("../data/galdera-roque.json"))?;
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 ensure(l.state.map=="Fld_Sea_3_1"&&l.state.time==2,"Roque supplies require daytime harbor field return")?;
 let mut s=Segment::new("Roque Island supplies");
 let before=l.rng;l.visit_map("Twn_Sea_3_1_A")?;l.skipped_event("MapInStorySelect")?;
 s.save(&mut l,"Roque Island entered during day; discard chapter start",before);
 ensure(l.state.party.contains(&7)&&l.state.existing_npcs.iter().any(|id|l.data.placements[id].resource==d.worker),"Daytime Factory Worker or Throne unavailable")?;
 let before=l.rng;l.draw(world::DrawKind::PathActionMenu,&d.worker,"Factory Worker Steal menu opening",None);
 for item in d.steals.iter().take(if wind_m {3}else{2}) {
  ensure(l.state.character_progression[&7].level>=item.level,"Factory Worker Steal not guaranteed")?;
  l.draw(world::DrawKind::StealCheck,&item.id,"Guaranteed Steal",Some(100));
  *l.state.route_items.entry(item.id.clone()).or_default()+=1;
 }
 l.boundary("Switch path action, then close Steal without dialogue lottery");
 s.save(&mut l,if wind_m {"Steal Pom M, Pom L and Wind Soulstone M; switch path action and close"}else{"Steal Pom M and Pom L; switch path action and close"},before);
 let before=l.rng;l.manual_time_switch(4,"Roque Island: night for guarded-building Ambush")?;s.save(&mut l,"Switch to night",before);
 ensure(l.state.character_progression[&7].level>=d.ambush_level&&l.state.existing_npcs.iter().any(|id|l.data.placements[id].resource==d.guard),"Roque Guard Ambush unavailable")?;
 let before=l.rng;l.draw(world::DrawKind::PathActionMenu,&d.guard,"Ambush preview",None);
 let roll=l.draw(world::DrawKind::DialogueLottery,&d.guard,"Ambush before dialogue",None);
 l.skipped_event(&d.before[usize::from(roll>16383)])?;
 for id in &d.guard_placements {if l.state.existing_npcs.contains(id){l.state.swooned_route_npcs.insert(id.clone());}}
 l.skipped_event(&d.success)?;s.save(&mut l,"Ambush Guard (requires Throne48)",before);
 let before=l.rng;
 for c in &d.chests {
  ensure(l.state.opened_route_chests.insert(c.id.clone()),"Roque chest already opened")?;
  if !c.money {*l.state.route_items.entry(c.item.clone()).or_default()+=c.count;}
  l.boundary(format!("Open {}: {} x{}",c.id,if c.money {"leaves"}else{&c.item},c.count));
 }
 s.save(&mut l,"Collect Lychee L, 39,800 leaves, three Rejuvenating Jams and Magic Nut L",before);
 let before=l.rng;
 for item in &d.shopping {
  *l.state.route_items.entry(item.id.clone()).or_default()+=1;
  l.boundary(format!("Shop purchase {} for {} leaves; ordinary inventory, no information unlock",item.id,item.price));
 }
 s.save(&mut l,"Buy Swift Shield and Quick Cloak (42,000 leaves); JP100 still pending",before);
 Ok(s)
}
