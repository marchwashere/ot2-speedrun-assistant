//! Pirro victory -> skipped 1900/2000 -> Eastern New Delsta Highroad control.
use crate::{
    CrtRng, Result, ensure,
    world::{self, Decision, Loader, Trace, WorldState},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub arrival: CrtRng,
    pub after: CrtRng,
    pub expected_draws: u64,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub world_after: WorldState,
    pub verification: &'static str,
}

/// Continue only a completed victory, preserving the caller's RNG count origin.
pub fn after_battle(battle: &crate::pirro::Prediction) -> Result<Option<Prediction>> {
    if battle.state.outcome != ot2_battle_engine::Outcome::Victory || battle.routine_error.is_some()
    {
        return Ok(None);
    }
    let arrival = battle.after_victory.ok_or_else(|| {
        crate::SimError("Pirro victory is missing its after-drops boundary".into())
    })?;
    with_world_state(arrival, battle.world_at_entry.clone()).map(Some)
}

/// `arrival` is after victory drops; `state` is Pirro's carried world_at_entry.
pub fn with_world_state(arrival: CrtRng, state: WorldState) -> Result<Prediction> {
    ensure(
        state.map == "Dng_Cty_1_1"
            && state.time == 4
            && state.party == [7]
            && state.bench.is_empty()
            && state.flag(8017)
            && !state.flag(8018)
            && !state.flag(2900)
            && state.active_scenarios.contains(&700),
        "Pirro aftermath requires the fresh solo Throné victory boundary",
    )?;
    let mut loader = Loader::new(world::catalog(), state, arrival)?;
    // Same battle-return callback as the captured 0600 -> 0700 handoff.
    // Suspended 1800 is replaced by 1900; do not execute its cleanup twice.
    loader.restore_field_after_battle("Return from Pirro battle")?;
    loader.skipped_event("MS_TOU_10_1900")?;
    ensure(
        loader.state.map == "Fld_Cty_1_3" && loader.state.flag(8019),
        "Chapter ending did not reach Eastern New Delsta Highroad",
    )?;
    loader.boundary("Eastern New Delsta Highroad: control restored");
    Ok(Prediction {
        arrival,
        after: loader.rng,
        expected_draws: loader.rng.draws - arrival.draws,
        trace: loader.trace,
        decisions: loader.decisions,
        world_after: loader.state,
        verification: "The 2026-09-29 live replay matches all 62 ending calls from seed-0 advance 5900 to 5962, including two canoe BeginPlay draws. Other world states remain source-derived. Assumes promptly skipped scenes; no fixed advance offset.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{city, mansion, parlor, parlor_revisit, world::DrawKind};

    fn state() -> WorldState {
        let city = city::after_second_victory(CrtRng::new(0)).unwrap();
        let parlor = parlor::after_city(&city).unwrap();
        let revisit = parlor_revisit::after_first_visit(&parlor).unwrap();
        let mansion = mansion::after_parlor_revisit(&revisit).unwrap();
        let mut world = mansion.lookout_ambush.world_after;
        world.set_flag(8017, true);
        world.first_encounter = false;
        world
    }

    #[test]
    fn completion_and_dawn_drive_ending_placements() {
        let p = with_world_state(
            CrtRng {
                state: 0x533b5d0c,
                draws: 5900,
            },
            state(),
        )
        .unwrap();
        // All calls below are compared against the continuous retail capture.
        assert_eq!(
            p.after,
            CrtRng {
                state: 0x6e0026c2,
                draws: 5962
            }
        );
        assert_eq!(p.world_after.time, 1);
        assert!(!p.world_after.active_scenarios.contains(&700));
        assert!(
            !p.world_after
                .prologues
                .contains("EPlayableCharacterID::eTHIEF")
        );
        assert!(
            p.world_after
                .existing_npcs
                .contains("NPC_SS_TCity13_Tutorial_0100")
        );
        assert_eq!(
            p.trace
                .iter()
                .filter(|t| t.kind == DrawKind::StreamedLevel)
                .count(),
            12
        );
        assert_eq!(
            p.trace
                .iter()
                .filter(|t| t.kind == DrawKind::NpcFrame)
                .count(),
            34
        );
        assert_eq!(
            p.trace
                .iter()
                .filter(|t| t.kind == DrawKind::Counter)
                .count(),
            14
        );
        assert!(!p.trace.iter().any(|t| t.kind == DrawKind::FlagLottery));
        assert!(p.trace.windows(2).all(|w| w[0].after == w[1].before));
        assert_eq!(p.world_after.encounter_counter, 862);
        let live: serde_json::Value = serde_json::from_str(include_str!(
            "../data/source/live-pirro-aftermath-5900.json"
        ))
        .unwrap();
        let draws: Vec<_> = p.trace.iter().filter(|t| t.after != t.before).collect();
        let records = live["records"].as_array().unwrap();
        assert_eq!(draws.len(), records.len());
        for (draw, record) in draws.iter().zip(records) {
            assert_eq!(record["position"], draw.before.draws);
            assert_eq!(record["before"], draw.before.state);
            let caller = match draw.kind {
                DrawKind::Counter => "0x30D7424",
                DrawKind::StreamedLevel => "0x11C010F",
                DrawKind::NpcFrame => "0x30D7604",
                DrawKind::EnvironmentInitialization => {
                    assert!(
                        record["blueprint"]
                            .as_str()
                            .unwrap()
                            .ends_with("ExecuteUbergraph_BP_EnvObjMD_Canoe_A")
                    );
                    assert!(
                        record["object"]
                            .as_str()
                            .unwrap()
                            .ends_with(draw.subject.split('.').next_back().unwrap())
                    );
                    "0x30D7065"
                }
                _ => panic!("Unexpected ending call {:?}", draw.kind),
            };
            assert_eq!(record["returnRva"], caller, "{}", draw.reason);
        }
        // Actor initialization is tied to actual level creation, not every
        // route event or every repeated request for the current field.
        let mut revisit = Loader::new(world::catalog(), p.world_after.clone(), p.after).unwrap();
        revisit.load_map("Fld_Cty_1_3").unwrap();
        assert_eq!(revisit.rng, p.after);
        revisit.load_map("Twn_Cty_1_1_B").unwrap();
        revisit.load_map("Fld_Cty_1_3").unwrap();
        assert_eq!(
            revisit
                .trace
                .iter()
                .filter(|t| t.kind == DrawKind::EnvironmentInitialization)
                .count(),
            2
        );
        let mut changed = state();
        changed.resident_levels.remove("Btl_SinExpose");
        let recreated = with_world_state(p.arrival, changed).unwrap();
        assert_eq!(recreated.expected_draws, p.expected_draws + 1);
        let mut changed = state();
        changed.set_flag(2900, true);
        assert!(with_world_state(p.arrival, changed).is_err());
    }
}
