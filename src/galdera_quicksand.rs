//! Warrior Osvald / Dancer Temenos, Sai return and one Western Sai Escape.
use crate::{Result,ensure,CrtRng,world::{self,Loader},snows_escape::Entry,galdera_coast::{self,SouthernSai}};
use serde::Serialize;
#[derive(Debug,Serialize)]pub struct Trip {pub setup:Entry,pub sai:Entry,pub southern_sai:Entry,pub western_sai:SouthernSai,pub quicksand:Entry,pub lord_sands:LordSands}
#[derive(Debug,Serialize)]pub struct LordSands {pub night:Entry,pub prompt:Entry,pub battle:ot2_battle_engine::lord_sands::Prediction,pub field_return:Entry,pub victory:Entry}
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
 let lord_sands=finish_boss(&quicksand)?;
 Ok(Some(Trip {setup,sai,southern_sai,western_sai,quicksand,lord_sands}))
}

fn finish_boss(entry:&Entry)->Result<LordSands> {
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 ensure(!l.state.flag(40010),"Lord of the Sands already defeated")?;
 // User confirmed no random encounters or other actions between entry and boss.
 l.manual_time_switch(4,"Quicksand Gaol: switch to night before Lord of the Sands")?;
 let night=point(&mut l,entry.after);
 let request=l.skipped_event_until_battle("EV_Dng_Dst_2_2_0010",&world::EventChoices::from([(("EV_Dng_Dst_2_2_0010".into(),0),"eDIALOG_YES".into())]))?;
 ensure(request.volume=="EVM_DNG_DST_2_2_L","Unexpected boss volume")?;
 let prompt=point(&mut l,night.after);
 let players:Vec<_>=l.state.party.iter().map(|id|l.state.character_progression[id].character.clone()).collect();
 let battle=ot2_battle_engine::lord_sands::predict(l.rng,&players,&l.state.enemy_knowledge)?;
 for (&id,r) in l.state.party.iter().zip(&battle.party_resources_after){l.state.character_progression.get_mut(&id).unwrap().character.resources=Some(r.clone());}
 l.state.enemy_knowledge=battle.enemy_knowledge.clone();l.state.award_victory(battle.rewards)?;
 for (item,&n) in &battle.drops {*l.state.route_items.entry(item.clone()).or_default()+=n;}
 l.rng=battle.after;l.restore_field_after_battle("Lord of the Sands victory: field restored")?;
 let field_return=point(&mut l,battle.after);
 l.skipped_event(&request.victory_event)?;
 ensure(l.state.flag(40010),"Boss defeat flag not applied")?;
 let victory=point(&mut l,field_return.after);
 Ok(LordSands {night,prompt,battle,field_return,victory})
}
