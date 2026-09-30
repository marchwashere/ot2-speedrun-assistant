//! Tag Abandoned Village, return to Flats, then walk toward Southern Cape Cold Snows.
use crate::{
    CrtRng, Result, encounter, ensure,
    route::MovementBudget,
    world::{self, Decision, Loader, Trace, WorldState},
};
use serde::Serialize;

pub const VILLAGE: &str = "Twn_Cty_1_2_A";
pub const MEASURED_WALK_UPDATES: u32 = 70;
pub const MEASURED_RUN_UPDATES: u32 = 66;
pub const SAFETY_UPDATES: u32 = 2;
pub const WALK_UPDATES: u32 = MEASURED_WALK_UPDATES + SAFETY_UPDATES;
pub const RUN_UPDATES: u32 = MEASURED_RUN_UPDATES + SAFETY_UPDATES;

#[derive(Debug, Serialize)]
pub struct MixedBudget {
    pub running_distance_percent: f64,
    pub walking_distance_percent: f64,
    pub estimated_cost: u32,
    pub estimated_counter_at_exit: u32,
}
#[derive(Debug, Serialize)]
pub struct SnowReach {
    pub initial_counter: u32,
    pub walking: MovementBudget,
    pub running: MovementBudget,
    pub mixed: Option<MixedBudget>,
    pub verification: &'static str,
}
pub fn snow_reach(counter: u32) -> Result<SnowReach> {
    let mut options = encounter::PredictionOptions {
        map: "Fld_Cty_1_2".into(),
        zone: "ENT_FLD_CTY_1_2_LND".into(),
        period: "DayTime".into(),
        dash: false,
        ..Default::default()
    };
    let walking = MovementBudget::new(
        counter,
        WALK_UPDATES,
        encounter::catalog().counter_decrement(&options)?,
    );
    options.dash = true;
    let running = MovementBudget::new(
        counter,
        RUN_UPDATES,
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
    Ok(SnowReach {
        initial_counter: counter,
        walking,
        running,
        mixed,
        verification: "Measured 70 walking / 66 running updates; add two safety updates to each, giving 72*9=648 walking and 68*12=816 running. Estimated route-distance cost is 648*(1-f)+816*f. Reserve one counter point, round running percentage down to 0.1% and cost up. Mixed movement interpolates measured speed-dependent overshoot; it is not a verified switch-position simulation. Ends before loading Southern Cape Cold Snows.",
    })
}

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub arrival: CrtRng,
    pub at_village: CrtRng,
    pub after: CrtRng,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub world_at_village: WorldState,
    pub world_after: WorldState,
    pub snow_reach: SnowReach,
    pub at_snow_entrance: Option<CrtRng>,
    pub verification: &'static str,
}

pub fn with_world_state(arrival: CrtRng, state: WorldState) -> Result<Prediction> {
    ensure(
        state.map == "Fld_Cty_1_2"
            && state.time == 2
            && !state.first_encounter
            && state.party == [7]
            && state.active_scenarios.is_empty()
            && state.prologues.is_empty(),
        "Village visit requires daytime solo Throné after the Flats Escape and completed chapter",
    )?;
    let first_visit = !state.visited_maps.contains(VILLAGE);
    let mut loader = Loader::new(world::catalog(), state, arrival)?;
    loader.visit_map(VILLAGE)?;
    // LevelKind=town, first visit, no active chapter/prologue. The fixed fast-
    // travel tutorial uses the audited generic popup path and consumes no CRT.
    if first_visit {
        loader.dismiss_tutorial(62)?;
    }
    loader.boundary("Abandoned Village: tagged, field control restored");
    let at_village = loader.rng;
    let world_at_village = loader.state.clone();
    loader.visit_map("Fld_Cty_1_2")?;
    loader.boundary("New Delsta Flats: returned from Abandoned Village");
    let reach = snow_reach(loader.state.encounter_counter)?;
    let at_snow_entrance = reach.walking.reaches_reset.then_some(loader.rng);
    Ok(Prediction {
        arrival,
        at_village,
        after: loader.rng,
        trace: loader.trace,
        decisions: loader.decisions,
        world_at_village,
        world_after: loader.state,
        snow_reach: reach,
        at_snow_entrance,
        verification: "Source-derived, awaiting live validation. Enter Abandoned Village on foot, tag it, dismiss any fast-travel popup, immediately return to daytime Flats. Both loads evaluate carried flags, NPC gates, level residency and counter resets. Tavern event is None. Village blueprint audit finds no new BeginPlay CRT calls; seeded-stream and cooked construction-only effects are excluded. The return rolls a fresh first-encounter counter, not the normal counter from Escape. No fixed RNG offsets or extra menu/time actions.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn movement_uses_independent_padded_speeds_and_strict_thresholds() {
        for (counter, safe, run, percent) in [
            (648, false, false, None),
            (649, true, false, Some(0.0)),
            (700, true, false, Some(30.3)),
            (816, true, false, Some(99.4)),
            (817, true, true, Some(100.0)),
        ] {
            let r = snow_reach(counter).unwrap();
            assert_eq!((r.walking.cost, r.running.cost), (648, 816));
            assert_eq!(
                (r.walking.reaches_reset, r.running.reaches_reset),
                (safe, run)
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
