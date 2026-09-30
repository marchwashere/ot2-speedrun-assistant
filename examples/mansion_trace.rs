//! Continue the saved post-Steal city world through the mansion arrival.
//! Input: JSON emitted by post_steal_main_trace (after the main-area load).
use ot2_speedrun_assistant::{CrtRng, world};

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("path to post_steal_main_trace JSON");
    let input: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let state: world::WorldState = serde_json::from_value(input["world_after"].clone()).unwrap();
    let mut arrival: CrtRng = serde_json::from_value(input["after_load"].clone()).unwrap();
    assert_eq!(state.map, "Twn_Cty_1_1_A");
    assert_eq!(state.time, 3);
    assert!(state.flag(8013) && !state.flag(8014));
    // Artist: one menu opening, three guaranteed steals, one closing dialogue.
    // The source audit is in docs/throne-artist-steal.md.
    arrival.advance(5);
    let mut loader = world::Loader::new(world::catalog(), state, arrival).unwrap();
    loader.skipped_event("MS_TOU_10_1500").unwrap();
    loader.boundary("Mansion: control returned after skipped 1500 -> 1600");
    let count = |kind| loader.trace.iter().filter(|t| t.kind == kind).count();
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "verification":"Source-derived prediction, both scenes skipped; begins after Artist menu closure. No intervening actions or automatic time change.",
        "arrival":arrival, "after_cutscenes":loader.rng,
        "draws":loader.rng.draws-arrival.draws,
        "counts":{
            "streamed_levels":count(world::DrawKind::StreamedLevel),
            "npc_frames":count(world::DrawKind::NpcFrame),
            "counters":count(world::DrawKind::Counter),
            "flag_lotteries":count(world::DrawKind::FlagLottery),
        },
        "trace":loader.trace, "decisions":loader.decisions, "world_after":loader.state,
    })).unwrap());
}
