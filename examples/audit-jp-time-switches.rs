//! Compare native time-switch draws at the post-restoration Galdera state.
use ot2_speedrun_assistant::{CrtRng,world::{self,Loader,WorldState}};
use serde_json::Value;
fn find(v:&Value)->Option<&Value> {
 if v["restoration"].is_object() && v["jam_steal"].is_object(){return Some(&v["jam_steal"]);}
 v.as_object()?.values().find_map(find)
}
fn main()->Result<(),Box<dyn std::error::Error>> {
 let input:Value=serde_json::from_str(&std::fs::read_to_string(std::env::args().nth(1).ok_or("export path required")?)?)?;
 let e=find(&input).ok_or("Gravell restoration not found")?;
 let state:WorldState=serde_json::from_value(e["world_after"].clone())?;
 let rng:CrtRng=serde_json::from_value(e["after"].clone())?;
 for (name,map) in [("Gravell","Twn_Wld_3_1_A"),("Sai","Twn_Dst_2_1_A"),("Crackridge","Twn_Wld_2_1_A"),("Tropu'hopu","Twn_Isd_2_1_A")] {
  let mut l=Loader::new(world::catalog(),state.clone(),rng)?;
  l.visit_map(map)?;l.skipped_event("MapInStorySelect")?;
  let load=l.rng.draws-rng.draws;
  let mut deltas=vec![];
  for time in [4,2,4,2,4,2] {
   let before=l.rng.draws;l.manual_time_switch(time,"audit")?;deltas.push(l.rng.draws-before);
  }
  println!("{name}: arrival {load}; D/N,N/D repeated {deltas:?}");
 }
 Ok(())
}
