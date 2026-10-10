use ot2_speedrun_assistant::{conning,world::{self,Loader,WorldState},CrtRng};
use ot2_battle_engine::field_escape::FieldEscapeMenu;
fn main()->Result<(),Box<dyn std::error::Error>> {
 let j:serde_json::Value=serde_json::from_str(&std::fs::read_to_string("outputs/conning-route.json")?)?;
 let n=&j["cropdale_exit"]["oresrush"]["ryu"]["canalbrine"]["conning"]["northern"];
 let w:WorldState=serde_json::from_value(n["arrival"]["night"]["world_after"].clone())?;
 let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
 let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
 let ids:Vec<_>=n["arrival"]["encounter"]["enemies"].as_array().unwrap().iter().map(|s|s["enemy"]["id"].as_str().unwrap()).collect();
 let situation=if n["arrival"]["encounter"]["situation"]=="Preemptive" {ot2_battle_engine::escape::Situation::Preemptive}else{ot2_battle_engine::escape::Situation::Normal};
 let menu=FieldEscapeMenu::with_players(serde_json::from_value(n["arrival"]["battle_entry"].clone())?,situation,&players,&levels,&ids,&w.enemy_knowledge)?;
 let danger=n["arrival"]["encounter"]["encounter_level"].as_u64().unwrap() as u32;
 let mut successes=0;let mut cats=vec![];let mut surprises=0;
 for boost in 0..=231 {
  let a=menu.clone().attempt(&conning::path(boost),danger)?;
  if !a.success {continue;} successes+=1;
  let mut state=w.clone();state.first_encounter=false;state.enemy_knowledge=a.enemy_knowledge;
  let mut l=Loader::new(world::catalog(),state,a.after)?;
  l.restore_field_after_battle("audit")?;l.manual_time_switch(2,"audit day")?;
  let west=conning::arrival(&l.state,l.rng,conning::WEST)?;
  let surprise=west.encounter.situation.as_deref()==Some("Surprise");
  surprises+=u32::from(surprise);
  if west.encounter.enemies.iter().any(|s|s.enemy.id=="ENE_NML_OTR_140") {cats.push((boost,west.encounter.situation));}
 }
 println!("Escape successes {successes}/232; western surprises among successes {surprises}; Cait candidates {cats:?}");
 let _:Option<CrtRng>=None;
 Ok(())
}
