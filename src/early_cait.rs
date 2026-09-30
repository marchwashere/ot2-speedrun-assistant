//! First village visit -> early clerics -> daytime Osvald -> first Snows encounter.
//! No battle or boost RNG is duplicated here; callers replay their Escape plan.
use crate::{black_market, encounter::{self, Boundary, PredictionOptions}, ensure, osvald,
    route::MovementBudget, village_return, world::{self, Decision, Loader, Trace, WorldState}, CrtRng, Result};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Settings {
    pub clerics_max_boost_repeats: Option<u32>,
    pub cait_max_boost_repeats: Option<u32>,
    /// Cait selection is opt-in; avoiding surprise remains an independent goal.
    pub require_cait: bool,
}
impl Default for Settings {
    fn default() -> Self { Self { clerics_max_boost_repeats: None, cait_max_boost_repeats: None, require_cait: false } }
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
    /// Includes the normal night -> day switch; always odd to finish daytime.
    pub post_cleric_switches: u32,
    pub post_osvald_switches: u32,
    pub after_osvald_switches: Option<CrtRng>,
    pub world_at_encounter: Option<WorldState>,
    pub encounter_time_trace: Vec<Trace>,
    pub encounter_time_decisions: Vec<Decision>,
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
    project_with_switches(arrival,state,throne_level,0,0)
}

pub const TIME_SWITCH_INPUT_COST:u32=7;

/// Additional cleric switches must form full day/night cycles. Osvald switches
/// may end in either phase, and use that phase's actual encounter table.
pub fn project_with_switches(arrival:CrtRng,state:WorldState,throne_level:u32,
    cleric_extra:u32,osvald_switches:u32) -> Result<Prediction> {
    ensure(cleric_extra%2==0,"Extra cleric switches must end back on day")?;
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
        after_night: None, clerics_present: false, after_day: None,
        post_cleric_switches:1+cleric_extra,post_osvald_switches:osvald_switches,
        after_osvald_switches:None,world_at_encounter:None,encounter_time_trace:vec![],encounter_time_decisions:vec![],remaining_walk: None,
        osvald: None, encounter: None, before_battle_initialization: None,
        cait_present: false, non_surprise: false, trace: vec![], decisions: vec![],
        verification: "Source-derived fork, not live-validated. Daytime Flats return; clerics at 42 updates (40 measured +2); switch night, buy soulstones, then an odd number of switches to end daytime. Shared time lotteries and NPC refreshes do not reset the movement counter. Ordinary purchases add no CRT draws. Total Flats movement: 72-walk/68-run updates including clerics. Load Snows, walk 70 updates to Osvald, recruit and skip his story/tutorials. Replay post-recruitment time switches with the carried party/progression and counter. Predict the first land encounter in the final phase's actual table, actual Throné route level + Lv1 Osvald, carried FirstEncount, no rare bonuses and no other CRT actions. Endpoint is composition/situation before enemy actor initialization; no Cait battle is simulated.",
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
        for _ in 0..=cleric_extra {
            let time=if l.state.time==4 {2} else {4};
            l.set_time(time, "After cleric purchases: time switch")?;
            l.refresh_bar()?;
            l.refresh_npcs("After cleric purchases: time-switch placements")?;
        }
        p.after_day = Some(l.rng);
        let updates = village_return::WALK_UPDATES - black_market::CLERIC_APPROACH_UPDATES;
        let remaining = MovementBudget::new(l.state.encounter_counter, updates, p.movement.walking.decrement);
        if let Some(left) = remaining.counter_before_reset {
            l.state.encounter_counter = left;
            l.boundary("Clerics -> Southern Cape Cold Snows: remaining daytime walking updates");
            let recruited = osvald::with_world_state(l.rng, l.state.clone())?;
            if let (Some(after), Some(w)) = (recruited.after_recruitment, &recruited.world_after) {
                let mut time_loader=Loader::new(world::catalog(),w.clone(),after)?;
                for _ in 0..osvald_switches {
                    let time=if time_loader.state.time==2 {4} else {2};
                    time_loader.set_time(time,"After Osvald recruitment: time switch")?;
                    time_loader.refresh_bar()?;
                    time_loader.refresh_npcs("After Osvald recruitment: time-switch placements")?;
                }
                let encounter_world=&time_loader.state;
                let options = PredictionOptions {
                    map: osvald::MAP.into(), zone: "ENT_FLD_SNW_1_2_LND".into(),
                    period: if encounter_world.time==2 {"DayTime"} else {"Night"}.into(), progress: 1, first_encounter: encounter_world.first_encounter,
                    boundary: Boundary::CurrentCounter, current_counter: Some(encounter_world.encounter_counter),
                    party_levels: vec![throne_level, 1], dash: true, ..Default::default()
                };
                ensure(w.time == 2 && w.party == [7,6], "Cait encounter requires recruited daytime party")?;
                p.after_osvald_switches=Some(time_loader.rng);
                p.world_at_encounter=Some(time_loader.state.clone());
                let mut rng = time_loader.rng;
                let e = encounter::catalog().predict(&mut rng, &options)?;
                // A passed family gate alone does not guarantee an eligible replacement slot.
                let labels = &encounter::catalog().rare_labels["CAT_SNOW"];
                p.cait_present = e.enemies.iter().any(|slot| labels.contains(&slot.enemy.id));
                p.non_surprise = matches!(e.situation.as_deref(), Some("Normal"|"Preemptive"));
                p.before_battle_initialization = Some(rng);
                p.encounter = Some(e);
                p.encounter_time_trace=time_loader.trace;
                p.encounter_time_decisions=time_loader.decisions;
            }
            p.osvald = Some(recruited);
        }
        p.remaining_walk = Some(remaining);
    }
    p.trace = l.trace;
    p.decisions = l.decisions;
    Ok(p)
}

/// Four outcomes across Flats boosts and both later time-switch control points.
pub fn outcomes(p: &Prediction) -> [bool;4] {
    [p.clerics_present, p.movement.walking.reaches_reset,
        p.osvald.as_ref().is_some_and(|o| o.after_recruitment.is_some()), p.cait_present && p.non_surprise]
}

pub fn outcomes_for(p: &Prediction, require_cait: bool) -> [bool;4] {
    let mut goals = outcomes(p);
    goals[3] = p.non_surprise && (!require_cait || p.cait_present);
    goals
}
