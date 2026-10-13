//! Warrior Osvald / Dancer Temenos, Sai return and one Western Sai Escape.
use crate::{Result,ensure,CrtRng,world::{self,Loader},snows_escape::Entry,galdera_coast::{self,SouthernSai}};
use serde::Serialize;
#[derive(Debug,Serialize)]pub struct Trip {pub setup:Entry,pub sai:Entry,pub southern_sai:Entry,pub western_sai:SouthernSai,pub quicksand:Entry}
fn point(l:&mut Loader<'_>,arrival:CrtRng)->Entry {Entry {arrival,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}}
pub(crate) fn finish(entry:&Entry)->Result<Option<Trip>> {
 ensure(entry.world_after.party==[7,6,5,4]&&entry.world_after.time==2,"Quicksand trip requires daytime party after Temenos recruitment")?;
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 crate::gravell_setup::assign(&mut l.state,6,Some(2))?;
 crate::gravell_setup::assign(&mut l.state,5,Some(5))?;
 l.state.refresh_support_skills();
 l.boundary("Osvald secondary job: Warrior; Temenos secondary job: Dancer; existing supports retained");
 let setup=point(&mut l,entry.after);
 l.visit_map("Twn_Dst_2_1_A")?;l.skipped_event("MapInStorySelect")?;
 let sai=point(&mut l,setup.after);
 l.visit_map("Fld_Dst_2_3")?;let southern_sai=point(&mut l,sai.after);
 // User specifies no Southern encounter and one Western encounter; distances unmeasured.
 l.visit_map("Fld_Dst_2_2")?;let western=point(&mut l,southern_sai.after);
 let Some(western_sai)=galdera_coast::sai_encounter(western,"ENT_FLD_DST_2_2_LND")? else {return Ok(None);};
 let mut l=Loader::new(world::catalog(),western_sai.field_return.world_after.clone(),western_sai.field_return.after)?;
 l.visit_map("Dng_Dst_2_2")?;let quicksand=point(&mut l,western_sai.field_return.after);
 Ok(Some(Trip {setup,sai,southern_sai,western_sai,quicksand}))
}
