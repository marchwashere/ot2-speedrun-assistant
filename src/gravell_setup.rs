//! Source-backed job/JP/support setup after the Merchant guild.
use crate::{Result,ensure,world::{WorldState,Loader},snows_escape::Entry};
use serde::Deserialize;
use std::sync::OnceLock;

pub const STEP_AHEAD:&str="ABI_INV_SUP_01";
pub const MONEY_UP:&str="ABI_MER_SUP_01";
#[derive(Deserialize)]
struct Ability { #[serde(rename="AbilityName")] id:String, #[serde(rename="GetParam")] count:usize }
#[derive(Deserialize)]
struct Job { #[serde(rename="JobID")] id:u32, #[serde(rename="JobCommandAbility")] commands:Vec<Ability>,
 #[serde(rename="JobSupportAbility")] supports:Vec<Ability>, #[serde(rename="JPCost")] costs:Vec<u32> }
#[derive(Deserialize)]
struct Data {jobs:Vec<Job>}
fn data()->&'static Data {static D:OnceLock<Data>=OnceLock::new();D.get_or_init(||serde_json::from_str(include_str!("../data/gravell-setup.json")).unwrap())}
pub(crate) fn unlock(w:&mut WorldState,id:u32,job:u32)->Result<()> {
 let d=data().jobs.iter().find(|j|j.id==job).unwrap();
 let inventions=w.inventor_abilities.clone();
 let p=w.character_progression.get_mut(&id).unwrap();
 let learned=p.jobs.learned_abilities.entry(job).or_default();
 learned.extend(d.commands.iter().filter(|a|a.count==1).map(|a|a.id.clone()));
 if job==11 {learned.extend(inventions);}
 for a in &d.supports {if learned.len()>=a.count {p.jobs.unlocked_support.insert(a.id.clone());}}
 Ok(())
}
pub(crate) fn assign(w:&mut WorldState,id:u32,job:Option<u32>)->Result<()> {
 if let Some(j)=job {
  ensure(w.job_licenses.get(&j).copied().unwrap_or(0)>0,"Missing subjob license")?;
  let used=w.party.iter().chain(&w.bench).filter(|&&i|i!=id&&w.character_progression[&i].jobs.secondary_job==Some(j)).count();
  ensure(used<usize::from(w.job_licenses[&j]),"Subjob license already in use")?;
 }
 let p=w.character_progression.get_mut(&id).unwrap();
 crate::ensure(p.character.combat.is_some(),"Missing character combat data")?;
 ot2_battle_engine::jobs::change_secondary_job(&mut p.character,id,p.level,p.jobs.secondary_job,job)?;
 p.jobs.secondary_job=job;
 if let Some(j)=job {unlock(w,id,j)?;}
 Ok(())
}
pub(crate) fn learn(w:&mut WorldState,id:u32,job:u32,abilities:&[&str])->Result<()> {
 unlock(w,id,job)?;
 let d=data().jobs.iter().find(|j|j.id==job).unwrap();
 for &ability in abilities {
  let p=w.character_progression.get_mut(&id).unwrap();
  let learned=p.jobs.learned_abilities.get_mut(&job).unwrap();
  if learned.contains(ability) {continue;}
  ensure(d.commands.iter().any(|a|a.id==ability),"Unknown job ability")?;
  let cost=d.costs[learned.len()];
  ensure(p.jp_earned.saturating_sub(p.jp_spent)>=cost,format!("Insufficient JP for {ability}"))?;
  p.jp_spent+=cost;learned.insert(ability.into());
  unlock(w,id,job)?;
 }
 Ok(())
}
pub(crate) fn jp_setup(l:&mut Loader<'_>)->Result<()> {
 let before=l.rng;
 learn(&mut l.state,5,5,&["ABI_SET_DAN_070"])?;
 learn(&mut l.state,4,0,&["ABI_SET_MER_030"])?;
 // Crafted Inventor commands/supports are globally available without job swaps.
 unlock(&mut l.state,5,11)?;
 ensure(l.state.character_progression[&5].jobs.unlocked_support.contains(STEP_AHEAD),"Temenos Step Ahead not unlocked")?;
 l.state.character_progression.get_mut(&5).unwrap().jobs.equipped_support.insert(STEP_AHEAD.into());
 for id in [4,6] {
  let p=l.state.character_progression.get_mut(&id).unwrap();
  ensure(p.jobs.unlocked_support.contains("ABI_MER_SUP_02"),format!("Boost-Start not unlocked for character {id}"))?;
  p.jobs.equipped_support.insert("ABI_MER_SUP_02".into());
 }
 l.state.refresh_support_skills();
 l.boundary("Gravell setup: Temenos learns Bewildering Grace; retain the A Step Ahead and Boost-Start supports equipped after his recruitment");
 ensure(l.rng==before,"JP setup changed RNG")
}
pub fn apply(l:&mut Loader<'_>)->Result<()> {
 let before=l.rng;let counter=l.state.encounter_counter;
 let merchant=if l.state.galdera_route {6} else {1};
 ensure(l.state.party==[7,merchant,8,4],"Unexpected Gravell party")?;
 assign(&mut l.state,7,Some(11))?;
 assign(&mut l.state,merchant,Some(0))?;
 learn(&mut l.state,merchant,0,&["ABI_SET_MER_050","ABI_SET_MER_070"])?;
 learn(&mut l.state,8,5,&["ABI_SET_DAN_060","ABI_SET_DAN_070"])?;
 learn(&mut l.state,4,0,&["ABI_SET_MER_070","ABI_SET_MER_050"])?;
 l.state.refresh_support_skills();
 l.boundary(format!("Gravell setup: Throné -> Inventor; {} Merchant; learned Donate BP/Hired Help, Stimulate/Bewildering Grace, Partitio Hired Help/Donate BP",if merchant==6 {"Osvald"} else {"Hikari"}));
 ensure(l.rng==before&&l.state.encounter_counter==counter,"Job menu setup changed field RNG/counter")
}

/// Keep a snapshot for diagnostics without inventing any battle draws.
pub fn snapshot(l:&mut Loader<'_>,before:crate::CrtRng)->Entry {Entry {arrival:before,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}}

/// Equip the unlocked Inventor support immediately after crafting, preserving jobs.
pub fn equip_step_ahead(l:&mut Loader<'_>)->Result<()> {
 let before=l.rng;let counter=l.state.encounter_counter;
 let party=l.state.party.clone();
 for id in party {
  let previous=l.state.character_progression[&id].jobs.secondary_job;
  assign(&mut l.state,id,Some(11))?;
  let p=l.state.character_progression.get_mut(&id).unwrap();
  ensure(p.jobs.unlocked_support.contains(STEP_AHEAD),"A Step Ahead is not unlocked")?;
  p.jobs.equipped_support.insert(STEP_AHEAD.into());
  ensure(p.jobs.equipped_support.len()<=4,"Too many support skills")?;
  let p=l.state.character_progression.get_mut(&id).unwrap();
  ot2_battle_engine::jobs::change_secondary_job(&mut p.character,id,p.level,Some(11),previous)?;
  p.jobs.secondary_job=previous;
 }
 l.state.refresh_support_skills();
 l.boundary("Inventor crafting complete: equip A Step Ahead on every active character");
 ensure(l.rng==before&&l.state.encounter_counter==counter,"Support setup changed RNG/counter")
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn galdera_gives_osvald_merchant_and_step_ahead_preserving_native_scholar() {
  let fixture:serde_json::Value=serde_json::from_str(include_str!("../data/source/gravell-complete-route-fixture.json")).unwrap();
  let mut w:WorldState=serde_json::from_value(fixture["before"]["world_after"].clone()).unwrap();
  // The original fixture benches Osvald: supply the active Merchant's later
  // JP for this setup test, rather than treating the benched ledger as active.
  let later_jp=w.character_progression[&1].jp_earned;
  let osvald=w.character_progression.get_mut(&6).unwrap();
  osvald.jp_earned+=later_jp;
  osvald.jobs.learned_abilities.insert(6,["ABI_SET_SCH_010","ABI_SET_SCH_020","ABI_SET_SCH_030","ABI_SET_SCH_060"].into_iter().map(str::to_owned).collect());
  w.galdera_route=true;
  w.party=vec![7,6,8,4];w.bench.retain(|&id|id!=6);
  w.character_progression.remove(&1);
  w.character_progression.get_mut(&7).unwrap().jobs.secondary_job=None;
  w.job_licenses.insert(0,1);
  let scholar=w.character_progression[&6].jobs.learned_abilities.clone();
  let osvald_spent=w.character_progression[&6].jp_spent;
  let rng=crate::CrtRng::new(9876);let counter=w.encounter_counter;
  let mut l=Loader::new(crate::world::catalog(),w,rng).unwrap();
  equip_step_ahead(&mut l).unwrap();
  apply(&mut l).unwrap();
  let p=&l.state.character_progression[&6];
  assert_eq!(p.jobs.secondary_job,Some(0));
  assert!(p.jobs.learned_abilities[&0].contains("ABI_SET_MER_050"));
  assert!(p.jobs.learned_abilities[&0].contains("ABI_SET_MER_070"));
  assert_eq!(p.jobs.learned_abilities.get(&6),scholar.get(&6));
  assert_eq!(p.jp_spent,osvald_spent+130);
  assert!(p.jobs.equipped_support.contains(STEP_AHEAD));
  assert!(p.jobs.equipped_support.contains(crate::scholar::EVASIVE));
  assert!(!p.jobs.equipped_support.contains(MONEY_UP));
  assert_eq!(l.state.party,[7,6,8,4]);
  assert_eq!(l.rng,rng);assert_eq!(l.state.encounter_counter,counter);
 }
}
