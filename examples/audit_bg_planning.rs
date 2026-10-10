use ot2_speedrun_assistant::{world::{self,WorldState,Loader},CrtRng};
use ot2_battle_engine::{gravell_routine::{OpeningPlan},escape::Situation,field_escape::carried_step_ahead};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let v:serde_json::Value=serde_json::from_str(&std::fs::read_to_string("outputs/bg-planning-inventor.json")?)?;
 for index in [0,7] {
  let e=&v["clockite_stages"][index][1];let w:WorldState=serde_json::from_value(e["world_after"].clone())?;
  let rng:CrtRng=serde_json::from_value(e["after"].clone())?;let mut l=Loader::new(world::catalog(),w,rng)?;
  for time in [4,2,4,2] {let before=l.rng;l.manual_time_switch(time,"audit")?;println!("Clockite stage {index} -> {time}: {}",l.rng.draws-before.draws);}
 }
 let mut e=&v["crackridge_return"];while !e["southern"].is_null(){e=&e["southern"];}
 let g=&e["gravell"];let w:WorldState=serde_json::from_value(g["night"]["world_after"].clone())?;
 let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
 let ids:Vec<_>=g["encounter"]["enemies"].as_array().unwrap().iter().map(|e|e["enemy"]["id"].as_str().unwrap()).collect();
 let mut samples=vec![];
 for seed in 0..512 {
  let mut b=carried_step_ahead(CrtRng::new(seed),Situation::Normal,&players,&ids,true,&w.enemy_knowledge)?;
  let plan=OpeningPlan::from_battle(&b)?;
  for p in plan.bonus_turn {b.step_player(p.action.command())?;}
  if b.state().outcome==ot2_battle_engine::Outcome::Ongoing {samples.push(b.state().rng.draws);}
 }
 samples.sort();println!("Battle entry -> BG menu: n={} mean={} min={} p95={} max={}",samples.len(),samples.iter().sum::<u64>() as f64/samples.len() as f64,samples[0],samples[samples.len()*95/100],samples.last().unwrap());
 println!("Saved route Clockbank exit -> Gravell battle entry: {}",g["battle_entry"]["draws"].as_u64().unwrap()-v["clockite_stages"][7][1]["after"]["draws"].as_u64().unwrap());
 Ok(())
}
