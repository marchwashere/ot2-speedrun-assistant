//! Temenos recruitment after the encounterless Flamechurch approach.
use crate::{Result,ensure,CrtRng,world::{self,Loader,CharacterProgression},snows_escape::Entry};
use serde::Serialize;
#[derive(Debug,Serialize)]pub struct Trip {pub introduction:Entry,pub battle:ot2_battle_engine::temenos_recruitment::Prediction,pub field_return:Entry,pub recruitment:Entry,pub quicksand:crate::galdera_quicksand::Trip}
fn point(l:&mut Loader<'_>,arrival:CrtRng)->Entry {Entry {arrival,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}}
pub(crate) fn finish(entry:&Entry)->Result<Trip> {
 ensure(entry.world_after.map=="Twn_Mnt_1_1_A"&&entry.world_after.time==2&&entry.world_after.party==[7,6,8,4],"Temenos recruitment requires daytime Flamechurch with original Galdera party")?;
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 l.state.sync_progression()?;
 let intro="MS_SIN_00_0100";let victory="MS_SIN_00_0300";
 let request=l.skipped_event_until_battle(intro,&world::EventChoices::from([((intro.into(),59),"eDIALOG_YES".into())]))?;
 ensure(request.volume=="ENM_EVE_CLE_C00_010"&&request.victory_event==victory,"Unexpected Temenos encounter")?;
 let introduction=point(&mut l,entry.after);
 let players:Vec<_>=l.state.party.iter().map(|id|l.state.character_progression[id].character.clone()).collect();
 let mut minimum_cycles=0;
 loop {
 let mut l=Loader::new(world::catalog(),introduction.world_after.clone(),introduction.after)?;
 let battle=ot2_battle_engine::temenos_recruitment::predict_from_boost_cycles(l.rng,&players,&l.state.enemy_knowledge,minimum_cycles)?;
 minimum_cycles=battle.boost_cycles+1;
 for (&id,r) in l.state.party.iter().zip(&battle.party_resources_after){l.state.character_progression.get_mut(&id).unwrap().character.resources=Some(r.clone());}
 l.state.enemy_knowledge=battle.enemy_knowledge.clone();l.state.award_victory(battle.rewards)?;
 for (item,&n) in &battle.drops {*l.state.route_items.entry(item.clone()).or_default()+=n;}
 l.rng=battle.after;l.restore_field_after_battle("Temenos recruitment victory: field restored before recruitment scenes")?;
 let field_return=point(&mut l,battle.after);
 let character=ot2_battle_engine::field_escape::route_player(5,0)?;let growth=ot2_battle_engine::progression::growth(5,0)?.clone();
 l.state.character_progression.insert(5,CharacterProgression {experience:0,jp_earned:0,jp_spent:0,level:growth.level,growth,character,jobs:Default::default()});
 l.skipped_event_with_choices(victory,&world::EventChoices::from([((victory.into(),46),"eDIALOG_YES".into()),((victory.into(),58),"eDIALOG_NO".into()),((victory.into(),68),"8".into())]))?;
 ensure(l.state.party==[7,6,5,4]&&l.state.bench.contains(&8)&&l.state.evasive_maneuvers,"Temenos did not replace Agnea correctly")?;
 let recruitment=point(&mut l,field_return.after);
 if let Some(quicksand)=crate::galdera_quicksand::finish(&recruitment)? {
 return Ok(Trip {introduction,battle,field_return,recruitment,quicksand});
 }
 }

}
#[cfg(test)]mod tests {
 use super::*;
 #[test]fn recruitment_boosts_repair_western_sai_ambush() {
  let f:serde_json::Value=serde_json::from_str(include_str!("../data/source/gravell-complete-route-fixture.json")).unwrap();
  let mut w:world::WorldState=serde_json::from_value(f["before"]["world_after"].clone()).unwrap();
  w.job_licenses.insert(2,1);w.job_licenses.insert(5,1);w.party=vec![7,6,8,4];w.bench=vec![1,3];
  w.party_experience=w.party.iter().map(|id|w.character_progression[id].experience).collect();
  w.character_progression.get_mut(&6).unwrap().jobs.equipped_support.insert("ABI_SCH_SUP_01".into());
  w.award_victory(ot2_battle_engine::progression::Reward {exp:300_000,jp:0,money:0}).unwrap();
  w.time=2;w.galdera_route=true;
  for seed in 0u32..128 {
   let rng=CrtRng::new(seed.wrapping_mul(1775700753));
   let mut loaded=Loader::new(world::catalog(),w.clone(),rng).unwrap();
   loaded.visit_map("Twn_Mnt_1_1_A").unwrap();
   let entry=point(&mut loaded,rng);
   let t=finish(&entry).unwrap();
   let players:Vec<_>=t.introduction.world_after.party.iter().map(|id|t.introduction.world_after.character_progression[id].character.clone()).collect();
   let base=ot2_battle_engine::temenos_recruitment::predict(t.introduction.after,&players,&t.introduction.world_after.enemy_knowledge).unwrap();
   assert_eq!(base.at_command_menu,t.battle.at_command_menu);
   if t.battle.boost_cycles==base.boost_cycles {continue;}
   let replay=ot2_battle_engine::temenos_recruitment::predict_from_boost_cycles(t.introduction.after,&players,&t.introduction.world_after.enemy_knowledge,t.battle.boost_cycles).unwrap();
   assert_eq!(replay.after,t.battle.after);
   assert_eq!(replay.boost_cycles,t.battle.boost_cycles);
   assert_eq!(t.field_return.arrival,replay.after);
   assert_eq!(t.recruitment.arrival,t.field_return.after);
   let q=&t.quicksand;
   assert_eq!(q.setup.arrival,t.recruitment.after);
   assert_eq!(q.southern_sai.arrival,q.sai.after);
   let mut l=Loader::new(world::catalog(),q.sai.world_after.clone(),q.sai.after).unwrap();
   l.visit_map("Fld_Dst_2_3").unwrap();assert_eq!(l.rng,q.southern_sai.after);
   l.visit_map("Fld_Dst_2_2").unwrap();assert_eq!(l.rng,q.western_sai.arrival.after);
   assert_ne!(q.western_sai.encounter.situation.as_deref(),Some("Surprise"));
   assert!(q.western_sai.escape.success);assert_eq!(q.western_sai.escape.chance_percent,100);
   assert_eq!(q.quicksand.world_after.time,2);
   eprintln!("seed {seed}: kill-only cycles {}, safe-route cycles {}",base.boost_cycles,t.battle.boost_cycles);
   return;
  }
  panic!("Must exercise downstream ambush rejection");
 }
}
