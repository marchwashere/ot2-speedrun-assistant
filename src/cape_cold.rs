//! Osvald recruited -> daytime walk/run to Cape Cold -> shared map loading.
use crate::{
    CrtRng, Result, encounter, ensure, osvald,
    route::MovementBudget,
    world::{self, Decision, Loader, Trace, WorldState},
};
use serde::Serialize;

pub const MAP: &str = "Twn_Snw_1_2_A";
pub const MEASURED_WALK_UPDATES: u32 = 52;
pub const MEASURED_RUN_UPDATES: u32 = 49;
pub const SAFETY_UPDATES: u32 = 2;

#[derive(Debug, Serialize)]
pub struct MixedBudget {
    pub running_distance_percent: f64,
    pub walking_distance_percent: f64,
    pub estimated_cost: u32,
    pub estimated_counter_at_exit: u32,
}
#[derive(Debug, Serialize)]
pub struct Reach {
    pub initial_counter: u32,
    pub walking: MovementBudget,
    pub running: MovementBudget,
    pub mixed: Option<MixedBudget>,
    pub verification: &'static str,
}
pub fn reach(counter: u32) -> Result<Reach> {
    let mut options = encounter::PredictionOptions {
        map: osvald::MAP.into(),
        zone: "ENT_FLD_SNW_1_2_LND".into(),
        period: "DayTime".into(),
        dash: false,
        ..Default::default()
    };
    let walking = MovementBudget::new(
        counter,
        MEASURED_WALK_UPDATES + SAFETY_UPDATES,
        encounter::catalog().counter_decrement(&options)?,
    );
    options.dash = true;
    let running = MovementBudget::new(
        counter,
        MEASURED_RUN_UPDATES + SAFETY_UPDATES,
        encounter::catalog().counter_decrement(&options)?,
    );
    let mixed = walking.reaches_reset.then(|| {
        let delta = running.cost - walking.cost;
        let tenths =
            (u64::from(counter - 1 - walking.cost) * 1000 / u64::from(delta)).min(1000) as u32;
        let cost = walking.cost + (delta * tenths).div_ceil(1000);
        MixedBudget {
            running_distance_percent: f64::from(tenths) / 10.0,
            walking_distance_percent: f64::from(1000 - tenths) / 10.0,
            estimated_cost: cost,
            estimated_counter_at_exit: counter - cost,
        }
    });
    Ok(Reach {
        initial_counter: counter,
        walking,
        running,
        mixed,
        verification: "Measured 52 walking / 49 running counter updates from Osvald to Cape Cold, plus two safety updates for each: 54*9=486 walking and 51*12=612 running. Estimate route-distance cost as 486*(1-f)+612*f, reserve one counter point, round running percentage down to 0.1% and cost up. Separate measured counts account for whole-route overshoot; mixed movement remains an interpolation, not a verified speed-switch simulation. Walking is safe even at the minimum first counter of 500. Movement consumes no CRT.",
    })
}

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub arrival: CrtRng,
    pub reach: Reach,
    pub at_entrance: Option<CrtRng>,
    pub after_loading: Option<CrtRng>,
    pub world_after: Option<WorldState>,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub verification: &'static str,
}

pub fn after_osvald(p: &osvald::Prediction) -> Result<Option<Prediction>> {
    let (Some(arrival), Some(state)) = (p.after_recruitment, &p.world_after) else {
        return Ok(None);
    };
    with_world_state(arrival, state.clone()).map(Some)
}

pub fn with_world_state(arrival: CrtRng, mut state: WorldState) -> Result<Prediction> {
    ensure(
        state.map == osvald::MAP
            && state.time == 2
            && state.party == [7, 6]
            && state.flag(7020)
            && state.flag(1370)
            && state.active_scenarios.is_empty()
            && state.prologues.is_empty(),
        "Cape Cold approach requires daytime Throné and Osvald after skipping his introductory story",
    )?;
    let reach = reach(state.encounter_counter)?;
    let at_entrance = reach.walking.reaches_reset.then_some(arrival);
    let mut after_loading = None;
    let mut world_after = None;
    let mut trace = Vec::new();
    let mut decisions = Vec::new();
    if let Some(remaining) = reach.walking.counter_before_reset {
        // Carry the full walking reference path. Safe mixed movement reaches
        // the same RNG; the outgoing load resets the counter before any use.
        state.encounter_counter = remaining;
        let mut loader = Loader::new(world::catalog(), state, arrival)?;
        loader.boundary("Cape Cold entrance: before loading");
        loader.visit_map(MAP)?;
        loader.boundary("Cape Cold: field control after loading");
        after_loading = Some(loader.rng);
        world_after = Some(loader.state);
        trace = loader.trace;
        decisions = loader.decisions;
    }
    Ok(Prediction {
        arrival,
        reach,
        at_entrance,
        after_loading,
        world_after,
        trace,
        decisions,
        verification: "Source-derived, awaiting live validation. Continue after Osvald recruitment/tutorials, no intervening CRT actions or time change. Load Cape Cold (Twn_Snw_1_2_A, map169) with the shared loader, carrying recruitment flags and party [7,6]. Evaluate case-insensitive streamed packages, eligible NPC frames and outgoing/incoming B/A resets. BarEventLabel=None; actor/parent audit found no additional BeginPlay CRT calls. No fixed map-load RNG offset. This extends the selected route without adding a boost-search target.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn padded_measured_speeds_reserve_one_point_and_round_down() {
        for (counter, walking_safe, running_safe, percent) in [
            (486, false, false, None),
            (487, true, false, Some(0.0)),
            (500, true, false, Some(10.3)),
            (552, true, false, Some(51.5)),
            (612, true, false, Some(99.2)),
            (613, true, true, Some(100.0)),
        ] {
            let r = reach(counter).unwrap();
            assert_eq!((r.walking.updates, r.running.updates), (54, 51));
            assert_eq!((r.walking.cost, r.running.cost), (486, 612));
            assert_eq!(
                (r.walking.reaches_reset, r.running.reaches_reset),
                (walking_safe, running_safe)
            );
            assert_eq!(
                r.mixed.as_ref().map(|m| m.running_distance_percent),
                percent
            );
            if let Some(m) = r.mixed {
                assert!(m.estimated_counter_at_exit > 0);
            }
        }
    }
}
