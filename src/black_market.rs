//! Mug R2 manipulation for the clerics at the final Flats night switch.
use crate::{CrtRng, Result, cape_cold_mug, encounter, ensure, manipulation::{self, BoostInputs},
    route::{MovementBudget, MixedMovementBudget},
    village_return::VILLAGE, world::{self, Decision, Loader, Trace, WorldState}};
use serde::Serialize;

const CLERICS: [&str; 2] = ["NPC_Fld_Cty_1_2_TALK_0900_N000", "NPC_Fld_Cty_1_2_TALK_1000_N000"];
pub fn clerics_present(state: &WorldState) -> bool {
    state.flag(25503) && CLERICS.iter().all(|label| state.existing_npcs.contains(*label))
}
pub const CLERIC_APPROACH_MEASURED_UPDATES: u32 = 40;
pub const CLERIC_APPROACH_SAFETY_UPDATES: u32 = 2;
pub const CLERIC_APPROACH_UPDATES: u32 = CLERIC_APPROACH_MEASURED_UPDATES + CLERIC_APPROACH_SAFETY_UPDATES;

#[derive(Debug, Serialize)]
pub struct ClericReach {
    pub initial_counter: u32,
    pub walking: MovementBudget,
    pub running: MovementBudget,
    pub mixed: Option<MixedMovementBudget>,
}

pub fn cleric_reach(counter: u32) -> Result<ClericReach> {
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
    let walking = MovementBudget::new(counter, CLERIC_APPROACH_UPDATES, walk);
    let running = MovementBudget::new(counter, CLERIC_APPROACH_UPDATES, run);
    let mixed = walking.reaches_reset.then(|| {
        // Both speeds use 42 updates, including two for player error. Replace walking updates with
        // running ones, retaining a positive counter at the night-switch spot.
        let max_running_updates = ((counter - 1 - walking.cost) / (run - walk))
            .min(CLERIC_APPROACH_UPDATES);
        let cost = walking.cost + max_running_updates * (run - walk);
        MixedMovementBudget {
            max_running_updates,
            walking_updates: CLERIC_APPROACH_UPDATES - max_running_updates,
            max_running_percent: f64::from(max_running_updates * 1000 / CLERIC_APPROACH_UPDATES) / 10.0,
            cost,
            counter_before_reset: counter - cost,
        }
    });
    Ok(ClericReach { initial_counter: counter, walking, running, mixed })
}

#[derive(Debug, Serialize)]
pub struct TimeSwitch {
    pub time: u8,
    pub after: CrtRng,
}
#[derive(Debug, Serialize)]
pub struct Route {
    pub cape_switches: Vec<TimeSwitch>,
    pub switched_at_village: bool,
    pub after_travel: CrtRng,
    pub after_day: CrtRng,
    pub after_flats: CrtRng,
    pub at_clerics: CrtRng,
    pub cleric_reach: ClericReach,
    pub after_night: CrtRng,
    pub clerics_present: bool,
    pub world_after: WorldState,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub verification: &'static str,
}
pub fn after_mug(p: &cape_cold_mug::Prediction) -> Result<Route> {
    after_mug_with_switches(p,0)
}
pub fn after_mug_with_switches(p: &cape_cold_mug::Prediction, switches:u32) -> Result<Route> {
    ensure(p.world_after.visited_maps.contains(VILLAGE), "Fast travel requires Abandoned Village to be tagged")?;
    ensure(p.mug.outcome == ot2_battle_engine::Outcome::Victory, "Clerics route requires Mug victory")?;
    let mut l = Loader::new(world::catalog(), p.world_after.clone(), p.after)?;
    let mut cape_switches=Vec::new();
    for _ in 0..switches {
        let time=if l.state.time==4 {2} else {4};
        l.set_time(time,"Cape Cold: post-Mug time switch")?;

        l.refresh_npcs("Cape Cold: time-switch placements")?;
        cape_switches.push(TimeSwitch {time,after:l.rng});
    }
    l.visit_map(VILLAGE)?;
    let after_travel = l.rng;
    let switched_at_village=l.state.time!=2;
    if switched_at_village {
        l.set_time(2, "Abandoned Village: switch to day after fast travel")?;

        l.refresh_npcs("Abandoned Village: daytime placements")?;
    }
    let after_day = l.rng;
    l.visit_map("Fld_Cty_1_2")?;
    let after_flats = l.rng;
    let cleric_reach = cleric_reach(l.state.encounter_counter)?;
    l.state.encounter_counter = cleric_reach.walking.counter_before_reset
        .ok_or_else(|| crate::SimError("Clerics approach encounters an enemy even while walking".into()))?;
    // Carry the full-walking reference counter; all safe mixed paths share CRT.
    l.boundary("Flats entry -> cleric spawns: 42 daytime movement updates (40 measured + 2 safety); walking reference");
    let at_clerics = l.rng;
    l.set_time(4, "Flats: switch to night near black market")?;

    l.refresh_npcs("Flats: nighttime black-market placements")?;
    let clerics_present = clerics_present(&l.state);
    Ok(Route { cape_switches, switched_at_village, after_travel, after_day, after_flats, at_clerics, cleric_reach, after_night:l.rng, clerics_present,
        world_after:l.state, trace:l.trace, decisions:l.decisions,
        verification:"Source-derived; not yet live-traced. Post-Mug Cape Cold time switches, tagged Abandoned Village fast travel, day switch only if still night, Flats load, 42 daytime walking/running updates to cleric spawns (40 measured + 2 for player error), then night switch; shared loader evaluates each callback. Costs are derived from zone data. Mixed movement substitutes running updates for walking updates and keeps one counter point. World counter carries the full-walking reference; safe movement consumes no CRT. Clerics require flag25503 and both nighttime NPC placements." })
}

#[derive(Debug, Serialize)]
pub struct Search {
    pub max_boost_repeats: Option<u32>,
    pub met: bool,
    pub extra_advances: u32,
    pub inputs: BoostInputs,
    pub candidates: u32,
    pub cape_time_switches:u32,
    pub total_time_switches:u32,
    pub weighted_input_cost:u32,
    pub input_cost_per_switch:u32,
}
pub struct Prediction {
    pub mug: cape_cold_mug::Prediction,
    pub route: Route,
    pub search: Search,
}

pub fn predict(before: CrtRng, world: WorldState, cap: Option<u32>) -> Result<Prediction> {
    predict_accepting(before,world,cap,None,|_|Ok(true))?
        .ok_or_else(|| crate::SimError("Clerics search exhausted".into()))
}
/// Jointly require clerics and the caller's downstream target. A finite cost
/// ceiling is an optional improvement; None searches until a valid plan exists.
pub fn predict_accepting(before:CrtRng, world:WorldState, cap:Option<u32>, max_cost:Option<u32>, accepts:impl Fn(&Route)->Result<bool>) -> Result<Option<Prediction>> {
    const SWITCH_COST:u32=crate::early_cait::TIME_SWITCH_INPUT_COST;
    // Mug normally ends at night. Include the village day switch (when needed)
    // AND the final Flats night switch in the objective, not just extra actions.
    // Cost layers guarantee minimum weighted total. Ties prefer fewer Cape Cold
    // switches, then fewer CRT advances; every selected path is replayed.
    let mut candidates=0;
    let mut cost=2+2*SWITCH_COST;
    loop {
        if max_cost.is_some_and(|limit|cost>limit) { return Ok(None); }
        for switches in 0..=(cost-2)/SWITCH_COST {
            let time_switches=switches+u32::from(switches%2==0)+1;
            let fixed=time_switches*SWITCH_COST;
            if fixed+2>cost {continue;}
            let button_budget=(cost-fixed) as usize;
            for extra in 0..=(button_budget-2) as u32 {
                if cap.is_some_and(|c|extra>c.saturating_mul(3)) {break;}
                let inputs=match cap {
                    Some(c)=>manipulation::optimal_inputs_with_cap(extra,2,2,c)?,
                    None=>Some(manipulation::optimal_inputs(extra,2,2)?),
                };
                let Some(inputs)=inputs else {continue;};
                if inputs.input_count!=button_budget {continue;}
                let mug=cape_cold_mug::predict_with_boost_inputs(before,world.clone(),Some(&inputs.levels))?;
                let route=after_mug_with_switches(&mug,switches)?;
                candidates+=1;
                if route.clerics_present && accepts(&route)? {
                    return Ok(Some(Prediction {mug,route,search:Search {max_boost_repeats:cap,met:true,
                        extra_advances:extra,inputs,candidates,cape_time_switches:switches,
                        total_time_switches:time_switches,weighted_input_cost:cost,input_cost_per_switch:SWITCH_COST}}));
                }
            }
        }
        cost=cost.checked_add(1).ok_or_else(||crate::SimError("Clerics weighted search exhausted representable cost".into()))?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (CrtRng, WorldState) {
        let f:serde_json::Value = serde_json::from_str(include_str!("../data/source/capecold-mug-entry-6154.json")).unwrap();
        (serde_json::from_value(f["before"].clone()).unwrap(),serde_json::from_value(f["world"].clone()).unwrap())
    }
    #[test]
    fn cleric_approach_reserves_a_positive_counter() {
        for (counter, safe_walk, run_updates) in [(378, false, 0), (379, true, 0), (438, true, 19), (504, true, 41), (505, true, 42)] {
            let reach = cleric_reach(counter).unwrap();
            assert_eq!((reach.walking.cost, reach.running.cost), (378, 504));
            assert_eq!(reach.walking.reaches_reset, safe_walk);
            if let Some(mixed) = reach.mixed {
                assert_eq!(mixed.max_running_updates, run_updates);
                assert!(mixed.counter_before_reset > 0);
                if run_updates < CLERIC_APPROACH_UPDATES {
                    assert!(mixed.cost + 3 >= counter);
                }
            } else { assert!(!safe_walk); }
        }
    }
    #[test]
    fn clerics_search_replays_mug_and_final_night_lottery() {
        let (rng,w) = fixture();
        let p = predict(rng,w.clone(),None).unwrap();
        assert!(p.search.met);
        assert_eq!(p.route.world_after.time,4);
        assert_eq!(p.route.world_after.map,"Fld_Cty_1_2");
        assert!(p.route.world_after.flag(25503));
        assert_eq!(p.route.at_clerics, p.route.after_flats);
        assert_eq!(p.route.world_after.encounter_counter, p.route.cleric_reach.walking.counter_before_reset.unwrap());
        assert!(p.route.cleric_reach.running.reaches_reset);
        let replay = cape_cold_mug::predict_with_boost_inputs(rng,w.clone(),Some(&p.search.inputs.levels)).unwrap();
        assert_eq!(replay.after,p.mug.after);
        assert_eq!(after_mug_with_switches(&replay,p.search.cape_time_switches).unwrap().after_night,p.route.after_night);
        assert_eq!(p.mug.mug.menus[1].rng.draws,6218);
        let lotteries:Vec<_> = p.route.trace.iter().filter(|t|t.kind == world::DrawKind::FlagLottery).collect();
        assert_eq!(lotteries.len(),p.search.total_time_switches as usize);
        assert!(lotteries.last().unwrap().before.draws >= p.route.after_flats.draws);
        // Exhaust all switch/boost combinations with a strictly lower cost.
        for switches in 0..p.search.weighted_input_cost/crate::early_cait::TIME_SWITCH_INPUT_COST {
            for extra in 0..p.search.weighted_input_cost {
                let inputs=manipulation::optimal_inputs(extra,2,2).unwrap();
                let cost=inputs.input_count as u32+crate::early_cait::TIME_SWITCH_INPUT_COST*(switches+u32::from(switches%2==0)+1);
                if cost>=p.search.weighted_input_cost {continue;}
                let m=cape_cold_mug::predict_with_boost_inputs(rng,w.clone(),Some(&inputs.levels)).unwrap();
                assert!(!after_mug_with_switches(&m,switches).unwrap().clerics_present);
            }
        }
        for switches in 1..=2 {
            let r=after_mug_with_switches(&replay,switches).unwrap();
            assert_eq!(r.switched_at_village,switches%2==0);
            if switches%2==1 {assert_eq!(r.after_day,r.after_travel);}
            assert_eq!(r.cape_switches.len(),switches as usize);
        }
        let baseline = predict(rng,w,Some(0)).unwrap();
        assert_eq!(baseline.search.extra_advances,0);
        assert_eq!(baseline.mug.after.draws,6299);
        println!("selected: {:?}, Mug {}, night {}",p.search,p.mug.after.draws,p.route.after_night.draws);
    }
}
