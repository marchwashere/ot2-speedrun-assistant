//! Day switch -> Al introduction -> first land encounter. R8 manipulation and
//! first-menu Escape are separate searches with independently bounded inputs.
use crate::{
    CrtRng, Result, SimError,
    encounter::{self, Boundary, PredictionOptions},
    ensure, flats,
    manipulation::{BoostInputs, describe_inputs},
    pirro, pirro_aftermath, second,
    world::{self, Decision, Loader, Trace, WorldState},
};
use ot2_battle_engine::{
    Event, Outcome, Resource,
    escape::Situation,
    field_escape::{
        FieldBreakEscape, FieldEscapeAttempt, FieldEscapeMenu, first_action_break,
        level_for_experience, route_battle_base_exp,
    },
};
use serde::Serialize;
use std::cell::Cell;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Settings {
    pub approach_max_boost_repeats: Option<u32>,
    pub escape_max_boost_repeats: Option<u32>,
    pub flats_max_boost_repeats: Option<u32>,
    pub flats_approach_max_boost_repeats: Option<u32>,
    pub flats_break_max_boost_repeats: Option<u32>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            approach_max_boost_repeats: Some(4),
            escape_max_boost_repeats: Some(4),
            flats_max_boost_repeats: Some(4),
            flats_approach_max_boost_repeats: Some(4),
            flats_break_max_boost_repeats: Some(0),
        }
    }
}
#[derive(Debug, Serialize)]
pub struct ApproachSearch {
    pub max_boost_repeats: Option<u32>,
    pub tested_candidates: u64,
    pub target_met: bool,
    pub plan: Vec<u8>,
    pub status: &'static str,
}
#[derive(Debug, Serialize)]
pub struct Escape {
    pub menu: FieldEscapeMenu,
    pub baseline: FieldEscapeAttempt,
    pub inputs: Option<BoostInputs>,
    pub attempt: Option<FieldEscapeAttempt>,
    pub method: EscapeMethod,
    pub break_action: Option<FieldBreakEscape>,
    pub status: &'static str,
    pub flats_search: Option<FlatsSearch>,
    pub flats_approach_search: Option<FlatsApproachSearch>,
    pub flats_break_search: Option<FlatsApproachSearch>,
    pub tested_candidates: u64,
}
#[derive(Debug, Serialize)]
pub struct FlatsApproachSearch {
    pub max_extra_boost_repeats: Option<u32>,
    pub prior_repeats: u32,
    pub added_repeats: u32,
    pub tested_candidates: u64,
    pub target_met: bool,
    pub status: &'static str,
}
#[derive(Debug, Serialize)]
pub struct FlatsSearch {
    pub max_extra_boost_repeats: Option<u32>,
    pub escape_repeats: u32,
    pub added_repeats: u32,
    pub tested_candidates: u64,
    pub target_met: bool,
    pub baseline_counter: u32,
    pub selected_counter: u32,
    pub required_counter: u32,
    pub status: &'static str,
}
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EscapeMethod {
    WithinLimit,
    BreakThenEscape,
    UnlimitedFallback,
    FlatsAdjustment,
    Unavailable,
}

#[derive(Debug, Serialize)]
pub struct FieldReturn {
    pub arrival: CrtRng,
    pub after: CrtRng,
    pub trace: Vec<Trace>,
    pub world_after: WorldState,
}

/// Resume the field after the selected successful Escape, not a search trial.
pub fn return_to_field(world: &WorldState, attempt: &FieldEscapeAttempt) -> Result<FieldReturn> {
    restore_field(world, attempt, "Fld_Cty_1_3", "Highroad Escape")
}

pub(crate) fn restore_field(
    world: &WorldState,
    attempt: &FieldEscapeAttempt,
    map: &str,
    label: &str,
) -> Result<FieldReturn> {
    ensure(
        attempt.success,
        "Cannot return to field after a failed Escape",
    )?;
    ensure(
        world.map == map && world.time == 2,
        "Escape return requires the expected daytime field",
    )?;
    let mut state = world.clone();
    state.first_encounter = false;
    let mut loader = Loader::new(world::catalog(), state, attempt.after)?;
    loader.restore_field_after_battle(&format!("Return from successful {label}"))?;
    loader.boundary(format!("{label}: field control restored"));
    Ok(FieldReturn {
        arrival: attempt.after,
        after: loader.rng,
        trace: loader.trace,
        world_after: loader.state,
    })
}

#[cfg(test)]
fn escape_plan(
    entry: CrtRng,
    situation: Situation,
    level: u32,
    enemies: &[&str],
    danger: u32,
    cap: Option<u32>,
) -> Result<Escape> {
    escape_plan_for_target(entry, situation, level, enemies, danger, cap, |_| Ok(true))
}

/// Search direct Escape, then a real immediate break, then unbounded direct
/// Escape. Every strategy must satisfy the caller's post-battle target too.
#[cfg(test)]
pub(crate) fn escape_plan_for_target(
    entry: CrtRng,
    situation: Situation,
    level: u32,
    enemies: &[&str],
    danger: u32,
    cap: Option<u32>,
    accepts: impl Fn(&FieldEscapeAttempt) -> Result<bool>,
) -> Result<Escape> {
    escape_plan_with_limit(entry, situation, level, enemies, danger, cap, None, accepts)
}

/// `cap` chooses when to prefer a viable break. `total_limit` bounds an
/// optional improvement; neither permits violating the supplied outcomes.
pub(crate) fn escape_plan_with_limit(
    entry: CrtRng,
    situation: Situation,
    level: u32,
    enemies: &[&str],
    danger: u32,
    cap: Option<u32>,
    total_limit: Option<u32>,
    accepts: impl Fn(&FieldEscapeAttempt) -> Result<bool>,
) -> Result<Escape> {
    let menu = FieldEscapeMenu::new(entry, situation, level, enemies)?;
    let baseline = menu.clone().attempt(&[], danger)?;
    let mut result = Escape {
        menu,
        baseline,
        inputs: None,
        attempt: None,
        method: EscapeMethod::Unavailable,
        break_action: None,
        status: "No Escape solution in the CRT cycle",
        flats_search: None,
        flats_approach_search: None,
        flats_break_search: None,
        tested_candidates: 0,
    };
    let tested = Cell::new(0u64);
    // Searches clone the untouched first menu. Testing a failed check never
    // changes the state used by the break fallback or another boost candidate.
    let search = |start: u64, end: u64| -> Result<Option<(BoostInputs, FieldEscapeAttempt)>> {
        for repeats in start..=end {
            tested.set(tested.get() + 1);
            let path: Vec<u8> = (0..repeats).flat_map(|_| [1, 0]).collect();
            let attempt = result.menu.clone().attempt(&path, danger)?;
            if attempt.success && accepts(&attempt)? {
                return Ok(Some((describe_inputs(repeats as u32, 0, path)?, attempt)));
            }
        }
        Ok(None)
    };
    let end = u64::from(total_limit.unwrap_or(u32::MAX));
    if let Some((inputs, attempt)) = search(0, cap.map_or(end, |n| u64::from(n).min(end)))? {
        result.inputs = Some(inputs);
        result.attempt = Some(attempt);
        result.method = EscapeMethod::WithinLimit;
        result.status = "First Escape succeeds";
        result.tested_candidates = tested.get();
        return Ok(result);
    }
    if let Some(cap) = cap {
        tested.set(tested.get() + 1);
        if let Some(plan) = first_action_break(entry, situation, level, enemies)? {
            ensure(
                plan.at_attack_menu == result.menu.rng,
                "Break fallback menu disagrees with Escape initialization",
            )?;
            if accepts(&plan.attempt)? {
                result.attempt = Some(plan.attempt.clone());
                result.break_action = Some(plan);
                result.method = EscapeMethod::BreakThenEscape;
                result.status = "Break now, then guaranteed Escape";
                result.tested_candidates = tested.get();
                return Ok(result);
            }
        }
        if let Some((inputs, attempt)) = search(u64::from(cap) + 1, end)? {
            result.inputs = Some(inputs);
            result.attempt = Some(attempt);
            result.method = EscapeMethod::UnlimitedFallback;
            result.status = "First Escape succeeds beyond break preference";
        }
    }
    result.tested_candidates = tested.get();
    Ok(result)
}

/// Required outcomes are evaluated together BEFORE choosing Escape strategy.
/// Numeric goals are then attempted as bounded improvements of that valid plan.
fn plan_route_escape(
    world: &WorldState,
    entry: CrtRng,
    situation: Situation,
    level: u32,
    enemies: &[&str],
    danger: u32,
    settings: Settings,
) -> Result<Escape> {
    let project = |a: &FieldEscapeAttempt| -> Result<(u32, bool, bool, bool)> {
        let f = flats::after_highroad(&return_to_field(world, a)?)?;
        let counter = f.world_after.encounter_counter;
        let walk = f.harbor_reach.walking.reaches_reset;
        let (safe, zero_bp_break) = if let Some(h) = crate::harbor::after_flats(&f)? {
            let (entry, e)=crate::flats_escape::first_encounter(h.after, &h.world_after, level)?;
            (e.situation.as_deref()!=Some("Surprise"), crate::flats_escape::unboosted_opening_break(entry,&e,level)?)
        } else {
            (false, false)
        };
        Ok((counter, walk, safe, zero_bp_break))
    };
    let required_walk = settings.flats_max_boost_repeats.is_none()
        || settings.flats_approach_max_boost_repeats.is_none()
        || settings.flats_break_max_boost_repeats.is_none();
    let required_safe = settings.flats_approach_max_boost_repeats.is_none();
    let required_break = settings.flats_break_max_boost_repeats.is_none();
    let required = |a: &FieldEscapeAttempt| -> Result<bool> {
        if !required_walk {
            return Ok(true);
        }
        let (_, walk, safe, zero_bp_break) = project(a)?;
        Ok(walk && (!required_safe || safe) && (!required_break || zero_bp_break))
    };
    let choose = |limit, accepts: &dyn Fn(&FieldEscapeAttempt) -> Result<bool>| {
        escape_plan_with_limit(
            entry,
            situation,
            level,
            enemies,
            danger,
            settings.escape_max_boost_repeats,
            limit,
            accepts,
        )
    };
    let mut plan = choose(None, &required)?;
    let Some(a) = &plan.attempt else {
        return Ok(plan);
    };
    let (counter, walk, _, _) = project(a)?;
    let base = plan.inputs.as_ref().map_or(0, |p| p.extra_repeats);
    let mut walking = FlatsSearch {
        max_extra_boost_repeats: settings.flats_max_boost_repeats,
        escape_repeats: base,
        added_repeats: 0,
        tested_candidates: 0,
        target_met: walk,
        baseline_counter: counter,
        selected_counter: counter,
        required_counter: flats::harbor_reach(counter)?.walking.minimum_start_counter,
        status: "Harbor walk satisfied by the required-outcome plan",
    };
    if !walk && let Some(extra) = settings.flats_max_boost_repeats {
        let candidate = choose(Some(base.saturating_add(extra)), &|a| {
            Ok(required(a)? && project(a)?.1)
        })?;
        walking.tested_candidates = candidate.tested_candidates;
        if candidate.attempt.is_some() {
            walking.target_met = true;
            walking.added_repeats = candidate
                .inputs
                .as_ref()
                .map_or(0, |p| p.extra_repeats)
                .saturating_sub(base);
            walking.status = "Optional harbor walk achieved; required outcomes preserved";
            plan = candidate;
            if plan.method != EscapeMethod::BreakThenEscape {
                plan.method = EscapeMethod::FlatsAdjustment;
            }
        } else {
            walking.status =
                "Optional harbor walk not found within budget; keeping required outcomes";
        }
    }
    let (_, _, safe, _) = project(plan.attempt.as_ref().unwrap())?;
    let prior = plan.inputs.as_ref().map_or(0, |p| p.extra_repeats);
    let mut approach = FlatsApproachSearch {
        max_extra_boost_repeats: settings.flats_approach_max_boost_repeats,
        prior_repeats: prior,
        added_repeats: 0,
        tested_candidates: 0,
        target_met: safe,
        status: "Non-surprise Flats opening satisfied",
    };
    if !walking.target_met {
        approach.status = "Optional Flats approach unavailable: harbor walk not reached";
    } else if !safe && let Some(extra) = settings.flats_approach_max_boost_repeats {
        let candidate = choose(Some(prior.saturating_add(extra)), &|a| {
            let (_, walk, safe, _) = project(a)?;
            Ok(walk && safe && required(a)?)
        })?;
        approach.tested_candidates = candidate.tested_candidates;
        if candidate.attempt.is_some() {
            approach.target_met = true;
            approach.added_repeats = candidate
                .inputs
                .as_ref()
                .map_or(0, |p| p.extra_repeats)
                .saturating_sub(prior);
            approach.status =
                "Optional non-surprise Flats opening achieved; required outcomes preserved";
            plan = candidate;
            if plan.method != EscapeMethod::BreakThenEscape {
                plan.method = EscapeMethod::FlatsAdjustment;
            }
        } else {
            approach.status = "Optional non-surprise Flats opening not found within budget; keeping required outcomes";
        }
    }
    let prior=plan.inputs.as_ref().map_or(0,|p|p.extra_repeats);
    let mut break_search=FlatsApproachSearch {
        max_extra_boost_repeats:settings.flats_break_max_boost_repeats,prior_repeats:prior,
        added_repeats:0,tested_candidates:0,target_met:project(plan.attempt.as_ref().unwrap())?.3,
        status:"Unboosted Flats break available in selected plan",
    };
    if !break_search.target_met && let Some(extra)=settings.flats_break_max_boost_repeats && extra>0 {
        let candidate=choose(Some(prior.saturating_add(extra)), &|a| {
            let (_,walk,safe,can_break)=project(a)?;
            Ok(required(a)? && can_break && (!walking.target_met||walk) && (!approach.target_met||safe))
        })?;
        break_search.tested_candidates=candidate.tested_candidates;
        if candidate.attempt.is_some() {
            break_search.target_met=true;
            break_search.added_repeats=candidate.inputs.as_ref().map_or(0,|p|p.extra_repeats).saturating_sub(prior);
            plan=candidate;
            if plan.method!=EscapeMethod::BreakThenEscape {plan.method=EscapeMethod::FlatsAdjustment;}
        } else {break_search.status="Optional unboosted Flats break not found within budget; preceding outcomes preserved";}
    }
    if !break_search.target_met && settings.flats_break_max_boost_repeats==Some(0) {
        break_search.status="Unboosted Flats break target not met; optional budget is zero";
    }
    let (counter,walk,safe,_) = project(plan.attempt.as_ref().unwrap())?;
    walking.selected_counter=counter; walking.target_met=walk; approach.target_met=safe;
    ensure(
        required(plan.attempt.as_ref().unwrap())?,
        "Highroad plan violated required outcomes",
    )?;
    plan.flats_search = Some(walking);
    plan.flats_approach_search = Some(approach);
    plan.flats_break_search = Some(break_search);
    Ok(plan)
}
#[derive(Debug, Serialize)]
pub struct Prediction {
    pub settings: Settings,
    pub arrival: CrtRng,
    pub after_al: CrtRng,
    pub world_after_al: WorldState,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub level: u32,
    pub encounter: encounter::Prediction,
    pub before_battle_initialization: CrtRng,
    pub escape: Option<Escape>,
    pub field_return: Option<FieldReturn>,
    pub approach_search: Option<ApproachSearch>,
    pub verification: &'static str,
}

fn reward(base: u32, party: usize, steps: &[second::Step]) -> u32 {
    let mut broken = false;
    let mut overkill = false;
    for step in steps {
        let mut damage = std::collections::BTreeMap::new();
        for event in &step.report.events {
            match *event {
                Event::Broken { actor, .. } if actor.0 >= party => broken = true,
                Event::Damage { target, amount, .. } => {
                    damage.insert(target, amount);
                }
                Event::ResourceChanged {
                    actor,
                    resource: Resource::Hp,
                    before,
                    after: 0,
                } if actor.0 >= party && before > 0 => {
                    overkill |= damage
                        .get(&actor)
                        .is_some_and(|&n| u64::from(n) >= 2 * u64::from(before));
                }
                _ => {}
            }
        }
    }
    base + (u32::from(broken) + u32::from(overkill)) * (base / 10).max(1)
}
pub fn route_level(second: &second::Prediction, boss: &pirro::Prediction) -> Result<u32> {
    let before = second.opening_exp
        + reward(
            route_battle_base_exp("ENG_EVE_THI_C01_020")?,
            3,
            &second.steps,
        );
    let after = before + reward(route_battle_base_exp(boss.group)?, 1, &boss.steps);
    let level = level_for_experience(after);
    ensure(
        level > level_for_experience(before),
        "Highroad healthy-HP profile requires the post-Pirro level-up",
    )?;
    Ok(level)
}

pub fn predict(
    ending: &pirro_aftermath::Prediction,
    level: u32,
    settings: Settings,
) -> Result<Prediction> {
    predict_internal(ending, level, settings, true)
}
fn predict_internal(
    ending: &pirro_aftermath::Prediction,
    level: u32,
    settings: Settings,
    plan_escape: bool,
) -> Result<Prediction> {
    ensure(
        settings.approach_max_boost_repeats.is_none_or(|n| n <= 256)
            && settings.escape_max_boost_repeats.is_none_or(|n| n <= 256)
            && settings.flats_max_boost_repeats.is_none_or(|n| n <= 256)
            && settings
                .flats_approach_max_boost_repeats
                .is_none_or(|n| n <= 256)
            && settings.flats_break_max_boost_repeats.is_none_or(|n|n<=256),
        "Highroad caps must be 0..256 or unlimited",
    )?;
    let world = &ending.world_after;
    ensure(
        world.map == "Fld_Cty_1_3"
            && world.time == 1
            && world.party == [7]
            && world.first_encounter
            && world.flag(8019)
            && !world.flag(20505),
        "Highroad stage requires fresh post-chapter Dawn before Al's introduction",
    )?;
    let mut loader = Loader::new(world::catalog(), world.clone(), ending.after)?;
    loader.set_time(2, "Immediate manual daytime switch")?;
    loader.refresh_npcs("Manual daytime switch: new phase placements")?;
    loader.skipped_event("SC_FC13_Tutorial_0100")?;
    loader.refresh_npcs("Al introduction: quest flag refresh")?;
    loader.boundary("Al introduction and tutorial dismissed");
    let options = PredictionOptions {
        map: loader.state.map.clone(),
        zone: "ENT_FLD_CTY_1_3_LND".into(),
        period: "DayTime".into(),
        progress: 1,
        first_encounter: loader.state.first_encounter,
        boundary: Boundary::CurrentCounter,
        current_counter: Some(loader.state.encounter_counter),
        party_levels: vec![level],
        dash: true,
        ..Default::default()
    };
    let mut entry = loader.rng;
    let encounter = encounter::catalog().predict(&mut entry, &options)?;
    let situation = match encounter.situation.as_deref() {
        Some("Normal") => Situation::Normal,
        Some("Preemptive") => Situation::Preemptive,
        Some("Surprise") => Situation::Surprise,
        _ => return Err(SimError("Unknown highroad situation".into())),
    };
    let escape = if situation == Situation::Surprise || !plan_escape {
        None
    } else {
        let enemies: Vec<_> = encounter
            .enemies
            .iter()
            .map(|e| e.enemy.id.as_str())
            .collect();
        let volume = &encounter::catalog().volumes[encounter.volume.as_ref().unwrap()];
        let danger = volume
            .encounters
            .iter()
            .rev()
            .find(|r| r.table != "None" && r.progress <= options.progress)
            .ok_or_else(|| SimError("Missing highroad danger row".into()))?
            .danger;
        let plan = plan_route_escape(
            &loader.state,
            entry,
            situation,
            level,
            &enemies,
            danger,
            settings,
        )?;
        Some(plan)
    };
    let field_return = escape
        .as_ref()
        .and_then(|e| e.attempt.as_ref())
        .map(|attempt| return_to_field(&loader.state, attempt))
        .transpose()?;
    Ok(Prediction {
        settings,
        arrival: ending.after,
        after_al: loader.rng,
        world_after_al: loader.state,
        trace: loader.trace,
        decisions: loader.decisions,
        level,
        encounter,
        before_battle_initialization: entry,
        escape,
        field_return,
        approach_search: None,
        verification: "Live capture verifies all 10 CRT calls from menu6002 through five boost cycles, Escape, two field gondola callbacks and counter B/A to6012. Other starts remain source-derived. Immediate daytime switch, skipped Al introduction, unchanged equipment, no intervening CRT actions; first land encounter. Try Escape within the cap, then an immediate single-enemy break, otherwise unlimited boost search. Break uses actual hit rolls and guarantees next-menu Escape without a lottery. Surprise combat and failed-Escape continuation are not approximated.",
    })
}

/// Freeze the selected R3 plan. The first no-grape R8 menu ends at 3 BP swords
/// in both latent branches. Early latent's activation occurs BEFORE boost input.
pub fn from_battle(
    boss: &mut pirro::Prediction,
    second: &second::Prediction,
    settings: Settings,
    replay: Option<Vec<u8>>,
) -> Result<Option<(pirro_aftermath::Prediction, Prediction)>> {
    let Some(ending) = pirro_aftermath::after_battle(boss)? else {
        return Ok(None);
    };
    let mut prediction = predict(&ending, route_level(second, boss)?, settings)?;
    let mut ending = ending;
    let mut info = ApproachSearch {
        max_boost_repeats: settings.approach_max_boost_repeats,
        tested_candidates: 0,
        target_met: prediction.encounter.situation.as_deref() != Some("Surprise"),
        plan: vec![1, 2, 3],
        status: "Non-surprise opening",
    };
    let no_grape = boss.routine.heal_before_round_eight == Some(false);
    if (!info.target_met || replay.is_some()) && no_grape {
        let r3 = boss
            .boost_search
            .as_ref()
            .map(|s| s.plan.clone())
            .or_else(|| boss.manipulation.as_ref().map(|m| m.inputs.levels.clone()))
            .unwrap_or(vec![1, 2, 3]);
        let menu = pirro::r8_menu(boss, &r3)?;
        let mut world = boss.world_at_entry.clone();
        world.set_flag(8017, false);
        let inventory = pirro::replay_inventory(boss)?;
        let run = |path: &[u8]| {
            pirro::predict_with_plans(
                boss.arrival,
                &world,
                boss.initial_state.actors[0].resources.clone(),
                inventory.clone(),
                &r3,
                path,
            )
        };
        let accepts = |p: &pirro::Prediction| -> Result<bool> {
            if p.state.outcome != Outcome::Victory
                || p.routine_error.is_some()
                || p.routine.heal_before_round_eight != Some(false)
            {
                return Ok(false);
            }
            let after = pirro_aftermath::after_battle(p)?.unwrap();
            let candidate = predict_internal(
                &after,
                route_level(second, p)?,
                Settings {
                    escape_max_boost_repeats: Some(0),
                    ..settings
                },
                false,
            )?;
            Ok(candidate.encounter.situation.as_deref() != Some("Surprise"))
        };
        let solution = if let Some(path) = replay {
            ensure(
                settings.approach_max_boost_repeats.is_none_or(|cap| {
                    pirro::describe_path(&path).is_ok_and(|i| i.extra_repeats <= cap)
                }),
                "Pirro R8 replay exceeds its repeat budget",
            )?;
            let p = run(&path)?;
            // A zero-adjustment export may represent a failed capped search.
            ensure(
                path == [1, 2, 3] || accepts(&p)?,
                "Pirro R8 replay does not preserve the target",
            )?;
            Some((p, path))
        } else {
            let (solution, tested) = pirro::search_paths(
                &menu,
                menu.can_use_latent(),
                settings.approach_max_boost_repeats,
                &run,
                &accepts,
            )?;
            info.tested_candidates = tested;
            solution
        };
        if let Some((mut selected, plan)) = solution {
            ending = pirro_aftermath::after_battle(&selected)?.unwrap();
            prediction = predict(&ending, route_level(second, &selected)?, settings)?;
            selected.boost_search = boss.boost_search.take();
            *boss = selected;
            info.plan = plan;
        }
    } else if let Some(path) = replay {
        ensure(
            path == [1, 2, 3],
            "R8 boost replay unavailable in the healing branch",
        )?;
    }
    info.target_met = prediction.encounter.situation.as_deref() != Some("Surprise");
    if !info.target_met {
        info.status = if no_grape {
            "No non-surprise plan within R8 repeat limit"
        } else {
            "R8 swords unavailable in the healing branch"
        };
    }
    prediction.approach_search = Some(info);
    Ok(Some((ending, prediction)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn highroad_world() -> WorldState {
        let city = crate::city::after_second_victory(CrtRng::new(0)).unwrap();
        let parlor = crate::parlor::after_city(&city).unwrap();
        let revisit = crate::parlor_revisit::after_first_visit(&parlor).unwrap();
        let mansion = crate::mansion::after_parlor_revisit(&revisit).unwrap();
        let mut state = mansion.lookout_ambush.world_after;
        state.set_flag(8017, true);
        state.first_encounter = false;
        let ending = pirro_aftermath::with_world_state(CrtRng::new(0), state).unwrap();
        predict_internal(&ending, 5, Settings::default(), false)
            .unwrap()
            .world_after_al
    }

    #[test]
    fn required_route_outcomes_gate_every_escape_strategy_before_optional_budgets() {
        let world = highroad_world();
        let enemies = ["ENE_NML_FLD_CTY_010"];
        let meets = |a: &FieldEscapeAttempt| {
            let f = flats::after_highroad(&return_to_field(&world, a).unwrap()).unwrap();
            let walk = f.harbor_reach.walking.reaches_reset;
            let safe = crate::harbor::after_flats(&f).unwrap().is_some_and(|h| {
                crate::flats_escape::first_encounter(h.after, &h.world_after, 5)
                    .unwrap()
                    .1
                    .situation
                    .as_deref()
                    != Some("Surprise")
            });
            (walk, safe)
        };
        let mut accepted_break = false;
        let mut rejected_break = false;
        for seed in 0..32 {
            let mut entry = CrtRng::new(0);
            entry.advance(seed * 7919);
            entry.draws = 6000;
            let settings = Settings {
                escape_max_boost_repeats: Some(0),
                flats_max_boost_repeats: Some(0),
                flats_approach_max_boost_repeats: None,
                ..Default::default()
            };
            // Required non-surprise includes reaching Harbor, even with an
            // optional walking budget of zero. Validate the full loader replay.
            let plan =
                plan_route_escape(&world, entry, Situation::Normal, 5, &enemies, 5, settings)
                    .unwrap();
            assert_eq!(meets(plan.attempt.as_ref().unwrap()), (true, true));
            let baseline_valid = plan.baseline.success && meets(&plan.baseline) == (true, true);
            let break_plan = first_action_break(entry, Situation::Normal, 5, &enemies).unwrap();
            let break_valid = break_plan
                .as_ref()
                .is_some_and(|p| meets(&p.attempt) == (true, true));
            assert_eq!(
                plan.method,
                if baseline_valid {
                    EscapeMethod::WithinLimit
                } else if break_valid {
                    accepted_break = true;
                    EscapeMethod::BreakThenEscape
                } else {
                    rejected_break |= break_plan.is_some();
                    EscapeMethod::UnlimitedFallback
                }
            );
            let both_required = plan_route_escape(
                &world,
                entry,
                Situation::Normal,
                5,
                &enemies,
                5,
                Settings {
                    flats_max_boost_repeats: None,
                    ..settings
                },
            )
            .unwrap();
            assert_eq!(
                both_required.attempt.unwrap().after,
                plan.attempt.unwrap().after
            );
            // An optional non-surprise improvement cannot destroy required walking.
            let optional = plan_route_escape(
                &world,
                entry,
                Situation::Normal,
                5,
                &enemies,
                5,
                Settings {
                    flats_max_boost_repeats: None,
                    flats_approach_max_boost_repeats: Some(0),
                    ..settings
                },
            )
            .unwrap();
            assert!(meets(optional.attempt.as_ref().unwrap()).0);
            let boost_only = plan_route_escape(
                &world,
                entry,
                Situation::Normal,
                5,
                &enemies,
                5,
                Settings {
                    escape_max_boost_repeats: None,
                    ..settings
                },
            )
            .unwrap();
            assert!(boost_only.break_action.is_none());
            assert_eq!(meets(boost_only.attempt.as_ref().unwrap()), (true, true));
        }
        assert!(
            accepted_break,
            "Fixture must cover a complete valid break plan"
        );
        assert!(
            rejected_break,
            "Fixture must reject a break that fails required route outcomes"
        );
    }

    #[test]
    fn flats_unboosted_break_target_replays_and_preserves_earlier_required_goals() {
        let world=highroad_world();
        let project=|a:&FieldEscapeAttempt| {
            let f=flats::after_highroad(&return_to_field(&world,a).unwrap()).unwrap();
            assert!(f.harbor_reach.walking.reaches_reset);
            let h=crate::harbor::after_flats(&f).unwrap().unwrap();
            crate::flats_escape::first_encounter(h.after,&h.world_after,5).unwrap()
        };
        for seed in 0..8 {
            let mut entry=CrtRng::new(0); entry.advance(seed*7919); entry.draws=6000;
            let settings=Settings {escape_max_boost_repeats:None,flats_max_boost_repeats:None,
                flats_approach_max_boost_repeats:None,flats_break_max_boost_repeats:None,..Default::default()};
            let plan=plan_route_escape(&world,entry,Situation::Normal,5,&["ENE_NML_FLD_CTY_010"],5,settings).unwrap();
            let replay=plan.menu.clone().attempt(&plan.inputs.as_ref().unwrap().levels,5).unwrap();
            assert_eq!(replay.after,plan.attempt.as_ref().unwrap().after);
            let (at,e)=project(&replay);
            assert!(crate::flats_escape::unboosted_opening_break(at,&e,5).unwrap());
            assert!(plan.flats_break_search.as_ref().unwrap().target_met);
            assert!(plan.flats_search.as_ref().unwrap().target_met);
            assert!(plan.flats_approach_search.as_ref().unwrap().target_met);
            let prior=plan_route_escape(&world,entry,Situation::Normal,5,&["ENE_NML_FLD_CTY_010"],5,
                Settings {flats_break_max_boost_repeats:Some(0),..settings}).unwrap();
            let optional=plan_route_escape(&world,entry,Situation::Normal,5,&["ENE_NML_FLD_CTY_010"],5,
                Settings {flats_break_max_boost_repeats:Some(4),..settings}).unwrap();
            let (_,e)=project(optional.attempt.as_ref().unwrap());
            assert_ne!(e.situation.as_deref(),Some("Surprise"));
            assert!(optional.inputs.as_ref().unwrap().extra_repeats<=prior.inputs.as_ref().unwrap().extra_repeats+4);
        }
    }

    #[test]
    fn escape_fallback_obeys_limit_then_break_then_unlimited_priority() {
        let mut covered = [false; 3];
        for seed in 0..64 {
            let entry = CrtRng {
                state: seed,
                draws: 6000,
            };
            for enemies in [
                vec!["ENE_NML_FLD_CTY_010"],
                vec!["ENE_NML_FLD_ISD_100"],
                vec!["ENE_NML_FLD_CTY_120", "ENE_NML_FLD_CTY_120"],
                vec!["ENE_NML_OTR_130"],
            ] {
                let p = escape_plan(entry, Situation::Normal, 5, &enemies, 5, Some(0)).unwrap();
                assert!(p.attempt.as_ref().unwrap().success);
                match p.method {
                    EscapeMethod::WithinLimit => {
                        covered[0] = true;
                        assert!(p.baseline.success);
                        assert!(p.inputs.as_ref().unwrap().levels.is_empty());
                        assert!(p.break_action.is_none());
                    }
                    EscapeMethod::BreakThenEscape => {
                        covered[1] = true;
                        assert!(!p.baseline.success);
                        assert_eq!(enemies.len(), 1);
                        assert_eq!(p.menu.rng, p.break_action.as_ref().unwrap().at_attack_menu);
                        assert_eq!(
                            p.attempt.as_ref().unwrap().after,
                            p.attempt.as_ref().unwrap().before_check
                        );
                        assert!(p.inputs.is_none());
                        // An unbounded initial search never chooses break instead.
                        assert_eq!(
                            escape_plan(entry, Situation::Normal, 5, &enemies, 5, None)
                                .unwrap()
                                .method,
                            EscapeMethod::WithinLimit
                        );
                    }
                    EscapeMethod::UnlimitedFallback => {
                        covered[2] = true;
                        assert!(!p.baseline.success);
                        assert!(p.inputs.as_ref().unwrap().extra_repeats > 0);
                        assert!(
                            first_action_break(entry, Situation::Normal, 5, &enemies)
                                .unwrap()
                                .is_none()
                        );
                        let direct =
                            escape_plan(entry, Situation::Normal, 5, &enemies, 5, None).unwrap();
                        assert_eq!(direct.inputs.unwrap().levels, p.inputs.unwrap().levels);
                        assert_eq!(direct.attempt.unwrap().after, p.attempt.unwrap().after);
                    }
                    EscapeMethod::Unavailable | EscapeMethod::FlatsAdjustment => {
                        panic!("Expected one of the three successful strategies")
                    }
                }
            }
        }
        assert_eq!(covered, [true; 3]);
    }
}
