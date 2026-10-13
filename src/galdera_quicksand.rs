//! Warrior Osvald / Dancer Temenos, Sai return and one Western Sai Escape.
use crate::{Result,ensure,CrtRng,world::{self,Loader},snows_escape::Entry,galdera_coast::{self,SouthernSai}};
use serde::Serialize;
#[derive(Debug,Serialize)]pub struct Trip {pub setup:Entry,pub sai:Entry,#[serde(skip_serializing_if="Option::is_none")]pub adjustment:Option<Entry>,pub additional_time_switches:u32,pub southern_sai:Entry,pub western_sai:SouthernSai,pub quicksand:Entry}
fn point(l:&mut Loader<'_>,arrival:CrtRng)->Entry {Entry {arrival,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}}
pub(crate) fn finish(entry:&Entry)->Result<Trip> {
 ensure(entry.world_after.party==[7,6,5,4]&&entry.world_after.time==2,"Quicksand trip requires daytime party after Temenos recruitment")?;
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 crate::gravell_setup::assign(&mut l.state,6,Some(2))?;
 crate::gravell_setup::assign(&mut l.state,5,Some(5))?;
 l.state.refresh_support_skills();
 l.boundary("Osvald secondary job: Warrior; Temenos secondary job: Dancer; existing supports retained");
 let setup=point(&mut l,entry.after);
 l.visit_map("Twn_Dst_2_1_A")?;l.skipped_event("MapInStorySelect")?;
 let sai=point(&mut l,setup.after);let mut departure=sai.clone();
 let mut additional_time_switches=0;let mut adjustment:Option<Entry>=None;
 let (southern_sai,western_sai)=loop {
  let mut route=Loader::new(world::catalog(),departure.world_after.clone(),departure.after)?;
  route.visit_map("Fld_Dst_2_3")?;let southern=point(&mut route,departure.after);
  // User specifies no Southern Sai encounter and one Western Sai encounter.
  // No distance measurements were supplied; do not invent movement costs.
  route.visit_map("Fld_Dst_2_2")?;let western=point(&mut route,southern.after);
  if let Some(fight)=galdera_coast::sai_encounter(western,"ENT_FLD_DST_2_2_LND")? {break (southern,fight);}
  l.manual_time_switch(4,"Sai before Quicksand: extra night switch to avoid ambush")?;
  l.manual_time_switch(2,"Sai before Quicksand: return to day")?;additional_time_switches+=2;
  let next=point(&mut l,departure.after);
  if let Some(a)=&mut adjustment {a.after=next.after;a.world_after=next.world_after;a.trace.extend(next.trace);a.decisions.extend(next.decisions);}else{adjustment=Some(next);}
  departure=adjustment.as_ref().unwrap().clone();
 };
 let mut l=Loader::new(world::catalog(),western_sai.field_return.world_after.clone(),western_sai.field_return.after)?;
 l.visit_map("Dng_Dst_2_2")?;let quicksand=point(&mut l,western_sai.field_return.after);
 Ok(Trip {setup,sai,adjustment,additional_time_switches,southern_sai,western_sai,quicksand})
}
