//! Radar popup -> parlor -> first scene -> trigger/Yes -> two more scenes
//! -> backstreets. Map loads and script callbacks use the persistent world loader.
use crate::{
    CrtRng, Result, ensure,
    world::{self, Decision, EventChoices, Loader, Trace, WorldState},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub arrival: CrtRng,
    pub after_entry: CrtRng,
    pub after_first_scene: CrtRng,
    pub after_return: CrtRng,
    pub map: String,
    pub expected_draws: u64,
    pub verification: &'static str,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub world_after: WorldState,
}

pub fn after_city(city: &crate::city::Prediction) -> Result<Prediction> {
    with_world_state(city.after_popup, city.world_after.clone())
}

pub fn with_world_state(arrival: CrtRng, state: WorldState) -> Result<Prediction> {
    ensure(
        state.map == "Twn_Cty_1_1_B",
        "Parlor route starts in New Delsta backstreets",
    )?;
    let mut loader = Loader::new(world::catalog(), state, arrival)?;
    let parlor = loader.map_name(loader.data.events["MS_TOU_10_0900"].map_id)?;
    loader.load_map(&parlor)?;
    let after_entry = loader.rng;
    loader.boundary("Entered parlor");
    loader.skipped_event("MS_TOU_10_0900")?;
    let after_first_scene = loader.rng;
    loader.boundary("First parlor cutscene finished; run to next trigger");
    // Walking inside the parlor and this ordinary Yes dialog have no CRT calls
    // in the captured route. 0910's own FinishPre has EncountReset=false.
    let choices = EventChoices::from([(("MS_TOU_10_0910".into(), 0), "eDIALOG_YES".into())]);
    loader.skipped_event_with_choices("MS_TOU_10_0910", &choices)?;
    loader.boundary("Two further cutscenes finished; control returned in backstreets");
    Ok(Prediction {
        arrival,
        after_entry,
        after_first_scene,
        after_return: loader.rng,
        map: loader.state.map.clone(),
        expected_draws: loader.rng.draws - arrival.draws,
        verification: "Enter parlor, skip the first cutscene, run to the next trigger, select Yes, skip two further cutscenes and return to New Delsta backstreets. Both map changes use the town-loading evaluator with carried progression, NPC and level state. Event scripts evaluate the Day change, Black Market lottery and counter callbacks. All 50 CRT calls and all 24 NPC identities match the 2026-09-27 capture (5374 -> 5424); the total is derived, not a fixed offset.",
        trace: loader.trace,
        decisions: loader.decisions,
        world_after: loader.state,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::DrawKind;

    fn city() -> crate::city::Prediction {
        crate::city::after_second_victory(CrtRng {
            state: 0x41c7db10,
            draws: 5264,
        })
        .unwrap()
    }

    #[test]
    fn evaluated_parlor_route_matches_every_live_call_and_npc() {
        let city = city();
        let p = after_city(&city).unwrap();
        assert_eq!(p.arrival, city.after_popup);
        assert_eq!(p.after_entry.draws, 5388);
        assert_eq!(p.after_first_scene.draws, 5390);
        assert_eq!(
            p.after_return,
            CrtRng {
                state: 0x080e80b0,
                draws: 5424
            }
        );
        let capture: serde_json::Value =
            serde_json::from_str(include_str!("../data/source/live-parlor-20260927.json")).unwrap();
        let rows = capture["records"].as_array().unwrap();
        let calls: Vec<_> = p.trace.iter().filter(|t| t.before != t.after).collect();
        assert_eq!(calls.len(), rows.len());
        for (span, row) in calls.iter().zip(rows) {
            assert_eq!(span.before.draws, row["position"].as_u64().unwrap());
            assert_eq!(span.before.state, row["before"].as_u64().unwrap() as u32);
            assert_eq!(span.after.draws, span.before.draws + 1);
            let rva = match span.kind {
                DrawKind::Counter => "0x30D7424",
                DrawKind::NpcFrame => "0x30D7604",
                DrawKind::StreamedLevel => "0x11C010F",
                DrawKind::FlagLottery => "0xEA7749",
                other => panic!("Unexpected draw {other:?}"),
            };
            assert_eq!(row["returnRva"], rva, "{}", span.reason);
            if span.kind == DrawKind::NpcFrame {
                let npc = row["blueprint"]["spawn"]["values"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|v| v["name"] == "NPCLabel")
                    .unwrap();
                assert_eq!(npc["value"], span.subject, "NPC at {}", span.before.draws);
            }
            let mut reference = span.before;
            reference.next_u15();
            assert_eq!(reference, span.after);
        }
        assert!(
            p.trace
                .windows(2)
                .all(|pair| pair[0].after == pair[1].before)
        );
        assert_eq!(p.world_after.map, "Twn_Cty_1_1_B");
        assert_eq!(p.world_after.time, 2);
        for flag in [8007, 8008, 8009, 8010, 2206] {
            assert!(p.world_after.flag(flag));
        }
        assert_eq!(p.world_after.party, city.world_after.party);
        assert!(
            p.trace
                .iter()
                .any(|t| t.kind == DrawKind::FlagLottery && t.subject == "GMK_BLACKMARKET_CONTROL")
        );
        assert!(!p.trace.iter().any(|t| t.reason.contains("0910 FinishPre")));
        assert_eq!(p.expected_draws, 50);
        assert_eq!(
            p.trace
                .iter()
                .filter(|t| t.kind == DrawKind::NpcFrame)
                .count(),
            24
        );
    }

    #[test]
    fn parlor_uses_carried_flags_and_residency_instead_of_a_fixed_offset() {
        let city = city();
        let baseline = after_city(&city).unwrap();
        let mut state = city.world_after.clone();
        state.set_flag(2206, false);
        let changed = with_world_state(city.after_popup, state).unwrap();
        assert_eq!(changed.expected_draws, baseline.expected_draws - 1);
        assert!(
            !changed
                .trace
                .iter()
                .any(|t| t.subject == "Epilogue_PLAYER_IS_Throne")
        );
        let mut state = city.world_after.clone();
        state.resident_levels.remove("Btl_SinExpose");
        let changed = with_world_state(city.after_popup, state).unwrap();
        assert_eq!(changed.expected_draws, baseline.expected_draws + 1);
        assert_eq!(
            changed
                .trace
                .iter()
                .filter(|t| t.subject == "Btl_SinExpose")
                .count(),
            1
        );
        let mut state = city.world_after;
        state.set_flag(20313, true);
        let changed = with_world_state(city.after_popup, state).unwrap();
        assert_eq!(changed.expected_draws, baseline.expected_draws + 1);
        assert!(changed.world_after.flag(20314));
    }

    #[test]
    fn dialog_requires_explicit_choice_and_relative_counts_remain_relative() {
        let mut city = city();
        city.after_popup.draws = 420;
        let p = after_city(&city).unwrap();
        assert_eq!(p.after_return.draws, 470);
        assert_eq!(p.after_return.state, 0x080e80b0);
        let mut loader = Loader::new(world::catalog(), p.world_after, p.after_return).unwrap();
        loader.load_map("Twn_Cty_1_1_C").unwrap();
        let rng = loader.rng;
        assert!(
            loader
                .skipped_event("MS_TOU_10_0910")
                .unwrap_err()
                .to_string()
                .contains("Missing dialog choice")
        );
        assert_eq!(loader.rng, rng);
    }
}
