//! Continue an exported Flats prediction through harbor tagging and return.
use ot2_speedrun_assistant::{CrtRng, harbor, world::WorldState};
fn main() {
    let path = std::env::args().nth(1).expect("route export JSON");
    let input: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let flats = &input["post_battle_field"]["flats"];
    let mut rng: CrtRng = serde_json::from_value(flats["after"].clone()).unwrap();
    rng.draws += input["crt_setup"]["battle_entry_advances"]
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let state: WorldState = serde_json::from_value(flats["world_after"].clone()).unwrap();
    println!(
        "{}",
        serde_json::to_string_pretty(&harbor::with_world_state(rng, state).unwrap()).unwrap()
    );
}
