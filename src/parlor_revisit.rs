//! Second parlor visit: 1200 -> 1300, through the Steal tutorial and event finish.
use crate::{
    CrtRng, Result, ensure,
    world::{self, Decision, Loader, Trace, WorldState},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub arrival: CrtRng,
    pub after_entry: CrtRng,
    pub after_return: CrtRng,
    pub after_popup: CrtRng,
    pub expected_draws: u64,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub world_after: WorldState,
    pub verification: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::DrawKind;

    fn previous() -> crate::parlor::Prediction {
        let city = crate::city::after_second_victory(CrtRng {
            state: 0x41c7db10,
            draws: 5264,
        })
        .unwrap();
        crate::parlor::after_city(&city).unwrap()
    }

    #[test]
    fn revisit_follows_progression_through_steal_popup_without_an_extra_reset() {
        let previous = previous();
        let p = after_first_visit(&previous).unwrap();
        assert_eq!(p.arrival, previous.after_return);
        assert_eq!(p.after_entry.draws, 5435);
        // Source-derived expectation, NOT a live capture fixture.
        assert_eq!(
            p.after_return,
            CrtRng {
                state: 0xb8e6f54d,
                draws: 5467
            }
        );
        assert_eq!(p.expected_draws, 43);
        let count = |kind| p.trace.iter().filter(|t| t.kind == kind).count();
        assert_eq!(count(DrawKind::StreamedLevel), 11);
        assert_eq!(count(DrawKind::NpcFrame), 22);
        assert_eq!(count(DrawKind::Counter), 10);
        assert_eq!(count(DrawKind::FlagLottery), 0);
        assert!(p.trace.iter().any(|t| t.subject == "NPC_TOU_10_0500_D000"));
        assert!(
            !previous
                .world_after
                .existing_npcs
                .contains("NPC_TOU_10_0500_D000")
        );
        assert!(p.world_after.existing_npcs.contains("NPC_TOU_10_0500_D000"));
        assert!(p.world_after.flag(8011));
        assert!(p.world_after.flag(8012));
        assert!(!p.world_after.flag(514));
        assert!(p.world_after.flag(1029)); // Tutorial available in the list.
        assert!(p.world_after.flag(1329)); // Tutorial opened.
        assert_eq!(p.after_return, p.after_popup);
        assert_eq!(p.world_after.map, "Twn_Cty_1_1_B");
        assert_eq!(p.world_after.time, 3);
        assert!(
            p.trace
                .last()
                .unwrap()
                .reason
                .contains("Steal tutorial dismissed; control returned")
        );
        let mut rng = p.arrival;
        for span in &p.trace {
            assert_eq!(span.before, rng);
            rng.advance(span.after.draws - span.before.draws);
            assert_eq!(span.after, rng);
        }
        assert_eq!(rng, p.after_popup);
    }

    #[test]
    fn revisit_evaluates_carried_flags_and_rejects_wrong_story_boundary() {
        let previous = previous();
        let baseline = after_first_visit(&previous).unwrap();
        let mut state = previous.world_after.clone();
        state.set_flag(2206, false);
        let changed = with_world_state(previous.after_return, state).unwrap();
        assert_eq!(changed.expected_draws, baseline.expected_draws - 1);
        assert!(
            !changed
                .trace
                .iter()
                .any(|t| t.subject == "Epilogue_PLAYER_IS_Throne")
        );
        assert!(with_world_state(baseline.after_return, baseline.world_after).is_err());
        let mut state = previous.world_after;
        state.time = 4;
        assert!(with_world_state(previous.after_return, state).is_err());
    }

    #[test]
    fn dusk_refresh_keeps_existing_npcs_and_black_market_selection() {
        let previous = previous();
        let mut loader = Loader::new(
            world::catalog(),
            previous.world_after,
            previous.after_return,
        )
        .unwrap();
        loader.load_map("Twn_Cty_1_1_C").unwrap();
        let rng = loader.rng;
        let flags = loader.state.flags.clone();
        let npcs = loader.state.existing_npcs.clone();
        loader
            .state
            .npc_start_delays
            .insert("future placement".into(), 2);
        loader.set_time(3, "MS_TOU_10_1200").unwrap();
        assert_eq!(loader.rng, rng);
        assert_eq!(loader.state.flags, flags);
        assert_eq!(loader.state.existing_npcs, npcs);
        assert_eq!(loader.state.npc_start_delays["future placement"], 1);
    }
}

pub fn after_first_visit(previous: &crate::parlor::Prediction) -> Result<Prediction> {
    with_world_state(previous.after_return, previous.world_after.clone())
}

pub fn with_world_state(arrival: CrtRng, state: WorldState) -> Result<Prediction> {
    ensure(
        state.map == "Twn_Cty_1_1_B" && state.flag(8010) && !state.flag(8011),
        "Parlor revisit requires the backstreets state after 1100 and before 1200",
    )?;
    ensure(
        state.time == 2,
        "Parlor revisit assumes the route's daytime state",
    )?;
    let mut loader = Loader::new(world::catalog(), state, arrival)?;
    let parlor = loader.map_name(loader.data.events["MS_TOU_10_1200"].map_id)?;
    loader.load_map(&parlor)?;
    let after_entry = loader.rng;
    loader.boundary("Second parlor visit: entered during daytime");
    loader.skipped_event_until_arrival("MS_TOU_10_1200", "MS_TOU_10_1300")?;
    let after_return = loader.rng;
    loader.skipped_event("MS_TOU_10_1300")?;
    // FinishEvent runs implicit FinishPre because 1300 has no command 9990.
    // Its flag writes refresh placements, but EncountReset is false.
    ensure(
        !loader.data.events["MS_TOU_10_1300"].reset_counter,
        "1300 cleanup needs re-audit if EncountReset changes",
    )?;
    loader.refresh_npcs("MS_TOU_10_1300 implicit FinishPre")?;
    loader.boundary("Steal tutorial dismissed; control returned");
    Ok(Prediction {
        arrival,
        after_entry,
        after_return,
        after_popup: loader.rng,
        expected_draws: loader.rng.draws - arrival.draws,
        trace: loader.trace,
        decisions: loader.decisions,
        world_after: loader.state,
        verification: "Source-derived prediction, not yet live-validated: enter the parlor from the daytime 1100 state, skip 1200 promptly, complete the fixed 1300 dialogue and dismiss Steal tutorial 31. Both map changes evaluate persisted world state. Day-to-Dusk does not invoke the Black Market lottery. The tutorial adds no CRT draw; 1300 cleanup refreshes placements with EncountReset=false. Final world state includes flag 8012 and tutorial flags.",
    })
}
