//! Reproduce the main-area load after the two source-audited Steal actions.
//! The main-area draw count is evaluated by the persistent world loader.
use ot2_speedrun_assistant::{CrtRng, city, parlor, parlor_revisit, world};

fn main() {
    let city = city::after_second_victory(CrtRng {
        state: 0x41c7db10,
        draws: 5264,
    })
    .unwrap();
    let first = parlor::after_city(&city).unwrap();
    let revisit = parlor_revisit::after_first_visit(&first).unwrap();
    let mut state = revisit.world_after;
    assert_eq!(state.map, "Twn_Cty_1_1_B");
    assert_eq!(state.time, 3);
    assert!(state.flag(8012) && !state.flag(8013));
    assert_eq!(state.party, vec![7]);

    // Apply the actual 1400 flag commands and Save tutorial, using the audited
    // source fixture. Neither theft removes/replaces its field NPC or party.
    let clothes: serde_json::Value =
        serde_json::from_str(include_str!("../data/source/throne-clothes-steal.json")).unwrap();
    let town: serde_json::Value =
        serde_json::from_str(include_str!("../data/town-loading.json")).unwrap();
    assert_eq!(clothes["event"]["EncountReset"], false);
    for c in clothes["commands"].as_array().unwrap() {
        match c["cmd"].as_u64().unwrap() {
            70 | 71 => state.set_flag(c["target"].as_u64().unwrap() as u32, c["opt"][0] == "1"),
            7100 => {
                let tutorial = &town["tutorials"][c["target"].to_string()];
                state.set_flag(tutorial["list_flag"].as_u64().unwrap() as u32, true);
                state.set_flag(tutorial["opened_flag"].as_u64().unwrap() as u32, true);
            }
            40 | 8030 | 10000 => (), // Audited fixed dialogue/chapter setup/finish.
            other => panic!("Re-audit new 1400 command {other}"),
        }
    }
    let mut arrival = revisit.after_popup;
    arrival.advance(2); // Clothes: menu timer + success roll, story override.
    assert_eq!(
        arrival,
        CrtRng {
            state: 0xbeb9762f,
            draws: 5469
        }
    );
    arrival.advance(3); // Boy: menu timer + success roll + generic success line.
    assert_eq!(
        arrival,
        CrtRng {
            state: 0x956ef860,
            draws: 5472
        }
    );
    let mut loader = world::Loader::new(world::catalog(), state, arrival).unwrap();
    loader
        .refresh_npcs("1400 cleanup after clothes theft")
        .unwrap();
    assert_eq!(loader.rng, arrival);
    loader.load_map("Twn_Cty_1_1_A").unwrap();
    loader.boundary("New Delsta main area loaded; before further story triggers");
    let count = |kind| loader.trace.iter().filter(|t| t.kind == kind).count();
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "verification":"Source-derived; post-boy menu/dialogue at 5472, same Day/Dusk phase, no intervening actions or automatic time change. Stops after the map load, before 1500 or later story events.",
        "arrival":arrival, "after_load":loader.rng,
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
