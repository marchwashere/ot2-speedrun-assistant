use ot2_speedrun_assistant::{CrtRng,world::WorldState,snows_escape::Entry,hikari_manipulation};
use serde::Deserialize;
#[derive(Deserialize)] struct Snapshot {after:CrtRng,world_after:WorldState}
fn main()->Result<(),Box<dyn std::error::Error>> {
 let response:serde_json::Value=serde_json::from_str(&std::fs::read_to_string("outputs/conning-nearest-live-4997.json")?)?;
 let v=response.pointer("/prediction/post_battle_field/flats_escape/snows_escape/before_osvald/western/winterbloom/voyage_with_second").unwrap();
 assert_eq!(v["cropdale"]["time_switches"].as_array().map_or(0,|s|s.len()),0);
 let s:Snapshot=serde_json::from_value(v["cropdale_exit"]["oresrush"]["ryu"]["town_entry"].clone())?;
 let town=Entry {arrival:s.after,after:s.after,world_after:s.world_after,trace:vec![],decisions:vec![]};
 let start=std::time::Instant::now();let p=hikari_manipulation::nearest_coasts(&town)?;
 let water=p.canalbrine.water.as_ref().unwrap();let coast=p.canalbrine.conning.as_ref().unwrap();
 println!("{}",serde_json::json!({"elapsed_ms":start.elapsed().as_secs_f64()*1000.0,
 "eastern_escape":v["cropdale_exit"]["escape"]["inputs"],"hikari":p.search.inputs,
 "water":water.escape.inputs,"northern":coast.northern.inputs,"western":coast.western.inputs,
 "northern_cait":coast.northern.victory.is_some(),"downstream_inputs":p.search.total_input_cost,
 "east_inputs":v["cropdale_exit"]["escape"]["weighted_input_cost"]}));
 Ok(())
}
