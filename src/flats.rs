//! Successful Highroad Escape -> daytime New Delsta Flats field control.
use crate::{
    CrtRng, Result, encounter, ensure, highroad,
    route::MovementBudget,
    world::{self, Decision, Loader, Trace, WorldState},
};
use serde::Serialize;

// User-measured complete Flats -> harbor paths. UpdatePlayerStep resets its
// position anchor after a decrement, discarding overshoot: these counts are
// speed-specific measurements, not interchangeable units of path distance.
pub const HARBOR_WALKING_UPDATES: u32 = 61;
pub const HARBOR_RUNNING_UPDATES: u32 = 56;

#[derive(Debug, Serialize)]
pub struct HarborReach {
    pub initial_counter: u32,
    pub walking: MovementBudget,
    pub running: MovementBudget,
    pub mixed: Option<MixedHarborBudget>,
    pub verification: &'static str,
}

#[derive(Debug, Serialize)]
pub struct MixedHarborBudget {
    /// Estimated fraction of route distance, not elapsed time or update count.
    pub running_distance_percent: f64,
    pub walking_distance_percent: f64,
    pub estimated_cost: u32,
    pub estimated_counter_at_harbor: u32,
}

fn mixed_budget(counter: u32, walk_cost: u32, run_cost: u32) -> Option<MixedHarborBudget> {
    if counter <= walk_cost {
        return None;
    }
    // Measured whole-route densities: 61*9 walking versus 56*12 running.
    // Cost(f) = walk_cost*(1-f) + run_cost*f. Reserve one counter point,
    // round distance share DOWN to 0.1%, and fractional counter cost UP.
    // This is interpolation, not a per-frame movement/switch-phase simulation.
    let tenths = (u64::from(counter - 1 - walk_cost) * 1000 / u64::from(run_cost - walk_cost))
        .min(1000) as u32;
    let cost = walk_cost + ((run_cost - walk_cost) * tenths).div_ceil(1000);
    Some(MixedHarborBudget {
        running_distance_percent: f64::from(tenths) / 10.0,
        walking_distance_percent: f64::from(1000 - tenths) / 10.0,
        estimated_cost: cost,
        estimated_counter_at_harbor: counter - cost,
    })
}

pub fn harbor_reach(counter: u32) -> Result<HarborReach> {
    let mut options = encounter::PredictionOptions {
        map: "Fld_Cty_1_2".into(),
        zone: "ENT_FLD_CTY_1_2_LND".into(),
        period: "DayTime".into(),
        dash: false,
        ..Default::default()
    };
    let walk = encounter::catalog().counter_decrement(&options)?;
    options.dash = true;
    let run = encounter::catalog().counter_decrement(&options)?;
    let walking = MovementBudget::new(counter, HARBOR_WALKING_UPDATES, walk);
    let running = MovementBudget::new(counter, HARBOR_RUNNING_UPDATES, run);
    let mixed = mixed_budget(counter, walking.cost, running.cost);
    Ok(HarborReach {
        initial_counter: counter,
        walking,
        running,
        mixed,
        verification: "User measured 61 walking updates at 9 points and 56 running updates at 12 points to harbor. For running route-distance fraction f, mixed cost is estimated as 549*(1-f)+672*f. Reserve one counter point, round running share down to 0.1% and fractional cost up. Full-route measurements include overshoot; mixed movement is interpolation, not a verified speed-switch simulation. Requires a positive counter at the exit. No CRT draws are consumed by this budget calculation.",
    })
}

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub arrival: CrtRng,
    pub after: CrtRng,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub world_after: WorldState,
    pub harbor_reach: HarborReach,
    pub verification: &'static str,
}

pub fn after_highroad(field: &highroad::FieldReturn) -> Result<Prediction> {
    with_world_state(field.after, field.world_after.clone())
}

/// Begins after the Escape field-return reset. Preserve the caller's count origin.
pub fn with_world_state(arrival: CrtRng, state: WorldState) -> Result<Prediction> {
    ensure(
        state.map == "Fld_Cty_1_3" && state.time == 2 && !state.first_encounter,
        "Flats approach requires the daytime Highroad field after Escape",
    )?;
    let mut loader = Loader::new(world::catalog(), state, arrival)?;
    loader.visit_map("Fld_Cty_1_2")?;
    loader.boundary("New Delsta Flats: field control restored");
    let harbor_reach = harbor_reach(loader.state.encounter_counter)?;
    Ok(Prediction {
        arrival,
        after: loader.rng,
        trace: loader.trace,
        decisions: loader.decisions,
        world_after: loader.state,
        harbor_reach,
        verification: "Source-derived map load, awaiting live verification. Carries the selected Highroad Escape and world progression; assumes daytime, no intervening CRT actions or second Highroad encounter. No fixed map-load offset.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_distance_estimate_uses_both_measured_densities_and_rounds_down() {
        assert!(harbor_reach(549).unwrap().mixed.is_none());
        for (counter, percent, cost, left) in [
            (550, 0.0, 549, 1),
            (600, 40.6, 599, 1),
            (672, 99.1, 671, 1),
            (673, 100.0, 672, 1),
            (717, 100.0, 672, 45),
        ] {
            let m = harbor_reach(counter).unwrap().mixed.unwrap();
            assert_eq!(m.running_distance_percent, percent);
            assert!((m.walking_distance_percent + percent - 100.0).abs() < 1e-10);
            assert_eq!(m.estimated_cost, cost);
            assert_eq!(m.estimated_counter_at_harbor, left);
        }
    }

    #[test]
    fn measured_speeds_have_independent_costs_and_strict_exit_boundaries() {
        let r = harbor_reach(717).unwrap();
        assert_eq!(
            (
                r.walking.updates,
                r.walking.cost,
                r.walking.counter_before_reset
            ),
            (61, 549, Some(168))
        );
        assert_eq!(
            (
                r.running.updates,
                r.running.cost,
                r.running.counter_before_reset
            ),
            (56, 672, Some(45))
        );
        for (counter, walk_safe, run_safe) in [
            (549, false, false),
            (550, true, false),
            (672, true, false),
            (673, true, true),
        ] {
            let r = harbor_reach(counter).unwrap();
            assert_eq!(
                (r.walking.reaches_reset, r.running.reaches_reset),
                (walk_safe, run_safe)
            );
        }
    }
}
