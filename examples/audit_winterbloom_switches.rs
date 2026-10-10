use ot2_speedrun_assistant::{CrtRng, world::{self, WorldState, Loader}};
use serde::Deserialize;
#[derive(Deserialize)] struct Snapshot { after: CrtRng, world_after: WorldState }
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let r: serde_json::Value = serde_json::from_str(&std::fs::read_to_string("outputs/profile-19542.json")?)?;
    let s: Snapshot = serde_json::from_value(r.pointer("/prediction/post_battle_field/flats_escape/snows_escape/before_osvald/winterbloom_after_eastern/with_second/town").unwrap().clone())?;
    let mut l = Loader::new(world::catalog(), s.world_after, s.after)?;
    for time in [4, 2, 4, 2] {
        let before = l.rng.draws;
        l.manual_time_switch(time, "Winterbloom switch audit")?;
        println!("to {time}: {} CRT draws", l.rng.draws-before);
    }
    Ok(())
}
