//! Hikari's measured walking-time switch windows. Every branch runs real loaders.
use crate::{Result,CrtRng,ensure,world::{self,Loader},snows_escape::Entry,state_key::WorldKey};
use serde::Serialize;
use std::collections::BTreeMap;
use std::{cell::RefCell,rc::Rc,collections::VecDeque};
type Rows=Vec<([u8;7],Entry,u32)>;
type CacheKey=(u32,u64,WorldKey);
thread_local!{static CACHE:RefCell<VecDeque<(CacheKey,Rc<Rows>)>>=const{RefCell::new(VecDeque::new())};}
pub(crate) fn cached_candidates(entry:&Entry)->Result<Rc<Rows>> {
 let key=(entry.after.state,entry.after.draws,WorldKey::new(&entry.world_after));
 if let Some(rows)=CACHE.with(|c|c.borrow().iter().find(|(k,_)|k==&key).map(|(_,r)|Rc::clone(r))){return Ok(rows);}
 let rows=Rc::new(world::without_diagnostics(||candidates(entry))?);
 CACHE.with(|c|{let mut c=c.borrow_mut();if c.len()==8{c.pop_front();}c.push_back((key,Rc::clone(&rows)));});
 Ok(rows)
}
pub const CAPACITIES:[u8;7]=[2,2,1,2,2,2,3];
pub const WINDOWS:[&str;7]=[
 "Clockbank main town → Industrial District",
 "Industrial District entry → Will/Ori scenes",
 "Will/Ori scenes → factory/Roque scenes",
 "Factory/Roque scenes → headquarters scenes",
 "Headquarters scenes → first factory entry",
 "First factory exit → weaponsmith scene",
 "Weaponsmith scene → second factory entry",
];
#[derive(Debug,Clone,Serialize)]
pub struct Window {pub label:String,pub switches:u8,pub end_time:u8}
#[derive(Debug,Clone,Serialize)]
pub struct Plan {
 pub counts:[u8;7],pub windows:Vec<Window>,pub return_to_day:bool,pub input_cost:u32,
 pub stages:Vec<(String,Entry)>,pub departure:Entry,
}
fn point(l:&mut Loader<'_>,before:CrtRng)->Entry {
 Entry{arrival:before,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
fn step(entry:&Entry,index:usize,count:u8)->Result<(Entry,Window)> {
 ensure(count<=CAPACITIES[index],"Clockbank free-switch capacity exceeded")?;
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 for _ in 0..count {l.manual_time_switch(if l.state.time==2{4}else{2},WINDOWS[index])?;}
 let window=Window{label:WINDOWS[index].into(),switches:count,end_time:l.state.time};
 match index {
  0=>l.visit_map("Twn_Cty_2_1_B")?,
  1=>l.skipped_event("MS_SHO_20_0100")?,
  2=>l.skipped_event("MS_SHO_20_0200")?,
  3=>l.skipped_event("MS_SHO_20_0400")?,
  4=>l.skipped_event("MS_SHO_20_0500")?,
  5=>l.skipped_event("MS_SHO_20_0610")?,
  6=>{
   l.visit_map("Dng_Cty_2_1")?;
   l.skipped_event_with_choices("MS_SHO_20_0615",&world::EventChoices::from([(("MS_SHO_20_0615".into(),2),"eDIALOG_YES".into())]))?;
   ensure(l.state.flag(6059)&&l.state.flag(6077)&&l.state.map=="Twn_Cty_2_1_B","Clockite delivery did not finish")?;
  },_=>unreachable!(),
 }
 Ok((point(&mut l,entry.after),window))
}
pub fn replay(entry:&Entry,counts:[u8;7])->Result<Plan> {
 let mut current=entry.clone();let mut stages=vec![];let mut windows=vec![];
 for (i,count) in counts.into_iter().enumerate(){
  let (next,window)=step(&current,i,count)?;
  stages.push((WINDOWS[i].into(),next.clone()));windows.push(window);current=next;
 }
 let return_to_day=current.world_after.time==4;
 if return_to_day {
  let mut l=Loader::new(world::catalog(),current.world_after.clone(),current.after)?;
  l.manual_time_switch(2,"Clockite delivered: return to day before Crackridge Harbor")?;
  current=point(&mut l,current.after);stages.push(("After Clockite delivery · switch to day".into(),current.clone()));
 }
 Ok(Plan{counts,windows,return_to_day,input_cost:if return_to_day{12}else{0},stages,departure:current})
}
/// Merge only exact mutable world/RNG states; the timing windows themselves cost zero.
pub fn candidates(entry:&Entry)->Result<Vec<([u8;7],Entry,u32)>> {
 let mut nodes=vec![([0;7],entry.clone())];
 for (i,&cap) in CAPACITIES.iter().enumerate(){
  let mut next=BTreeMap::new();
  for (counts,e) in nodes {for count in 0..=cap {
   crate::cancellation::check()?;
   let (out,_)=step(&e,i,count)?;let mut counts=counts;counts[i]=count;
   next.entry((out.after.state,out.after.draws,WorldKey::new(&out.world_after))).or_insert((counts,out));
  }}
  nodes=next.into_values().collect();
 }
 let mut result=vec![];
 for (counts,e) in nodes {
  let mut l=Loader::new(world::catalog(),e.world_after.clone(),e.after)?;
  let cost=if l.state.time==4 {l.manual_time_switch(2,"Clockite delivered: return to day before Crackridge Harbor")?;12}else{0};
  result.push((counts,point(&mut l,e.after),cost));
 }
 Ok(result)
}

#[cfg(test)]mod tests {
 use super::*;
 fn entry()->Entry {
  let mut w:world::WorldState=serde_json::from_str(include_str!("../data/source/clockbank-search-party.json")).unwrap();
  for flag in 6050..=6077{w.set_flag(flag,false);}w.set_flag(6076,true);
  w.map="Fld_Cty_2_1".into();w.time=2;
  let mut l=Loader::new(world::catalog(),w,CrtRng::new(12345)).unwrap();
  l.visit_map("Twn_Cty_2_1_A").unwrap();point(&mut l,CrtRng::new(12345))
 }
 #[test]fn odd_windows_preserve_real_night_loads_and_replay(){
  let e=entry();let p=replay(&e,[0,1,0,0,0,0,0]).unwrap();
  assert_eq!(p.windows[4].end_time,4);assert_eq!(p.windows[6].end_time,4);
  assert!(p.return_to_day);assert_eq!(p.input_cost,12);assert_eq!(p.departure.world_after.time,2);
  assert!(p.departure.world_after.flag(6077));
  let all=replay(&e,CAPACITIES).unwrap();assert_eq!(all.counts.iter().sum::<u8>(),14);assert_eq!(all.input_cost,0);
  assert_eq!(all.windows[4].end_time,4);assert_eq!(all.windows[6].end_time,2);
 }
 #[test]fn frontier_keeps_zero_and_full_profiles_and_matches_exact_replay(){
  let e=entry();let rows=candidates(&e).unwrap();
  for counts in [[0;7],CAPACITIES,[1,0,0,0,0,0,0]] {
   let p=replay(&e,counts).unwrap();assert!(rows.iter().any(|(_,x,c)|x.after==p.departure.after&&*c==p.input_cost&&WorldKey::new(&x.world_after)==WorldKey::new(&p.departure.world_after)));
  }
  for (counts,x,c) in rows {let p=replay(&e,counts).unwrap();assert_eq!(p.departure.after,x.after);assert_eq!(p.input_cost,c);}
 }
}
