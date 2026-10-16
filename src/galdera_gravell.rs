//! Gravell blacksmith restoration, Smooth Talker sales and guaranteed Jam Steal.
use crate::{Result,ensure,CrtRng,world::{self,Loader},snows_escape::Entry};
use serde::{Deserialize,Serialize};
use std::sync::OnceLock;
#[derive(Deserialize)]struct Data {guard:String,ambush_level:u32,before_events:Vec<String>,success_event:String,shopkeeper:String,jam:String,proper_steal:u32,steal_level:u32,weapons:Vec<Weapon>,job:Job,first_talk_flag:u32,license_flag:u32}
#[derive(Deserialize)]struct Weapon {rust:String,item:String,sort:u32,sell_price:u32,flag:u32,ability:String}
#[derive(Deserialize)]struct Ability {#[serde(rename="AbilityName")]id:String,#[serde(rename="GetParam")]count:usize}
#[derive(Deserialize)]struct Job {#[serde(rename="JobCommandAbility")]commands:Vec<Ability>,#[serde(rename="JobSupportAbility")]supports:Vec<Ability>}
fn data()->&'static Data {static D:OnceLock<Data>=OnceLock::new();D.get_or_init(||serde_json::from_str(include_str!("../data/galdera-gravell.json")).unwrap())}
#[derive(Debug,Serialize)]pub struct Trip {pub arrival:Entry,pub setup:Entry,pub night:Entry,pub ambush:Entry,pub restoration:Entry,pub day:Entry,pub sales:Entry,pub jam_steal:Entry,pub throne_level:u32,pub ambush_required_level:u32,pub jam_guaranteed_level:u32,pub jam_steal_chance:u32,pub jam_steal_roll:u32,pub sale_proceeds:u32}
fn point(l:&mut Loader<'_>,before:CrtRng)->Entry {Entry {arrival:before,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}}
fn present(l:&Loader<'_>,npc:&str)->bool {l.state.existing_npcs.iter().any(|id|l.data.placements[id].resource==npc)}
pub(crate) fn finish(entry:&Entry)->Result<Trip> {
 let d=data();let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 ensure(l.state.time==2,"Gravell restoration starts after daytime Gaol loot")?;
 l.visit_map("Twn_Wld_3_1_A")?;l.skipped_event("MapInStorySelect")?;
 let arrival=point(&mut l,entry.after);
 crate::gravell_setup::jp_setup(&mut l)?;let setup=point(&mut l,arrival.after);
 l.manual_time_switch(4,"Gravell: night for Debt Collector Ambush")?;let night=point(&mut l,setup.after);
 let throne_level=l.state.character_progression[&7].level;
 ensure(l.state.party.contains(&7)&&present(&l,&d.guard)&&throne_level>=d.ambush_level,"Debt Collector Ambush unavailable")?;
 l.draw(world::DrawKind::PathActionMenu,&d.guard,"Ambush preview: opening action timer",None);
 let raw=l.draw(world::DrawKind::DialogueLottery,&d.guard,"Ambush before-dialogue lottery: weights 0.5 / 0.5",None);
 l.skipped_event(&d.before_events[usize::from(raw>16383)])?;
 l.state.swooned_route_npcs.insert("NPC_Twn_Wld_3_1_A_TALK_1600_D000".into());
 l.skipped_event(&d.success_event)?;l.boundary("Debt Collector ambushed; fixed success dialogue, no battle");
 let ambush=point(&mut l,night.after);
 ensure(d.weapons.iter().all(|w|l.state.route_items.get(&w.rust).copied().unwrap_or(0)>0),"Both Rusty Sword and Rusty Axe are required")?;
 ensure(!l.state.flag(d.first_talk_flag)&&!l.state.flag(d.license_flag),"Only first Armsmaster visit is audited")?;
 l.skipped_event("SYS_WPM_GUILD_0010")?;
 ensure(d.weapons.iter().all(|w|l.state.route_items.get(&w.item)==Some(&1))&&l.state.job_licenses.get(&8)==Some(&1),"Armsmaster restoration incomplete")?;
 let restoration=point(&mut l,ambush.after);
 l.manual_time_switch(2,"Gravell: day before shop sales and Reinforcing Jam Steal")?;let day=point(&mut l,restoration.after);
 ensure(l.state.party.contains(&4)&&l.state.hired_merchant.as_ref().is_some_and(|h|h.talent=="SPACT_MERCHANT_014"),"Weapon sales require hired Smooth Talker")?;
 let mut sale_proceeds=0;
 // User sells axe then sword, separately. Native fixed 25..25 range still draws.
 for w in d.weapons.iter().rev() {
  ensure(l.state.route_items.remove(&w.item)==Some(1),"Restored weapon missing at sale")?;
  l.draw(world::DrawKind::MerchantTalent,&w.item,"Smooth Talker sale: native random integer range 25..25",Some(1));
  let proceeds=w.sell_price+(w.sell_price*25).div_ceil(100);sale_proceeds+=proceeds;
  l.boundary(format!("Sold {} for {proceeds} leaves with Smooth Talker",w.item));
 }
 let sales=point(&mut l,day.after);
 ensure(present(&l,&d.shopkeeper)&&throne_level>=d.steal_level,"Reinforcing Jam Steal is unavailable or not guaranteed")?;
 l.draw(world::DrawKind::PathActionMenu,&d.shopkeeper,"Open Shopkeeper Steal menu: action timer",None);
 let jam_steal_roll=l.draw(world::DrawKind::StealCheck,&d.jam,format!("Reinforcing Jam: ProperSteal{}, Throné{throne_level}, chance100%",d.proper_steal),Some(100))+1;
 if let Some(t)=l.trace.last_mut(){t.value=Some(jam_steal_roll);}
 *l.state.route_items.entry(d.jam.clone()).or_default()+=1;
 l.boundary("Reinforcing Jam stolen; change selected path action, then close without dialogue or closing lottery");
 crate::nameless_setup::equip_gravell_osvald(&mut l)?;
 let jam_steal=point(&mut l,sales.after);
 Ok(Trip {arrival,setup,night,ambush,restoration,day,sales,jam_steal,throne_level,ambush_required_level:d.ambush_level,jam_guaranteed_level:d.steal_level,jam_steal_chance:100,jam_steal_roll,sale_proceeds})
}

/// Native BP_WeaponMasterEventFlowV2 states for the fresh two-weapon route.
/// The actual imported script determines jumps, flags, refreshes and tutorials.
#[derive(Default)]pub(crate) struct Flow {state:u8,last_weapon:Option<String>}
impl Flow {
 pub(crate) fn start(&mut self){self.state=1;}
 pub(crate) fn branch(&mut self,l:&mut Loader<'_>)->Result<usize> {
  let has_rust=data().weapons.iter().any(|w|l.state.route_items.contains_key(&w.rust));
  let (next,index)=match self.state {
   1=>{ensure(!l.state.flag(data().license_flag),"Armsmaster revisit not supported")?;(2,0)},
   2=>{ensure(!l.state.flag(data().first_talk_flag),"Armsmaster first talk already done")?;(3,0)},
   3=>{l.state.set_flag(data().first_talk_flag,true);ensure(has_rust,"No rusty weapon")?;(4,0)},
   4=>(20,0),20=>{ensure(!l.state.flag(data().license_flag),"Unexpected license before first conversion")?;(21,0)},
   21|22=>(23,0),
   23=>if has_rust {(22,0)}else{(6,1)},
   6=>(7,0),7=>(24,0),
   _=>return Err(crate::SimError(format!("Unsupported Armsmaster flow state {}",self.state))),
  };self.state=next;Ok(index)
 }
 pub(crate) fn apply(&mut self,l:&mut Loader<'_>)->Result<()> {
  let d=data();match self.state {
   20|22=>{
    let w=d.weapons.iter().filter(|w|l.state.route_items.contains_key(&w.rust)).min_by_key(|w|w.sort).ok_or_else(||crate::SimError("No rusty weapon to restore".into()))?;
    l.state.route_items.remove(&w.rust);l.state.route_items.entry(w.item.clone()).or_insert(1);l.state.set_flag(w.flag,true);self.last_weapon=Some(w.item.clone());
    l.boundary(format!("Armsmaster converted {} to {}",w.rust,w.item));
   },
   21=>{l.state.route_items.insert("ITM_EQP_JOB_0008".into(),1);l.state.job_licenses.insert(8,1);l.state.set_flag(d.license_flag,true);l.state.set_flag(406,false);l.boundary("Armsmaster license acquired");},
   23=>{
    let item=self.last_weapon.take().ok_or_else(||crate::SimError("Missing restored weapon for ability".into()))?;
    let w=d.weapons.iter().find(|w|w.item==item).unwrap();l.state.armsmaster_abilities.insert(w.ability.clone());
    for p in l.state.character_progression.values_mut(){
     let learned=p.jobs.learned_abilities.entry(8).or_default();learned.extend(d.job.commands.iter().filter(|a|a.count==1).map(|a|a.id.clone()));learned.extend(l.state.armsmaster_abilities.iter().cloned());
     for a in &d.job.supports {if learned.len()>=a.count {p.jobs.unlocked_support.insert(a.id.clone());}}
    }
    l.boundary(format!("Armsmaster ability {} unlocked globally; support unlocks updated, none equipped",w.ability));
   },_=>return Err(crate::SimError(format!("Unsupported Armsmaster apply state {}",self.state))),
  }Ok(())
 }
}

#[cfg(test)]mod tests {
 use super::*;
 #[test]fn gravell_skips_old_man_and_preserves_wellgrove_supplies() {
  let v:serde_json::Value=serde_json::from_str(include_str!("../data/source/galdera-flex-entry.json")).unwrap();
  let rng:CrtRng=serde_json::from_value(v["after"].clone()).unwrap();
  let mut w:world::WorldState=serde_json::from_value(v["world_after"].clone()).unwrap();
  w.set_flag(data().first_talk_flag,false);w.set_flag(data().license_flag,false);
  for weapon in &data().weapons {w.route_items.remove(&weapon.item);w.route_items.insert(weapon.rust.clone(),1);}
  w.route_items.remove("ITM_CSM_0240");
  for id in ["ITM_CSM_0670","ITM_MRL_COM_0010"] {w.route_items.insert(id.into(),1);}
  // This fixture predates the new post-recruitment Collect purchase.
  crate::gravell_setup::learn(&mut w,6,0,&["ABI_SET_MER_030"]).unwrap();
  crate::nameless_setup::track_augmentation_for_legacy_fixture(&mut w);
  for id in ["ITM_EQP_SWD_380","ITM_EQP_HLM_180"] {w.route_items.insert(id.into(),1);}
  w.swooned_route_npcs.remove("NPC_Twn_Wld_3_1_A_TALK_1600_D000");
  let e=Entry {arrival:rng,after:rng,world_after:w,trace:vec![],decisions:vec![]};
  let g=finish(&e).unwrap();
  assert_eq!(g.setup.arrival,g.arrival.after);assert_eq!(g.setup.after,g.arrival.after);
  assert_eq!(g.night.arrival,g.setup.after);
  for id in ["ITM_CSM_0070","ITM_MRL_COM_0020","ITM_CSM_0240","ITM_CSM_0670","ITM_MRL_COM_0010"] {assert_eq!(g.jam_steal.world_after.route_items.get(id),e.world_after.route_items.get(id));}
  assert_eq!(g.sale_proceeds,250000);
  assert_eq!(g.jam_steal.arrival,g.sales.after);
  assert_eq!(g.jam_steal.after.draws-g.jam_steal.arrival.draws,2);
  let o=&g.jam_steal.world_after.character_progression[&6];
  assert_eq!(o.jobs.equipped_route_items.len(),4);
  assert_eq!(o.jobs.equipped_route_items["Sword"],"ITM_EQP_SWD_380");
  assert!(o.character.combat.as_ref().unwrap().nuts.is_empty());
  assert!(g.jam_steal.world_after.character_progression[&7].character.combat.as_ref().unwrap().nuts.is_empty());
  let mut l=Loader::new(world::catalog(),g.jam_steal.world_after.clone(),g.jam_steal.after).unwrap();
  let once=crate::state_key::WorldKey::new(&l.state);
  crate::nameless_setup::equip_gravell_osvald(&mut l).unwrap();
  assert_eq!(crate::state_key::WorldKey::new(&l.state),once);assert_eq!(l.rng,g.jam_steal.after);
 }
}
