use ot2_speedrun_assistant::{world::{self,WorldState,Loader},CrtRng};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let v:serde_json::Value=serde_json::from_str(&std::fs::read_to_string("outputs/bg-planning-inventor.json")?)?;
 let e=&v["clockite_stages"][7][1];let w:WorldState=serde_json::from_value(e["world_after"].clone())?;
 let base:CrtRng=serde_json::from_value(e["after"].clone())?;
 let mut samples=vec![];
 for i in 0..128 {
  let mut rng=base;rng.advance(i*97);
  let mut l=Loader::new(world::catalog(),w.clone(),rng)?;
  for repeat in 0..3 {
   let before=l.rng.draws;l.visit_map("Twn_Cty_2_1_A")?;let middle=l.rng.draws;
   l.visit_map("Twn_Cty_2_1_B")?;let end=l.rng.draws;
   samples.push(end-before);
   if i==0 {println!("reload {repeat}: main {} industrial {} total {}",middle-before,end-middle,end-before);}
  }
 }
 samples.sort();println!("n={} min={} max={} mean={}",samples.len(),samples[0],samples.last().unwrap(),samples.iter().sum::<u64>() as f64/samples.len() as f64);
 let mut l=Loader::new(world::catalog(),w,base)?;
 for time in [4,2] {let before=l.rng.draws;l.manual_time_switch(time,"audit")?;println!("switch {time}: {}",l.rng.draws-before);}
 Ok(())
}
