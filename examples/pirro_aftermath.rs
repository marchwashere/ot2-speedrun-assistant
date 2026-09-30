//! Predict the chapter ending from a route JSON export (after Pirro's drops).
use ot2_speedrun_assistant::{CrtRng, pirro_aftermath, world::WorldState};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::env::args()
        .nth(1)
        .ok_or("Supply a route JSON export")?;
    let bytes = std::fs::read(file)?;
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes);
    let route: serde_json::Value = serde_json::from_slice(bytes)?;
    let pirro = &route["post_battle_field"]["pirro_battle"];
    let mut arrival: CrtRng = serde_json::from_value(pirro["after_victory"].clone())?;
    arrival.draws += route["crt_setup"]["battle_entry_advances"]
        .as_str()
        .ok_or("Missing seed position")?
        .parse::<u64>()?;
    let state: WorldState = serde_json::from_value(pirro["world_at_entry"].clone())?;
    let prediction = pirro_aftermath::with_world_state(arrival, state)?;
    println!("{}", serde_json::to_string_pretty(&prediction)?);
    Ok(())
}
