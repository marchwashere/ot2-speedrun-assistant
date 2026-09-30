//! Second victory -> skipped event scripts -> New Delsta radar tutorial.
//! The world evaluator derives each CRT call from state and cooked rules.
use crate::{
    CrtRng, Result,
    world::{self, Decision, Loader, Trace, WorldState},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub arrival: CrtRng,
    pub after_popup: CrtRng,
    pub map: String,
    pub tutorial: &'static str,
    pub expected_draws: u64,
    pub verification: &'static str,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub world_after: WorldState,
}

/// Arrival is after second-fight drops, before battle-to-field callbacks.
/// Earlier route draws are already included; world bootstrap consumes none.
pub fn after_second_victory(arrival: CrtRng) -> Result<Prediction> {
    with_world_state(arrival, world::throne_after_second_state()?)
}

pub fn with_world_state(arrival: CrtRng, state: WorldState) -> Result<Prediction> {
    let mut loader = Loader::new(world::catalog(), state, arrival)?;
    loader.restore_field_after_battle("Return from second battle")?;
    loader.skipped_event("MS_TOU_10_0700")?;
    Ok(Prediction {
        arrival,
        after_popup: loader.rng,
        map: loader.state.map.clone(),
        tutorial: "eRADAR_MAP",
        expected_draws: loader.rng.draws - arrival.draws,
        verification: "City CRT calls are evaluated from event commands, progression flags, time, placement candidates and streamed-level residency. The fresh Throné route matches every city call in the 2026-09-27 capture (5264 -> 5374); 110 is a regression result, not a runtime offset.",
        trace: loader.trace,
        decisions: loader.decisions,
        world_after: loader.state,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::DrawKind;

    #[test]
    fn city_evaluation_matches_every_live_call() {
        let arrival = CrtRng {
            state: 0x41C7_DB10,
            draws: 5264,
        };
        let p = after_second_victory(arrival).unwrap();
        assert_eq!(
            p.after_popup,
            CrtRng {
                state: 0x6B5B_BFD6,
                draws: 5374
            }
        );
        let capture: serde_json::Value = serde_json::from_str(include_str!(
            "../data/source/live-second-city-20260927.json"
        ))
        .unwrap();
        let rows: Vec<_> = capture["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["position"].as_u64().unwrap() >= 5264)
            .collect();
        let calls: Vec<_> = p.trace.iter().filter(|s| s.before != s.after).collect();
        assert_eq!(calls.len(), rows.len());
        for (span, row) in calls.iter().zip(rows) {
            assert_eq!(span.after.draws - span.before.draws, 1);
            assert_eq!(row["position"], span.before.draws);
            assert_eq!(row["before"], span.before.state);
            let rva = match span.kind {
                DrawKind::Counter => "0x30D7424",
                DrawKind::NpcFrame => "0x30D7604",
                DrawKind::StreamedLevel => "0x11C010F",
                other => panic!("Unexpected live draw {other:?}"),
            };
            assert_eq!(row["returnRva"], rva, "{}", span.reason);
            if span.kind == DrawKind::NpcFrame {
                let label = row["blueprint"]["spawn"]["values"]
                    .as_array()
                    .expect("captured spawn locals")
                    .iter()
                    .find(|v| v["name"] == "NPCLabel")
                    .expect("captured NPC identity");
                assert_eq!(
                    label["value"], span.subject,
                    "NPC order at {}",
                    span.before.draws
                );
            }
            let mut reference = span.before;
            reference.next_u15();
            assert_eq!(reference, span.after);
        }
        assert_eq!(
            calls
                .iter()
                .filter(|s| s.kind == DrawKind::NpcFrame)
                .count(),
            85
        );
        assert_eq!(
            calls
                .iter()
                .filter(|s| s.kind == DrawKind::StreamedLevel)
                .count(),
            11
        );
        assert_eq!(
            calls.iter().filter(|s| s.kind == DrawKind::Counter).count(),
            14
        );
        assert_eq!(p.expected_draws, 110);
        assert_eq!(p.world_after.party, vec![7]);
        assert!(p.world_after.flag(8007));
        assert!(
            p.trace
                .iter()
                .any(|t| t.subject == "Epilogue_PLAYER_IS_Throne")
        );
        assert!(
            p.decisions
                .iter()
                .any(|d| d.callback == "RefreshBarCharacter"
                    && d.result == "BarEventLabel=None; no lottery")
        );
        assert!(p.trace.windows(2).all(|p| p[0].after == p[1].before));
        let relative = after_second_victory(CrtRng {
            state: arrival.state,
            draws: 310,
        })
        .unwrap();
        assert_eq!(relative.after_popup.state, p.after_popup.state);
        assert_eq!(relative.after_popup.draws, 420);
    }

    #[test]
    fn script_branch_bootstrap_does_not_apply_both_sides() {
        let state = world::throne_after_second_state().unwrap();
        assert!(state.flag(2206));
        assert!(state.flag(415));
        assert!(!state.flag(413));
        assert!(!state.flag(25428));
        assert_eq!(state.party, vec![7, 15, 16]);
    }

    #[test]
    fn changing_placement_flags_and_level_residency_changes_evaluated_draws() {
        let rng = CrtRng::new(0);
        let baseline = after_second_victory(rng).unwrap();
        let mut state = world::throne_after_second_state().unwrap();
        state.set_flag(2206, false);
        let changed = with_world_state(rng, state).unwrap();
        assert_eq!(changed.expected_draws, baseline.expected_draws - 1);
        assert!(
            !changed
                .trace
                .iter()
                .any(|t| t.subject == "Epilogue_PLAYER_IS_Throne")
        );
        let mut state = world::throne_after_second_state().unwrap();
        state.resident_levels.remove("Btl_SinExpose");
        let recreated = with_world_state(rng, state).unwrap();
        assert_eq!(recreated.expected_draws, baseline.expected_draws + 1);
        assert_eq!(
            recreated
                .trace
                .iter()
                .filter(|t| t.subject == "Btl_SinExpose")
                .count(),
            1
        );
    }
}
