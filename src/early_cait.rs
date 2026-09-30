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
    /// Alternative first-encounter endpoints, before any encounter battle.
    pub snows_approach: Option<SnowsApproach>,
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

#[derive(Debug, Serialize)]
pub struct EncounterBranch {
    pub start: CrtRng,
    pub world: WorldState,
    pub encounter: encounter::Prediction,
    pub before_battle_initialization: CrtRng,
    pub non_surprise: bool,
}

#[derive(Debug, Serialize)]
pub struct SnowsApproach {
    pub measured_running_updates: u32,
    pub leeway: u32,
    pub running: MovementBudget,
    pub predicted_before_osvald: bool,
    /// Counterfactual endpoints share the same Flats plan. Neither includes a
    /// simulated escape or recruitment after a pre-Osvald encounter.
    pub before_osvald: EncounterBranch,
    pub after_osvald: EncounterBranch,
    pub both_non_surprise: bool,
    pub after_osvald_required: bool,
}

fn encounter_branch(start:CrtRng,world:WorldState,party_levels:Vec<u32>) -> Result<EncounterBranch> {
    let options=PredictionOptions {map:osvald::MAP.into(),zone:"ENT_FLD_SNW_1_2_LND".into(),
        period:if world.time==2 {"DayTime"} else {"Night"}.into(),progress:1,
        first_encounter:world.first_encounter,boundary:Boundary::CurrentCounter,
        current_counter:Some(world.encounter_counter),party_levels,dash:true,..Default::default()};
    let mut rng=start;
    let encounter=encounter::catalog().predict(&mut rng,&options)?;
    let non_surprise=matches!(encounter.situation.as_deref(),Some("Normal"|"Preemptive"));
    Ok(EncounterBranch {start,world,encounter,before_battle_initialization:rng,non_surprise})
}

pub fn project(arrival: CrtRng, state: WorldState, throne_level: u32) -> Result<Prediction> {
    project_with_switches(arrival,state,throne_level,0,0)
}

pub const TIME_SWITCH_INPUT_COST:u32=7;

/// Additional cleric switches must form full day/night cycles. Osvald switches
/// may end in either phase, and use that phase's actual encounter table.
pub fn project_with_switches(arrival:CrtRng,state:WorldState,throne_level:u32,
    cleric_extra:u32,osvald_switches:u32) -> Result<Prediction> {
    project_with_policy(arrival,state,throne_level,cleric_extra,osvald_switches,false)
}

pub fn project_with_policy(arrival:CrtRng,state:WorldState,throne_level:u32,
    cleric_extra:u32,osvald_switches:u32,require_cait:bool) -> Result<Prediction> {
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
        snows_approach:None,village_visit: visit, after_flats, world_at_flats, movement, cleric_reach,
        after_night: None, clerics_present: false, after_day: None,
        post_cleric_switches:1+cleric_extra,post_osvald_switches:osvald_switches,
        after_osvald_switches:None,world_at_encounter:None,encounter_time_trace:vec![],encounter_time_decisions:vec![],remaining_walk: None,
        osvald: None, encounter: None, before_battle_initialization: None,
        cait_present: false, non_surprise: false, trace: vec![], decisions: vec![],
        verification: "Source-derived fork, not live-validated. Flats retains its 42-update cleric stop and 72-walk/68-run total route. Run 64 updates from Snows entry to Osvald, no leeway. If encounter is inevitable before recruitment, require only that solo opening to avoid surprise. Otherwise protect both before/after timings for the player override. Both endpoints use actual party levels, progression, time, FirstEncount and CRT callbacks. The after-recruitment alternative assumes no earlier encounter; it does not simulate an earlier encounter's escape or claim subsequent recruitment RNG after that battle. The retained require_cait API uses its original safe 70-update walk. All encounter endpoints stop before enemy actor initialization.",
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
            // The retained Cait API keeps its original safe-walk experiment.
            // Ordinary routing allows the first encounter on either side of
            // recruitment. When perfect movement predicts recruitment first,
            // protect BOTH openings from the same control plan; otherwise the
            // solo opening is the only surprise-avoidance constraint.
            let recruited = if require_cait {osvald::with_world_state(l.rng,l.state.clone())?}
                else {osvald::with_approach(l.rng,l.state.clone(),
                    osvald::RUN_UPDATES,true,true)?};
            if let (Some(after), Some(w)) = (recruited.after_recruitment, &recruited.world_after) {
                let mut time_loader=Loader::new(world::catalog(),w.clone(),after)?;
                for _ in 0..osvald_switches {
                    let time=if time_loader.state.time==2 {4} else {2};
                    time_loader.set_time(time,"After Osvald recruitment: time switch")?;
                    time_loader.refresh_bar()?;
                    time_loader.refresh_npcs("After Osvald recruitment: time-switch placements")?;
                }
                ensure(w.time == 2 && w.party == [7,6], "Cait encounter requires recruited daytime party")?;
                p.after_osvald_switches=Some(time_loader.rng);
                p.world_at_encounter=Some(time_loader.state.clone());
                let before=encounter_branch(recruited.after_loading,recruited.world_at_entry.clone(),vec![throne_level])?;
                let after=encounter_branch(time_loader.rng,time_loader.state.clone(),vec![throne_level,1])?;
                let predicted_before=!require_cait && !recruited.walking.reaches_reset;
                let branch=if predicted_before {&before} else {&after};
                let e=branch.encounter.clone();
                // A passed family gate alone does not guarantee an eligible replacement slot.
                let labels = &encounter::catalog().rare_labels["CAT_SNOW"];
                p.cait_present = e.enemies.iter().any(|slot| labels.contains(&slot.enemy.id));
                p.non_surprise = if require_cait {after.non_surprise}
                    else {before.non_surprise && (predicted_before || after.non_surprise)};
                p.before_battle_initialization = Some(branch.before_battle_initialization);
                p.world_at_encounter=Some(branch.world.clone());
                p.encounter = Some(e);
                if !require_cait {p.snows_approach=Some(SnowsApproach {measured_running_updates:osvald::RUN_UPDATES,
                    leeway:0,running:MovementBudget::new(recruited.world_at_entry.encounter_counter,
                        osvald::RUN_UPDATES,before.encounter.decrement.unwrap()),predicted_before_osvald:predicted_before,
                    both_non_surprise:before.non_surprise && after.non_surprise,
                    after_osvald_required:!predicted_before,before_osvald:before,after_osvald:after});}
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
        p.snows_approach.is_some() || p.osvald.as_ref().is_some_and(|o| o.after_recruitment.is_some()), p.cait_present && p.non_surprise]
}

pub fn outcomes_for(p: &Prediction, require_cait: bool) -> [bool;4] {
    let mut goals = outcomes(p);
    goals[3] = p.non_surprise && (!require_cait || p.cait_present);
    goals
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world() -> WorldState {
        let mut w=world::throne_after_second_state().unwrap();
        w.map="Fld_Cty_1_2".into(); w.time=2; w.party=vec![7];
        w.active_scenarios.clear(); w.prologues.clear(); w.story_types.clear();
        w.first_encounter=false; w.encounter_counter=515;
        w.visited_maps.insert("Fld_Cty_1_1".into());
        w.resident_levels=world::catalog().maps[&w.map].streams.iter().cloned().collect();
        w
    }
    #[test]
    fn running_timing_uses_exact_distance_and_replays_two_independent_parties() {
        let mut covered=[false;2];
        let mut before_only=false;
        for seed in 0..256 {
            let p=project(CrtRng::new(seed),world(),5).unwrap();
            let Some(a)=&p.snows_approach else {continue};
            covered[usize::from(a.predicted_before_osvald)]=true;
            assert_eq!((a.running.updates,a.running.cost,a.leeway),(64,768,0));
            assert_eq!(a.predicted_before_osvald,a.before_osvald.world.encounter_counter<=768);
            assert_eq!(a.before_osvald.world.party,[7]);
            assert_eq!(a.after_osvald.world.party,[7,6]);
            assert_eq!(a.after_osvald_required,!a.predicted_before_osvald);
            assert_eq!(p.non_surprise,a.before_osvald.non_surprise
                && (a.predicted_before_osvald || a.after_osvald.non_surprise));
            before_only |= a.predicted_before_osvald && a.before_osvald.non_surprise && !a.after_osvald.non_surprise;
            for (branch,party) in [(&a.before_osvald,vec![5]),(&a.after_osvald,vec![5,1])] {
                let replay=encounter_branch(branch.start,branch.world.clone(),party).unwrap();
                assert_eq!(replay.before_battle_initialization,branch.before_battle_initialization);
                assert_eq!(serde_json::to_value(replay.encounter).unwrap(),serde_json::to_value(&branch.encounter).unwrap());
            }
            let selected=if a.predicted_before_osvald {&a.before_osvald} else {&a.after_osvald};
            assert_eq!(p.before_battle_initialization,Some(selected.before_battle_initialization));
        }
        assert_eq!(covered,[true;2]);
        assert!(before_only,"An irrelevant post-recruitment surprise must not reject an otherwise valid pre-recruitment plan");
    }
    #[test]
    fn post_recruitment_time_switches_cannot_change_the_solo_opening() {
        for seed in 0..100 {
            let first=project(CrtRng::new(seed),world(),5).unwrap();
            let Some(a)=first.snows_approach else {continue};
            let later=project_with_switches(CrtRng::new(seed),world(),5,0,1).unwrap();
            let b=later.snows_approach.unwrap();
            assert_eq!(a.before_osvald.before_battle_initialization,b.before_osvald.before_battle_initialization);
            assert_eq!(a.before_osvald.non_surprise,b.before_osvald.non_surprise);
            assert_eq!(a.after_osvald.world.time,2);
            assert_eq!(b.after_osvald.world.time,4);
            return;
        }
        panic!("No reachable Snows prefix");
    }
}
