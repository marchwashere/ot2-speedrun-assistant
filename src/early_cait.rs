//! First village visit -> early clerics -> daytime Osvald -> first Snows encounter.
//! No battle or boost RNG is duplicated here; callers replay their Escape plan.
use crate::{black_market, encounter::{self, Boundary, PredictionOptions}, ensure, osvald,
    route::MovementBudget, village_return, world::{self, Decision, Loader, Trace, WorldState}, CrtRng, Result};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Settings {
    pub clerics_max_boost_repeats: Option<u32>,
    pub cait_max_boost_repeats: Option<u32>,
}
impl Default for Settings {
    fn default() -> Self { Self { clerics_max_boost_repeats: None, cait_max_boost_repeats: None } }
}

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub village_visit: village_return::Visit,
    pub after_flats: CrtRng,
    pub world_at_flats: WorldState,
    pub movement: village_return::SnowReach,
    pub cleric_reach: black_market::ClericReach,
    pub after_night: Option<CrtRng>,
    pub clerics_present: bool,
    pub after_day: Option<CrtRng>,
    pub remaining_walk: Option<MovementBudget>,
    pub osvald: Option<osvald::Prediction>,
    pub encounter: Option<encounter::Prediction>,
    pub before_battle_initialization: Option<CrtRng>,
    pub cait_present: bool,
    pub non_surprise: bool,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub verification: &'static str,
}

pub fn project(arrival: CrtRng, state: WorldState, throne_level: u32) -> Result<Prediction> {
    let visit = village_return::visit(arrival, state)?;
    let mut l = Loader::new(world::catalog(), visit.world_after.clone(), visit.after)?;
    l.visit_map("Fld_Cty_1_2")?;
    l.boundary("Early clerics: returned from first Abandoned Village visit");
    let after_flats = l.rng;
    let world_at_flats = l.state.clone();
    let movement = village_return::snow_reach(l.state.encounter_counter)?;
    let cleric_reach = black_market::cleric_reach(l.state.encounter_counter)?;
    let mut p = Prediction {
        village_visit: visit, after_flats, world_at_flats, movement, cleric_reach,
        after_night: None, clerics_present: false, after_day: None, remaining_walk: None,
        osvald: None, encounter: None, before_battle_initialization: None,
        cait_present: false, non_surprise: false, trace: vec![], decisions: vec![],
        verification: "Source-derived fork, not live-validated. Daytime Flats return; clerics at 42 updates (40 measured +2); switch night, buy soulstones, immediately switch day. The two switches evaluate shared time lotteries and NPC refreshes without a counter reset. Ordinary purchases add no CRT draws. Assume the clerics are on the existing padded 72-walk/68-run-update route to Snows; carry walking reference counter. Load Snows, walk 70 updates to Osvald, recruit and skip his story/tutorials. Predict the first encounter in the same daytime land zone with actual Throné route level and Lv1 Osvald, carried FirstEncount, no rare bonuses and no intervening CRT actions. Endpoint is composition/situation, before enemy actor initialization; it does not simulate the Cait battle.",
    };
    if let Some(left) = p.cleric_reach.walking.counter_before_reset {
        l.state.encounter_counter = left;
        l.boundary("Flats return -> clerics: 42 daytime walking updates");
        l.set_time(4, "Early clerics: switch to night near the spawns")?;
        l.refresh_bar()?;
        l.refresh_npcs("Early clerics: night placements")?;
        p.after_night = Some(l.rng);
        p.clerics_present = black_market::clerics_present(&l.state);
        l.boundary("Early cleric purchases: ordinary shop transactions, no CRT draws");
        l.set_time(2, "Early clerics: switch back to day after purchases")?;
        l.refresh_bar()?;
        l.refresh_npcs("Early clerics: day placements")?;
        p.after_day = Some(l.rng);
        let updates = village_return::WALK_UPDATES - black_market::CLERIC_APPROACH_UPDATES;
        let remaining = MovementBudget::new(l.state.encounter_counter, updates, p.movement.walking.decrement);
        if let Some(left) = remaining.counter_before_reset {
            l.state.encounter_counter = left;
            l.boundary("Clerics -> Southern Cape Cold Snows: remaining daytime walking updates");
            let recruited = osvald::with_world_state(l.rng, l.state.clone())?;
            if let (Some(after), Some(w)) = (recruited.after_recruitment, &recruited.world_after) {
                let options = PredictionOptions {
                    map: osvald::MAP.into(), zone: "ENT_FLD_SNW_1_2_LND".into(),
                    period: "DayTime".into(), progress: 1, first_encounter: w.first_encounter,
                    boundary: Boundary::CurrentCounter, current_counter: Some(w.encounter_counter),
                    party_levels: vec![throne_level, 1], dash: true, ..Default::default()
                };
                ensure(w.time == 2 && w.party == [7,6], "Cait encounter requires recruited daytime party")?;
                let mut rng = after;
                let e = encounter::catalog().predict(&mut rng, &options)?;
                // A passed family gate alone does not guarantee an eligible replacement slot.
                let labels = &encounter::catalog().rare_labels["CAT_SNOW"];
                p.cait_present = e.enemies.iter().any(|slot| labels.contains(&slot.enemy.id));
                p.non_surprise = matches!(e.situation.as_deref(), Some("Normal"|"Preemptive"));
                p.before_battle_initialization = Some(rng);
                p.encounter = Some(e);
            }
            p.osvald = Some(recruited);
        }
        p.remaining_walk = Some(remaining);
    }
    p.trace = l.trace;
    p.decisions = l.decisions;
    Ok(p)
}

/// Four independent outcomes under the same Flats Escape control point.
pub fn outcomes(p: &Prediction) -> [bool;4] {
    [p.clerics_present, p.movement.walking.reaches_reset,
        p.osvald.as_ref().is_some_and(|o| o.after_recruitment.is_some()), p.cait_present && p.non_surprise]
}
