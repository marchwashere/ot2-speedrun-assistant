use ot2_speedrun_assistant::{world::{self,Loader,WorldState},CrtRng};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let v:serde_json::Value=serde_json::from_str(&std::fs::read_to_string("outputs/bg-planning-inventor.json")?)?;
 for stage in v["clockite_stages"].as_array().unwrap() {
  let w:WorldState=serde_json::from_value(stage[1]["world_after"].clone())?;
  let base:CrtRng=serde_json::from_value(stage[1]["after"].clone())?;
  let mut totals=vec![vec![];4];
  for sample in 0..64 {
   let mut r=base;r.advance(sample*97);
   let mut l=Loader::new(world::catalog(),w.clone(),r)?;
   for (i,t) in [4,2,4,2].into_iter().enumerate() {
    let b=l.rng.draws;l.manual_time_switch(t,"checkpoint audit")?;
    totals[i].push(l.rng.draws-b);
   }
  }
  println!("{} | {} | {:?}",stage[0],w.map,totals.iter().map(|x|(*x.iter().min().unwrap(),*x.iter().max().unwrap())).collect::<Vec<_>>());
 }
 for (idx,event) in [(0,"MS_SHO_20_0100"),(1,"MS_SHO_20_0200"),(2,"MS_SHO_20_0400"),(3,"MS_SHO_20_0500"),(4,"MS_SHO_20_0610"),(6,"MS_SHO_20_0615")] {
  let e=&v["clockite_stages"][idx][1];
  for time in [2,4] {
   let w:WorldState=serde_json::from_value(e["world_after"].clone())?;
   let base:CrtRng=serde_json::from_value(e["after"].clone())?;
   let mut l=Loader::new(world::catalog(),w,base)?;
   if l.state.time!=time {l.manual_time_switch(time,"entry time")?;}
   l.rng=base; // Compare event costs from the same RNG; exclude setup switching.
   let before=l.rng.draws;
   l.skipped_event_with_choices(event,&world::EventChoices::from([(("MS_SHO_20_0615".into(),2),"eDIALOG_YES".into())]))?;
   println!("event {event} entry time {time}: draws={} exit_time={} map={}",l.rng.draws-before,l.state.time,l.state.map);
  }
 }
 let e=&v["clockite_stages"][0][1];
 let w:WorldState=serde_json::from_value(e["world_after"].clone())?;
 let base:CrtRng=serde_json::from_value(e["after"].clone())?;
 for at in 0..8 {
  let mut l=Loader::new(world::catalog(),w.clone(),base)?;
  for checkpoint in 0..8 {
   if checkpoint==at {l.manual_time_switch(4,"pair night")?;l.manual_time_switch(2,"pair day")?;}
   match checkpoint {
    0=>l.skipped_event("MS_SHO_20_0100")?,
    1=>l.skipped_event("MS_SHO_20_0200")?,
    2=>l.skipped_event("MS_SHO_20_0400")?,
    3=>l.skipped_event("MS_SHO_20_0500")?,
    4=>l.skipped_event("MS_SHO_20_0610")?,
    5=>l.visit_map("Dng_Cty_2_1")?,
    6=>l.skipped_event_with_choices("MS_SHO_20_0615",&world::EventChoices::from([(("MS_SHO_20_0615".into(),2),"eDIALOG_YES".into())]))?,
    _=>(),
   }
  }
  println!("pair at {at}: endpoint {:08X} draws={} time={}",l.rng.state,l.rng.draws,l.state.time);
 }
 Ok(())
}
