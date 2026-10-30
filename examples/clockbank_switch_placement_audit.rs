use ot2_speedrun_assistant::{world::{self,Loader,WorldState},CrtRng};
fn main(){
 if std::env::args().any(|a|a=="--search") {
  std::thread::Builder::new().stack_size(32*1024*1024).spawn(||{
   let fixture:serde_json::Value=serde_json::from_str(include_str!("../data/source/retail-inventor-highroad-20261010.json")).unwrap();
   let mut w:WorldState=serde_json::from_value(fixture["world"].clone()).unwrap();w.galdera_route=false;w.time=2;w.sync_progression().unwrap();
   for id in w.party.clone(){let p=&mut w.character_progression.get_mut(&id).unwrap().character;let r=p.resources.as_mut().unwrap();r.hp=p.stats.max_hp;r.sp=p.stats.max_sp;}
   let rng=CrtRng::new(3221999247);
   let e=ot2_speedrun_assistant::snows_escape::Entry{arrival:rng,after:rng,world_after:w,trace:vec![],decisions:vec![]};
   let start=std::time::Instant::now();
   let a=ot2_speedrun_assistant::inventor::approach(&e).unwrap();
   let p=ot2_speedrun_assistant::inventor::finish(a).unwrap();
   let free=p.clockbank_free.as_ref().unwrap();let paid=p.new_delsta_adjustment.as_ref().unwrap();
   println!("Elapsed {:?}, paid={}, free={:?}, cost={}",start.elapsed(),paid.switches,free.counts,paid.input_cost);
   std::fs::write("target/hikari-clockbank-search-replay.json",serde_json::to_vec(&p).unwrap()).unwrap();
  }).unwrap().join().unwrap();return;
 }

 let mut w:WorldState=serde_json::from_str(include_str!("../data/source/clockbank-search-party.json")).unwrap();
 for flag in 6050..=6077 {w.set_flag(flag,false);}
 w.set_flag(6076,true);w.time=2;w.existing_npcs.clear();w.map="Twn_Cty_2_1_A".into();
 let mut l=Loader::new(world::catalog(),w,CrtRng::new(12345)).unwrap();
 l.visit_map("Twn_Cty_2_1_B").unwrap();
 for event in ["entry","MS_SHO_20_0100","MS_SHO_20_0200","MS_SHO_20_0400","MS_SHO_20_0500","MS_SHO_20_0610"] {
  let before=l.rng.draws;
  if event=="MS_SHO_20_0500" {
   for inside in [false,true] {
    let mut p=Loader::new(world::catalog(),l.state.clone(),l.rng).unwrap();
    p.manual_time_switch(4,"start night outside").unwrap();
    let start=p.rng.draws;
    if !inside {p.manual_time_switch(2,"day outside").unwrap();println!("outside day switch {}",p.rng.draws-start);}
    let n=p.rng.draws;p.visit_map("Dng_Cty_2_1").unwrap();println!("inside_switch={inside} entry draws={}",p.rng.draws-n);
    if inside {let n=p.rng.draws;p.manual_time_switch(2,"day inside").unwrap();println!("inside day switch {}",p.rng.draws-n);}
    p.skipped_event(event).unwrap();println!("inside_switch={inside} total={} end={:?}",p.rng.draws-start,p.rng);
   }
  }
  if event!="entry" {
   let mut a=Loader::new(world::catalog(),l.state.clone(),l.rng).unwrap();
   a.manual_time_switch(4,"night before scene").unwrap();
   let start=a.rng.draws;
   a.skipped_event(event).unwrap();
   println!("{event} NIGHT scene draws={} end_time={} end_map={}",a.rng.draws-start,a.state.time,a.state.map);
   if a.state.time==4 {a.manual_time_switch(2,"day after scene").unwrap();}
   let mut b=Loader::new(world::catalog(),l.state.clone(),l.rng).unwrap();
   b.skipped_event(event).unwrap();b.manual_time_switch(4,"night after scene").unwrap();b.manual_time_switch(2,"day after scene").unwrap();
   println!("  same final RNG={} draws night={} day={}",a.rng==b.rng,a.rng.draws,b.rng.draws);
  }
  if event!="entry" {l.skipped_event(event).unwrap();}
  let event_draws=l.rng.draws-before;
  let mut p=Loader::new(world::catalog(),l.state.clone(),l.rng).unwrap();
  let n=p.rng.draws;p.manual_time_switch(4,"audit night").unwrap();let night=p.rng.draws-n;
  let n=p.rng.draws;p.manual_time_switch(2,"audit day").unwrap();let day=p.rng.draws-n;
  println!("{event}: event={event_draws} night={night} day={day}");
 }
}
