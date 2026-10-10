//! Day switch -> Al introduction -> first land encounter. R8 manipulation and
//! first-menu Escape are separate searches with independently bounded inputs.
use crate::{
    CrtRng, Result, SimError,
    encounter::{self, Boundary, PredictionOptions},
    ensure, flats,
    manipulation::BoostInputs,
    pirro, pirro_aftermath, second,
    world::{self, Decision, Loader, Trace, WorldState},
};
use ot2_battle_engine::{
    Event, Outcome, Resource,
    escape::Situation,
    field_escape::{
        FieldBreakEscape, FieldEscapeAttempt, FieldEscapeMenu,
        level_for_experience, route_battle_base_exp,
    },
};
use serde::Serialize;
#[cfg(test)]
use ot2_battle_engine::field_escape::first_action_break;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Settings {
    pub approach_max_boost_repeats: Option<u32>,
    pub break_max_boost_repeats: Option<u32>,
    pub escape_max_boost_repeats: Option<u32>,
    pub flats_max_boost_repeats: Option<u32>,
    pub flats_approach_max_boost_repeats: Option<u32>,
    pub flats_break_max_boost_repeats: Option<u32>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            approach_max_boost_repeats: Some(4),
            break_max_boost_repeats: Some(0),
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
    pub break_max_boost_repeats: Option<u32>,
    pub unboosted_break_met: bool,
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
    pub escape_boost_inputs: Option<BoostInputs>,
    pub weighted_input_cost: u32,
    pub boost_repeat_count: u32,
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

pub(crate) fn apply_escape_resources(state:&mut WorldState,attempt:&FieldEscapeAttempt)->Result<()> {
    state.sync_progression()?;
    for (index,&id) in state.party.iter().enumerate() {
        let character=&mut state.character_progression.get_mut(&id).unwrap().character;
        let maximum=ot2_battle_engine::field_escape::player_latent_max(&character.id)?;
        let resource=character.resources.as_mut().unwrap();
        let cost=attempt.party_sp_cost.get(index).copied().unwrap_or(0);
        ensure(resource.sp>=cost,"Insufficient carried SP for selected break")?;
        resource.sp-=cost;
        resource.latent=resource.latent.saturating_add(attempt.party_latent_gain.get(index).copied().unwrap_or(0)).min(maximum);
    }
    Ok(())
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
    apply_escape_resources(&mut state,attempt)?;
    state.enemy_knowledge.extend(attempt.enemy_knowledge.clone());
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

/// Compare direct Escape and real opening break / next-menu boost plans.
/// Every strategy must satisfy the caller's post-battle target too.
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

/// A finite legacy `cap` enables breaking; None means Boost only. `total_limit` bounds an
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
    escape_plan_with_break_policy(entry,situation,level,enemies,danger,cap,total_limit,false,accepts)
}
pub(crate) fn escape_plan_with_break_policy(
    entry:CrtRng, situation:Situation, level:u32, enemies:&[&str], danger:u32,
    cap:Option<u32>, total_limit:Option<u32>, require_break:bool,
    accepts:impl Fn(&FieldEscapeAttempt)->Result<bool>,
)->Result<Escape> {
    let mut search=crate::escape_search::FieldSearch::new(entry,situation,level,enemies,danger)?;
    if require_break {search.require_break();}
    let mut tested=0;
    let maximum=total_limit.map(|n|n.saturating_mul(2).saturating_add(crate::escape_search::BREAK_INPUT_COST+crate::escape_search::ROUND_INPUT_COST+1));
    let mut cost=0u32;
    loop {
        if maximum.is_some_and(|m|cost>m) {let mut p=search.unavailable()?;p.tested_candidates=tested;return Ok(p);}
        for mut plan in search.candidates(cost,require_break || cap.is_some(),total_limit)? {
            tested+=1;
            if accepts(plan.attempt.as_ref().unwrap())? {
                plan.tested_candidates=tested;
                if plan.method!=EscapeMethod::BreakThenEscape && cap.is_some_and(|n|plan.boost_repeat_count>n) {
                    plan.method=EscapeMethod::UnlimitedFallback;
                }
                return Ok(plan);
            }
        }
        cost=cost.checked_add(1).ok_or_else(||SimError("Escape search exhausted representable cost".into()))?;
    }
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
            (e.situation.as_deref()!=Some("Surprise"), crate::flats_escape::opening_break(entry,&e,level)?)
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
    let choose = |limit:Option<u32>, accepts: &dyn Fn(&FieldEscapeAttempt) -> Result<bool>| {
        let mut search=crate::escape_search::FieldSearch::new(entry,situation,level,enemies,danger)?;
        if settings.break_max_boost_repeats.is_none() {search.require_break();}
        let maximum=limit.map(|n:u32|n.saturating_mul(2).saturating_add(crate::escape_search::BREAK_INPUT_COST+crate::escape_search::ROUND_INPUT_COST+1));
        let mut tested=0;
        for cost in 0..u32::MAX {
            if maximum.is_some_and(|m|cost>m) {return search.unavailable();}
            for mut plan in search.candidates(cost,true,limit)? {
                tested+=1;
                if accepts(plan.attempt.as_ref().unwrap())? {plan.tested_candidates=tested;return Ok(plan);}
            }
        }
        Err(SimError("Break search exhausted representable cost".into()))
    };
    let mut plan = choose(None, &required)?;
    let Some(a) = &plan.attempt else {
        return Ok(plan);
    };
    let (counter, walk, _, _) = project(a)?;
    let base = plan.boost_repeat_count;
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
    if !walk && let Some(extra) = settings.flats_max_boost_repeats && extra>0 {
        let candidate = choose(Some(base.saturating_add(extra)), &|a| {
            Ok(required(a)? && project(a)?.1)
        })?;
        walking.tested_candidates = candidate.tested_candidates;
        if candidate.attempt.is_some() {
            walking.target_met = true;
            walking.added_repeats = candidate.boost_repeat_count
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
    let prior = plan.boost_repeat_count;
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
    } else if !safe && let Some(extra) = settings.flats_approach_max_boost_repeats && extra>0 {
        let candidate = choose(Some(prior.saturating_add(extra)), &|a| {
            let (_, walk, safe, _) = project(a)?;
            Ok(walk && safe && required(a)?)
        })?;
        approach.tested_candidates = candidate.tested_candidates;
        if candidate.attempt.is_some() {
            approach.target_met = true;
            approach.added_repeats = candidate.boost_repeat_count
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
    let prior=plan.boost_repeat_count;
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
            break_search.added_repeats=candidate.boost_repeat_count.saturating_sub(prior);
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
    pub formation_search: Option<crate::formation_search::Report>,
    pub verification: &'static str,
}

pub(crate) fn reward(base: u32, party: usize, steps: &[second::Step]) -> u32 {
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
    let (before,after)=route_experience(second,boss)?;
    let level = level_for_experience(after);
    ensure(level > level_for_experience(before),"Highroad healthy-HP profile requires the post-Pirro level-up")?;
    Ok(level)
}
pub(crate) fn route_experience(second:&second::Prediction,boss:&pirro::Prediction)->Result<(u32,u32)> {
    let before = second.opening_exp
        + reward(
            route_battle_base_exp("ENG_EVE_THI_C01_020")?,
            3,
            &second.steps,
        );
    let after = before + reward(route_battle_base_exp(boss.group)?, 1, &boss.steps);
    Ok((before,after))
}
pub(crate) fn carry_progression(world:&mut WorldState,second:&second::Prediction,boss:&pirro::Prediction)->Result<()> {
    world.party_experience=vec![route_experience(second,boss)?.1];
    world.sync_progression()?;
    // Opening and second scripted fights finish on round two; Pirro takes
    // multiple rounds. None receives the one-turn JP bonus.
    let mut jp=0;
    for group in ["ENG_EVE_THI_C01_010",second.group,boss.group] {
        jp+=ot2_battle_engine::progression::formation_reward(group)?.jp;
    }
    world.character_progression.get_mut(&7).unwrap().jp_earned=jp;
    world.character_progression.get_mut(&7).unwrap().character=ot2_battle_engine::field_escape::route_player(7,world.party_experience[0])?;
    world.character_progression.get_mut(&7).unwrap().character.resources.as_mut().unwrap().latent=boss.state.actors[0].resources.latent;
    Ok(())
}

pub fn predict(
    ending: &pirro_aftermath::Prediction,
    level: u32,
    settings: Settings,
) -> Result<Prediction> {
    predict_internal(ending, level, settings, true)
}
pub(crate) fn predict_internal(
    ending: &pirro_aftermath::Prediction,
    level: u32,
    settings: Settings,
    plan_escape: bool,
) -> Result<Prediction> {
    ensure(
        settings.approach_max_boost_repeats.is_none_or(|n| n <= 256)
            && settings.break_max_boost_repeats.is_none_or(|n| n <= 256)
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
        progress: loader.state.main_progress_points,
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
        formation_search: None,
        verification: "Live capture verifies all 10 CRT calls from menu6002 through five boost cycles, Escape, two field gondola callbacks and counter B/A to6012. Other starts remain source-derived. Immediate daytime switch, skipped Al introduction, unchanged equipment, no intervening CRT actions; first land encounter. Compare direct Escape with actual break and next-menu boosts by estimated input cost: boost input1, break6 plus8 per elapsed round. Break uses actual hit rolls and guarantees next-menu Escape without a lottery. Surprise combat and failed-Escape continuation are not approximated.",
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
    from_battle_context(boss,Some(second),settings,replay)
}
pub fn from_identified_battle(boss:&mut pirro::Prediction,settings:Settings)
    ->Result<Option<(pirro_aftermath::Prediction,Prediction)>> {
    from_battle_context(boss,None,settings,None)
}
fn from_battle_context(boss:&mut pirro::Prediction,second:Option<&second::Prediction>,settings:Settings,replay:Option<Vec<u8>>)
    ->Result<Option<(pirro_aftermath::Prediction,Prediction)>> {
    let level=|p:&pirro::Prediction| ->Result<u32> {
        if let Some(second)=second {return route_level(second,p);}
        Ok(level_for_experience(p.world_at_entry.party_experience[0]+reward(route_battle_base_exp(p.group)?,1,&p.steps)))
    };
    let carry=|world:&mut WorldState,p:&pirro::Prediction| ->Result<()> {
        if let Some(second)=second {return carry_progression(world,second,p);}
        world.party_experience=vec![p.world_at_entry.party_experience[0]+reward(route_battle_base_exp(p.group)?,1,&p.steps)];
        world.sync_progression()?;
        let row=world.character_progression.get_mut(&7).ok_or_else(||SimError("Missing Throné progression".into()))?;
        row.jp_earned=["ENG_EVE_THI_C01_010","ENG_EVE_THI_C01_020",p.group].iter()
            .map(|g|ot2_battle_engine::progression::formation_reward(g).map(|r|r.jp)).collect::<Result<Vec<_>>>()?.iter().sum();
        row.character=ot2_battle_engine::field_escape::route_player(7,world.party_experience[0])?;
        row.character.resources.as_mut().unwrap().latent=p.state.actors[0].resources.latent;
        Ok(())
    };
    let Some(mut ending) = pirro_aftermath::after_battle(boss)? else {
        return Ok(None);
    };
    carry(&mut ending.world_after,boss)?;
    let mut prediction=predict_internal(&ending,level(boss)?,settings,false)?;
    let mut ending=ending;
    let facts=|p:&Prediction| -> Result<[bool;2]> {
        Ok([p.encounter.situation.as_deref()!=Some("Surprise"),
            crate::flats_escape::opening_break(p.before_battle_initialization,&p.encounter,p.level)?])
    };
    let caps=[settings.approach_max_boost_repeats,settings.break_max_boost_repeats];
    let mut goals=caps.map(|c|c.is_none());
    let mut info=ApproachSearch {max_boost_repeats:caps[0],break_max_boost_repeats:caps[1],
        unboosted_break_met:facts(&prediction)?[1],tested_candidates:0,target_met:facts(&prediction)?[0],
        plan:vec![1,2,3],status:"Highroad opening targets evaluated"};
    let no_grape=boss.routine.heal_before_round_eight==Some(false);
    let need_search=facts(&prediction)?.iter().zip(goals).any(|(met,needed)|needed && !met)
        || facts(&prediction)?.iter().zip(caps).any(|(met,cap)|!met && cap.is_some_and(|n|n>0));
    if (need_search || replay.is_some()) && no_grape {
        let r3=boss.boost_search.as_ref().map(|s|s.plan.clone())
            .or_else(||boss.manipulation.as_ref().map(|m|m.inputs.levels.clone())).unwrap_or_else(||if boss.identified_checkpoint.is_some(){vec![]}else{vec![1,2,3]});
        let menu=pirro::r8_menu(boss,&r3)?;
        let mut world=boss.world_at_entry.clone();world.set_flag(8017,false);
        let inventory=pirro::replay_inventory(boss)?;
        let run=|path:&[u8]| pirro::predict_from_checkpoint(boss.arrival,&world,
            boss.initial_state.actors[0].resources.clone(),inventory.clone(),&r3,path,boss.identified_checkpoint.as_ref());
        let check=|p:&pirro::Prediction,needed:[bool;2]| -> Result<bool> {
            if p.state.outcome!=Outcome::Victory || p.routine_error.is_some() || p.routine.heal_before_round_eight!=Some(false) {return Ok(false);}
            let after=pirro_aftermath::after_battle(p)?.unwrap();
            let projected=predict_internal(&after,level(p)?,settings,false)?;
            Ok(facts(&projected)?.iter().zip(needed).all(|(met,need)|!need || *met))
        };
        let mut selected=None;
        if let Some(path)=replay {
            if let (Some(a),Some(b))=(caps[0],caps[1]) {
                ensure(pirro::describe_path(&path)?.extra_repeats<=a.saturating_add(b),"Pirro R8 replay exceeds its outcome budgets")?;
            }
            let p=run(&path)?;
            ensure(check(&p,goals)?,"Pirro R8 replay violates required opening targets")?;
            selected=Some((p,path));
        } else {
            let current=facts(&prediction)?;
            if current.iter().zip(goals).any(|(met,need)|need && !met) {
                let (p,tested)=pirro::search_paths(&menu,menu.can_use_latent(),None,&run,&|p|check(p,goals))?;
                selected=p;info.tested_candidates+=tested;
            }
            for i in 0..2 {
                let met=if let Some((p,_))=&selected {
                    let after=pirro_aftermath::after_battle(p)?.unwrap();
                    facts(&predict_internal(&after,level(p)?,settings,false)?)?
                } else {facts(&prediction)?};
                for j in 0..2 {goals[j]|=met[j];}
                if !met[i] && let Some(extra)=caps[i] && extra>0 {
                    let base=selected.as_ref().map_or(0,|(_,path)|pirro::describe_path(path).unwrap().extra_repeats);
                    let mut needed=goals;needed[i]=true;
                    let (p,tested)=pirro::search_paths(&menu,menu.can_use_latent(),Some(base.saturating_add(extra)),&run,&|p|check(p,needed))?;
                    info.tested_candidates+=tested;
                    if p.is_some(){selected=p;goals[i]=true;}
                }
            }
        }
        if let Some((mut p,path))=selected {
            ending=pirro_aftermath::after_battle(&p)?.unwrap();
            prediction=predict_internal(&ending,level(&p)?,settings,false)?;
            p.boost_search=boss.boost_search.take();*boss=p;info.plan=path;
        }
    } else if let Some(path)=replay {ensure(path==[1,2,3],"R8 replay unavailable in healing branch")?;}
    let met=facts(&prediction)?;info.target_met=met[0];info.unboosted_break_met=met[1];
    if !met[0] || (caps[1].is_none() && !met[1]) {info.status="R8 opening target not met";}
    carry(&mut ending.world_after,boss)?;
    ensure(met[0] && (caps[1].is_some() || met[1]),"Pirro R8 did not secure a safe Throné-breakable Highroad encounter")?;
    prediction=predict(&ending,level(boss)?,settings)?;
    prediction.approach_search=Some(info);
    Ok(Some((ending,prediction)))
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
            if baseline_valid {assert_eq!(plan.weighted_input_cost,0);}
            if plan.method==EscapeMethod::BreakThenEscape {accepted_break=true;}
            rejected_break |= break_plan.is_some() && !break_valid;
            let mut exhaustive=crate::escape_search::FieldSearch::new(entry,Situation::Normal,5,&enemies,5).unwrap();
            for cost in 0..plan.weighted_input_cost {
                for trial in exhaustive.candidates(cost,true,None).unwrap() {
                    assert_ne!(meets(trial.attempt.as_ref().unwrap()),(true,true),"Rejected faster candidate must fail an actual route goal");
                }
            }            let both_required = plan_route_escape(
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
            let unlimited_escape = plan_route_escape(
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
            // An unlimited boost budget does not exclude a cheaper optional break.
            assert!(unlimited_escape.attempt.as_ref().unwrap().success);
            let required_break_plan=plan_route_escape(&world,entry,Situation::Normal,5,&enemies,5,Settings {break_max_boost_repeats:None,escape_max_boost_repeats:None,..settings}).unwrap();
            assert!(required_break_plan.break_action.is_some());
            assert_eq!(meets(required_break_plan.attempt.as_ref().unwrap()),(true,true));
            assert_eq!(meets(unlimited_escape.attempt.as_ref().unwrap()), (true, true));
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
            let mut replay_search=crate::escape_search::FieldSearch::new(entry,Situation::Normal,5,&["ENE_NML_FLD_CTY_010"],5).unwrap();
            let replay=replay_search.candidates(plan.weighted_input_cost,true,None).unwrap().into_iter().find(|c|c.inputs==plan.inputs && serde_json::to_value(&c.break_action).unwrap()==serde_json::to_value(&plan.break_action).unwrap()).unwrap().attempt.unwrap();
            assert_eq!(replay.after,plan.attempt.as_ref().unwrap().after);
            let (at,e)=project(&replay);
            assert!(crate::flats_escape::opening_break(at,&e,5).unwrap());
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
    fn escape_strategies_choose_minimum_input_equivalent_cost() {
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
                        assert!(p.weighted_input_cost>=crate::escape_search::BREAK_INPUT_COST);
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

impl Escape {
    pub fn enemy_knowledge(&self) -> ot2_battle_engine::knowledge::EnemyHistory {
        self.break_action.as_ref().map(|b|b.enemy_knowledge.clone())
            .unwrap_or_else(||self.menu.enemy_knowledge.clone())
    }
}
