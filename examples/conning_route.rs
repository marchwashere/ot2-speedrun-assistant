use ot2_speedrun_assistant::{CrtRng,world::WorldState,snows_escape::Entry,winter_voyage};
use serde::Deserialize;
#[derive(Deserialize)] struct Snapshot {after:CrtRng,world_after:WorldState}
fn main()->Result<(),Box<dyn std::error::Error>> {
    let response:serde_json::Value=serde_json::from_str(&std::fs::read_to_string("outputs/canalbrine-water-live-4997.json")?)?;
    let s:Snapshot=serde_json::from_value(response.pointer("/prediction/post_battle_field/flats_escape/snows_escape/before_osvald/western/winterbloom/voyage_with_second/destination").unwrap().clone())?;
    let entry=Entry {arrival:s.after,after:s.after,world_after:s.world_after,trace:vec![],decisions:vec![]};
    let reach=winter_voyage::cropdale_reach(&entry.world_after)?.unwrap();
    let start=std::time::Instant::now();
    let (crop,exit)=winter_voyage::search_oresrush(&entry,&reach)?;
    let result=serde_json::json!({"cropdale":crop,"cropdale_exit":exit});
    std::fs::write("outputs/conning-route.json",serde_json::to_vec(&result)?)?;
    println!("Complete coast route in {:?}; {} Cropdale switches; Oresrush {}/{}",start.elapsed(),crop.time_switches.len(),exit.oresrush.as_ref().unwrap().switches_before_partitio.len(),exit.oresrush.as_ref().unwrap().switches_after_partitio.len());
    Ok(())
}
