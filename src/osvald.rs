//! Southern Cape Cold Snows entry, safe approach and immediate Osvald recruitment.
use crate::{
    CrtRng, Result, encounter, ensure,
    route::MovementBudget,
    village_return,
    world::{self, Decision, EventChoices, Loader, Trace, WorldState},
};
use serde::Serialize;

pub const MAP: &str = "Fld_Snw_1_2";
pub const WALK_UPDATES: u32 = 70;
pub const EVENT: &str = "MS_GAK_00_0100";

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub arrival: CrtRng,
    pub after_loading: CrtRng,
    pub world_at_entry: WorldState,
    pub walking: MovementBudget,
    pub at_osvald: Option<CrtRng>,
    pub after_recruitment: Option<CrtRng>,
    pub world_after: Option<WorldState>,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub verification: &'static str,
}

pub fn after_village(v: &village_return::Prediction) -> Result<Option<Prediction>> {
    let Some(arrival) = v.at_snow_entrance else {
        return Ok(None);
    };
    let mut state = v.world_after.clone();
    state.encounter_counter = v.snow_reach.walking.counter_before_reset.unwrap();
    with_world_state(arrival, state).map(Some)
}

pub fn with_world_state(arrival: CrtRng, state: WorldState) -> Result<Prediction> {
    ensure(
        state.map == "Fld_Cty_1_2"
            && state.time == 2
            && state.party == [7]
            && state.bench.is_empty()
            && state.active_scenarios.is_empty()
            && state.prologues.is_empty()
            && !state.flag(7020)
            && !state.flag(37517),
        "Osvald recruitment requires the daytime solo Throné route before meeting him",
    )?;
    let mut loader = Loader::new(world::catalog(), state, arrival)?;
    loader.visit_map(MAP)?;
    loader.boundary("Southern Cape Cold Snows: field control after loading");
    let after_loading = loader.rng;
    let world_at_entry = loader.state.clone();
    let options = encounter::PredictionOptions {
        map: MAP.into(),
        zone: "ENT_FLD_SNW_1_2_LND".into(),
        period: "DayTime".into(),
        dash: false,
        ..Default::default()
    };
    let walking = MovementBudget::new(
        loader.state.encounter_counter,
        WALK_UPDATES,
        encounter::catalog().counter_decrement(&options)?,
    );
    let mut after_recruitment = None;
    let mut world_after = None;
    let at_osvald = walking.reaches_reset.then_some(loader.rng);
    if walking.reaches_reset {
        loader.state.encounter_counter = walking.counter_before_reset.unwrap();
        loader.boundary("Osvald: reached after 70 walking counter updates");
        let choices = EventChoices::from([
            ((EVENT.into(), 15), "eDIALOG_YES".into()),
            ((EVENT.into(), 61), "eDIALOG_YES".into()),
            ((EVENT.into(), 70), "eDIALOG_NO".into()),
        ]);
        loader.skipped_event_with_choices(EVENT, &choices)?;
        ensure(
            loader.state.party == [7, 6] && loader.state.flag(7020) && loader.state.flag(1370),
            "Osvald recruitment did not reach its final tutorial",
        )?;
        loader.boundary("Osvald recruited: cutscene skipped and tutorials dismissed");
        after_recruitment = Some(loader.rng);
        world_after = Some(loader.state.clone());
    }
    Ok(Prediction {
        arrival,
        after_loading,
        world_at_entry,
        walking,
        at_osvald,
        after_recruitment,
        world_after,
        trace: loader.trace,
        decisions: loader.decisions,
        verification: "Source-derived; loading matches the reported six-cycle endpoint at 6135 and live level/NPC inventory, not a per-call capture. Recruitment awaits live validation. Enter daytime Southern Cape Cold Snows, walk 70 counter updates, help Osvald, accept his invitation, decline playing his introductory story, skip the remaining scene and dismiss tutorials 66/71. The shared loader evaluates actual levels, NPC gates and B/A counters. Walk requires a strictly positive counter (70*9+1=631). No running measurement is assumed. Fixed recruitment dialogs and tutorials do not draw CRT; event FinishPre refreshes placements then resets the counter. No map reload or fixed RNG offset during recruitment.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world() -> WorldState {
        let mut w = world::throne_after_second_state().unwrap();
        w.map = "Fld_Cty_1_2".into();
        w.time = 2;
        w.party = vec![7];
        w.active_scenarios.clear();
        w.prologues.clear();
        w.story_types.clear();
        w.resident_levels = world::catalog().maps[&w.map]
            .streams
            .iter()
            .cloned()
            .collect();
        w
    }
    #[test]
    fn recruitment_executes_flags_party_tutorials_and_one_counter_reset() {
        let mut safe = 0;
        let mut unsafe_walk = 0;
        for seed in 0..100 {
            let p = with_world_state(CrtRng::new(seed), world()).unwrap();
            assert_eq!(p.walking.cost, 630);
            assert_eq!(
                p.walking.reaches_reset,
                p.world_at_entry.encounter_counter >= 631
            );
            assert!(p.world_at_entry.first_encounter);
            let mut previous = p.arrival;
            for t in &p.trace {
                assert_eq!(t.before, previous);
                previous = t.after;
            }
            if let Some(after) = p.after_recruitment {
                safe += 1;
                assert_eq!(previous, after);
                assert_eq!(after.draws - p.after_loading.draws, 2);
                let w = p.world_after.unwrap();
                assert_eq!(w.map, MAP);
                assert_eq!(w.party, [7, 6]);
                for flag in [
                    37517, 37510, 7020, 7060, 7062, 35, 25563, 1064, 1364, 1070, 1370,
                ] {
                    assert!(w.flag(flag), "Missing recruitment flag {flag}");
                }
                assert!(!w.existing_npcs.contains("NPC_PROFESSOR_0000_0000"));
                assert!(!w.existing_npcs.contains("NPC_PROFESSOR_0000_0100"));
                assert!(w.first_encounter);
                assert_eq!(w.event_variables.get(&1), Some(&1));
            } else {
                unsafe_walk += 1;
                assert!(p.world_after.is_none());
                assert_eq!(previous, p.after_loading);
            }
        }
        assert!(safe > 0 && unsafe_walk > 0);
    }
}
