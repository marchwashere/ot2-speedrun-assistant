use ot2_speedrun_assistant::{CrtRng, world};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).expect("route export JSON");
    let text = std::fs::read_to_string(path)?;
    let raw: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{feff}'))?;
    if raw.get("world").is_some() {
        let prediction = ot2_speedrun_assistant::cape_cold_mug::predict(
            serde_json::from_value(raw["before"].clone())?,
            serde_json::from_value(raw["world"].clone())?,
        )?;
        println!("{}", serde_json::to_string_pretty(&prediction)?);
        return Ok(());
    }
    let p = raw.get("prediction").unwrap_or(&raw);
    let cape = &p["post_battle_field"]["cape_cold"];
    let state: world::WorldState = serde_json::from_value(cape["world_after"].clone())?;
    let mut rng: CrtRng = serde_json::from_value(cape["after_loading"].clone())?;
    rng.draws += p["crt_setup"]["battle_entry_advances"]
        .as_str()
        .unwrap()
        .parse::<u64>()?;
    let prediction = ot2_speedrun_assistant::cape_cold_mug::predict(rng, state)?;
    println!("{}", serde_json::to_string_pretty(&prediction)?);
    Ok(())
}
