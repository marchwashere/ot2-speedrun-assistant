//! The 1800 rich-event battle handoff. World progression stays in this crate;
//! the imported battle core owns all combat and the fixed player policy.
use crate::manipulation::{BoostInputs, describe_inputs};
use crate::{CrtRng, Result, ensure, mansion_escape, second, world::WorldState};
use ot2_battle_engine::{
    ActorId, Battle, BattleConfig, BattleState, ItemCatalog, Outcome, Resources, Side,
    opening_crt::RngSpan, pirro::PirroRoutine,
};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Settings {
    pub victory_max_boost_repeats: Option<u32>,
    /// Additional repeats beyond the actual cost of the cheapest winning plan.
    pub no_grape_max_boost_repeats: Option<u32>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            victory_max_boost_repeats: Some(4),
            no_grape_max_boost_repeats: Some(4),
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Search {
    pub settings: Settings,
    pub baseline_outcome: Outcome,
    pub baseline_heal: Option<bool>,
    pub target_met: bool,
    pub survival_met: bool,
    pub limit_reached: bool,
    pub tested_candidates: u64,
    pub victory_tested_candidates: u64,
    pub no_grape_tested_candidates: u64,
    pub victory_plan: Vec<u8>,
    pub victory_repeats: u32,
    pub no_grape_total_repeat_cap: Option<u32>,
    pub victory_limit_reached: bool,
    pub no_grape_limit_reached: bool,
    /// Exact BP after each button press; always ends at 3 BP.
    pub plan: Vec<u8>,
    pub status: &'static str,
}
#[derive(Debug, Serialize)]
pub struct Manipulation {
    pub inputs: BoostInputs,
    pub extra_advances: u64,
    pub actor: ActorId,
    pub round: u32,
    pub round_menu: u32,
    pub rng_before: CrtRng,
    pub rng_after_inputs: CrtRng,
}

#[derive(Debug, Serialize)]
pub struct Prediction {
    #[serde(skip)]
    pub identified_checkpoint: Option<Battle>,
    pub group: &'static str,
    pub arrival: CrtRng,
    pub entry: CrtRng,
    /// Last simulated boundary; only a victory has an after_victory value.
    pub after: CrtRng,
    pub after_victory: Option<CrtRng>,
    pub actors: Vec<second::Actor>,
    pub action_names: BTreeMap<String, String>,
    pub initial_state: BattleState,
    pub state: BattleState,
    pub steps: Vec<second::Step>,
    pub routine: PirroRoutine,
    pub routine_error: Option<String>,
    pub manipulation: Option<Manipulation>,
    pub r8_manipulation: Option<Manipulation>,
    pub r8_plan: Vec<u8>,
    pub boost_search: Option<Search>,
    pub transition_trace: Vec<RngSpan>,
    /// Event is suspended in battle; no 1800 cleanup / 1900 aftermath yet.
    pub world_at_entry: WorldState,
    pub verification: &'static str,
}

pub fn from_route(
    second: &second::Prediction,
    escape: &mansion_escape::Prediction,
    settings: Settings,
    replay: Option<Vec<u8>>,
) -> Result<Option<Prediction>> {
    let Some(solution) = &escape.solution else {
        return Ok(None);
    };
    if !solution.reaches_pirro { return Ok(None); }
    let mansion = second
        .mansion
        .as_ref()
        .ok_or_else(|| crate::SimError("Mansion stage missing".into()))?;
    let catalog = ItemCatalog::builtin()?;
    ensure(
        catalog.items.len() == second.state.inventory.len(),
        "Route inventory does not match item catalog",
    )?;
    let mut inventory: BTreeMap<String, u32> = catalog
        .items
        .iter()
        .zip(&second.state.inventory)
        .map(|(i, n)| (i.id.clone(), *n))
        .collect();
    for (id, n) in &mansion.acquired_items {
        // Keep sale valuables in the route inventory; the battle catalog holds
        // usable combat items only.
        if matches!(id.as_str(),"gold_ring"|"antique_coin") {continue;}
        *inventory.entry(id.clone()).or_default() += n;
    }
    predict_with_search(
        solution.after_field_return,
        &solution.world_at_trigger,
        solution.attempt.state.actors[0].resources.clone(),
        inventory,
        settings,
        replay,
    )
    .map(Some)
}

pub fn predict(
    arrival: CrtRng,
    world: &WorldState,
    resources: Resources,
    inventory: BTreeMap<String, u32>,
) -> Result<Prediction> {
    predict_with_plan(arrival, world, resources, inventory, &[1, 2, 3])
}

pub fn predict_with_plan(
    arrival: CrtRng,
    world: &WorldState,
    resources: Resources,
    inventory: BTreeMap<String, u32>,
    levels: &[u8],
) -> Result<Prediction> {
    predict_with_plans(arrival, world, resources, inventory, levels, &[1, 2, 3])
}

pub fn predict_with_plans(
    arrival: CrtRng,
    world: &WorldState,
    resources: Resources,
    inventory: BTreeMap<String, u32>,
    levels: &[u8],
    r8_levels: &[u8],
) -> Result<Prediction> {
    predict_from_checkpoint(arrival,world,resources,inventory,levels,r8_levels,None)
}

pub(crate) fn predict_from_checkpoint(
    arrival:CrtRng, world:&WorldState, mut resources:Resources,
    inventory:BTreeMap<String,u32>, levels:&[u8], r8_levels:&[u8], checkpoint:Option<&Battle>,
)->Result<Prediction> {
    let _profile=crate::profile::span("Pirro replay");
    ensure(
        levels.last() == Some(&3) || (checkpoint.is_some() && levels.is_empty()),
        "Pirro R3 boost path must end at 3 BP",
    )?;
    ensure(
        world.map == "Dng_Cty_1_1"
            && world.time == 4
            && world.party == [7]
            && world.flag(8016)
            && !world.flag(8017)
            && !world.flag(652),
        "Pirro entry requires solo Throné after the lookout, before 1800",
    )?;
    ensure(
        resources.hp > 0,
        "Throné must survive the mansion encounter",
    )?;
    // MS_TOU_10_1800 cmd71 sets 8017. RI_MS_TOU_10_1500 starts the boss
    // inside cmd1500. Native StartEventBattle suspends the parent script;
    // EncountReset=true is NOT executed here. Same map, no field reload.
    let entry = arrival;
    let mut world_at_entry = world.clone();
    world_at_entry.set_flag(8017, true);
    let mut config = BattleConfig::throne_pirro(entry.state)?;
    resources.bp = 1;
    config.players[0].resources = Some(resources);
    config.inventory = inventory;
    let catalog = ItemCatalog::builtin()?;
    let action_names = config
        .skills
        .iter()
        .map(|s| (s.id.clone(), s.name.clone()))
        .chain(catalog.items.iter().map(|i| (i.id.clone(), i.name.clone())))
        .collect();
    let actors = config
        .players
        .iter()
        .map(|p| second::Actor {
            name: p.name.clone(),
            side: Side::Player,
            max_hp: p.stats.max_hp,
            max_sp: p.stats.max_sp,
        })
        .chain(config.enemies.iter().map(|e| second::Actor {
            name: e.name.clone(),
            side: Side::Enemy,
            max_hp: e.stats.max_hp,
            max_sp: e.stats.max_sp,
        }))
        .collect();
    let mut battle = if let Some(b)=checkpoint {b.clone()} else {Battle::new_pirro(config, catalog, true)?};
    let initial_state = battle.state().clone();
    let mut routine = PirroRoutine::default();
    let mut steps = Vec::new();
    let mut routine_error = None;
    let mut manipulation = None;
    let mut r8_manipulation = None;
    while battle.state().outcome == Outcome::Ongoing {
        ensure(steps.len() < 48, "Pirro action budget exhausted")?;
        let round = battle.state().round;
        let report =
            if round == 3 && battle.current_actor() == Some(ActorId(0)) && checkpoint.is_some() {
                let before=battle.state().rng;
                let mut selected=3;
                for &next in levels {battle.advance_boost_input(selected,next)?;selected=next;}
                ensure(selected==3,"Identification adjustment must finish at 3 BP")?;
                let after_inputs=battle.state().rng;
                let mut report=battle.pirro_identified_darkest_night()?;
                report.rng_before=before;
                if !levels.is_empty() {
                    manipulation=Some(Manipulation {inputs:BoostInputs {
                        levels:levels.to_vec(),normal_bp:3,input_count:levels.len(),extra_input_count:levels.len(),
                        extra_repeats:(levels.len()/2) as u32,setup:vec![],extras:vec![],resume:vec![],
                    },extra_advances:after_inputs.draws-before.draws,actor:ActorId(0),round,round_menu:1,
                        rng_before:before,rng_after_inputs:after_inputs});
                }
                Ok(report)
            } else if round == 3 && battle.current_actor() == Some(ActorId(0)) && levels != [1, 2, 3] {
                let rng_before = battle.state().rng;
                let rng_after_inputs = battle.preview_boost_inputs(levels)?.rng;
                let normal_after_inputs = battle.preview_boost_inputs(&[1, 2, 3])?.rng;
                let inputs = describe_path(levels)?;
                let report = routine.step_with_boost_inputs(&mut battle, &inputs.levels);
                if report.is_ok() {
                    manipulation = Some(Manipulation {
                        inputs,
                        extra_advances: rng_after_inputs.draws - normal_after_inputs.draws,
                        actor: ActorId(0),
                        round,
                        round_menu: 1,
                        rng_before,
                        rng_after_inputs,
                    });
                }
                report
            } else if round == 8
                && battle.current_actor() == Some(ActorId(0))
                && r8_manipulation.is_none()
                && r8_levels != [1, 2, 3]
            {
                ensure(
                    routine.heal_before_round_eight == Some(false),
                    "R8 swords require the no-grape branch",
                )?;
                let rng_before = battle.state().rng;
                let latent = battle.can_use_latent();
                let preview = |path: &[u8]| {
                    if latent {
                        battle.preview_boost_inputs_after_latent(path)
                    } else {
                        battle.preview_boost_inputs(path)
                    }
                };
                let rng_after_inputs = preview(r8_levels)?.rng;
                let normal = preview(&[1, 2, 3])?.rng;
                let inputs = describe_path(r8_levels)?;
                let result = routine.step_with_boost_inputs(&mut battle, r8_levels);
                if result.is_ok() {
                    r8_manipulation = Some(Manipulation {
                        inputs,
                        extra_advances: rng_after_inputs.draws - normal.draws,
                        actor: ActorId(0),
                        round,
                        round_menu: 1,
                        rng_before,
                        rng_after_inputs,
                    });
                }
                result
            } else {
                routine.step(&mut battle)
            };
        match report {
            Ok(report) => steps.push(second::Step {
                round,
                report,
                actors_after: battle.state().actors.clone(),
            }),
            Err(e) => {
                routine_error = Some(e.to_string());
                break;
            }
        }
    }
    ensure(
        checkpoint.is_some() || levels == [1, 2, 3] || manipulation.is_some(),
        "Pirro R3 boost menu was not reached",
    )?;
    ensure(
        r8_levels == [1, 2, 3] || r8_manipulation.is_some(),
        "Pirro R8 boost menu was not reached",
    )?;
    let mut after = entry;
    after.advance(battle.state().rng.draws);
    ensure(
        after.state == battle.state().rng.state,
        "Pirro CRT continuity failed",
    )?;
    Ok(Prediction {
        identified_checkpoint:checkpoint.cloned(),
        group: "ENG_BOS_THI_C01_010",
        arrival,
        entry,
        after,
        after_victory: (battle.state().outcome == Outcome::Victory).then_some(after),
        actors,
        action_names,
        initial_state,
        state: battle.state().clone(),
        steps,
        routine,
        routine_error,
        manipulation,
        r8_manipulation,
        r8_plan: r8_levels.to_vec(),
        boost_search: None,
        transition_trace: vec![RngSpan {
            reason: if checkpoint.is_some() {"pirro_r3_identified_checkpoint"} else {"pirro_cutscene_battle_handoff"},
            actor: None,
            before: arrival,
            after: entry,
        }],
        world_at_entry,
        verification: if checkpoint.is_some() {"Begins immediately after the last recorded identification boost, with 3 BP selected. R1/R2 battle state is reconstructed from the voice transcript and R3 turn order, starting at full HP/SP with the standard equipment. CRT draw count restarts at zero; no early manipulation or battle initialization is replayed after this boundary. Retail validation of this entry mode is pending."} else {"The seed-0 +5678 loss matches all 203 captured CRT calls through 5881. The 2026-09-29 entry5595 victory matches all 305 captured calls through 5900, including the death-slow timer. Other seeds remain source-derived. Skipped 1800 enters battle directly (+0 pre-initialization draws). Carries Escape HP/SP and the route's stolen soulstones. R7 chooses the healing branch from the R8 preview. Latent unlock grants 55/80; when unavailable on no-grape R8, swords stay on R8 and R9 latent pairs Wind with Ice. Victory boundary includes drops, before 1900 and field return. The supplied routine is simulated as written and can lose on some damage/critical/order rolls."},
    })
}

fn won(p: &Prediction) -> bool {
    p.state.outcome == Outcome::Victory && p.routine_error.is_none()
}

pub(crate) fn replay_inventory(p: &Prediction) -> Result<BTreeMap<String, u32>> {
    Ok(ItemCatalog::builtin()?
        .items
        .iter()
        .zip(&p.initial_state.inventory)
        .map(|(item, &count)| (item.id.clone(), count))
        .collect())
}

pub(crate) fn r8_menu(p: &Prediction, r3: &[u8]) -> Result<Battle> {
    let mut config = BattleConfig::throne_pirro(p.entry.state)?;
    config.players[0].resources = Some(p.initial_state.actors[0].resources.clone());
    config.inventory = replay_inventory(p)?;
    let mut battle = if let Some(b)=&p.identified_checkpoint {b.clone()} else {Battle::new_pirro(config, ItemCatalog::builtin()?, false)?};
    let mut routine = PirroRoutine::default();
    while battle.state().outcome == Outcome::Ongoing {
        if battle.state().round == 8 && battle.current_actor() == Some(ActorId(0)) {
            ensure(
                routine.heal_before_round_eight == Some(false),
                "R8 swords require the no-grape branch",
            )?;
            return Ok(battle);
        }
        if battle.state().round == 3 && battle.current_actor() == Some(ActorId(0)) {
            if p.identified_checkpoint.is_some() {
                let mut bp=3;for &next in r3 {battle.advance_boost_input(bp,next)?;bp=next;}
                ensure(bp==3,"R3 replay must finish at 3 BP")?;battle.pirro_identified_darkest_night()?;
            } else {routine.step_with_boost_inputs(&mut battle, r3)?;}
        } else {
            routine.step(&mut battle)?;
        }
    }
    Err(crate::SimError("Pirro R8 menu not reached".into()))
}
fn target_met(p: &Prediction) -> bool {
    won(p) && p.routine.heal_before_round_eight == Some(false)
}
pub(crate) fn describe_path(levels: &[u8]) -> Result<BoostInputs> {
    // This helper describes climbs/button counts only. Actual CRT consumption
    // comes from the core: Pirro's special 50% max-boost voice adds a conditional draw.
    let mut previous = 0;
    let mut nominal = 0;
    for &bp in levels {
        ensure(
            bp <= 3 && bp.abs_diff(previous) == 1,
            "Invalid Pirro boost input",
        )?;
        if bp > previous {
            nominal += u32::from(bp);
        }
        previous = bp;
    }
    ensure(previous == 3, "Pirro boost path must end at 3 BP")?;
    describe_inputs(nominal - 6, 3, levels.to_vec())
}

/// Find the cheapest path for one target, with conditional voice draws intact.
/// The ordinary path has already been evaluated by the caller and failed.
pub(crate) fn search_paths(
    menu: &Battle,
    latent: bool,
    cap: Option<u32>,
    run: &impl Fn(&[u8]) -> Result<Prediction>,
    accepts: &impl Fn(&Prediction) -> Result<bool>,
) -> Result<(Option<(Prediction, Vec<u8>)>, u64)> {
    let preview_inputs = |path: &[u8]| {
        if latent {
            menu.preview_boost_inputs_after_latent(path)
        } else {
            menu.preview_boost_inputs(path)
        }
    };
    let normal = preview_inputs(&[1, 2, 3])?;
    let mut best = None;
    let mut best_cost = (usize::MAX, u64::MAX);
    let mut count = 0;
    // CRT + used voices identify equivalent futures. Selected BP, direction,
    // and cancel counts preserve the two-press repeat budget. BFS minimizes presses, then draws.
    let mut queue = VecDeque::from([(Vec::<u8>::new(), 0u8, false, 0u32)]);
    let mut visited = HashSet::new();
    let mut tested = HashSet::from([(
        normal.rng.state,
        normal.opening_crt.as_ref().unwrap().used_voices.clone(),
    )]);
    while let Some((levels, bp, _rising, climbs)) = queue.pop_front() {
        if levels.len() >= best_cost.0 {
            continue;
        }
        for next in [bp.checked_add(1).filter(|&n| n <= 3), bp.checked_sub(1)]
            .into_iter()
            .flatten()
        {
            let up = next > bp;
            let next_climbs = climbs + u32::from(!up);
            if cap.is_some_and(|cap| next_climbs > cap) {
                continue;
            }
            let mut path = levels.clone();
            path.push(next);
            let preview = preview_inputs(&path)?;
            let voices = preview.opening_crt.as_ref().unwrap().used_voices.clone();
            let budget = if cap.is_some() { next_climbs } else { 0 };
            if !visited.insert((preview.rng.state, voices.clone(), next, up, budget)) {
                continue;
            }
            if next == 3 && tested.insert((preview.rng.state, voices)) {
                let cost = (path.len(), preview.rng.draws);
                count += 1;
                let candidate = run(&path)?;
                if accepts(&candidate)? && cost < best_cost {
                    best_cost = cost;
                    best = Some((candidate, path.clone()));
                }
            }
            if path.len() < best_cost.0 {
                queue.push_back((path, next, up, next_climbs));
            }
        }
    }
    Ok((best, count))
}

/// Visit distinct R8 futures in button-cost order. The caller tightens the bound
/// whenever a cheaper COMPLETE continuation is found; unlike search_paths, an
/// inexpensive immediate encounter does not stop downstream cost comparisons.
pub(crate) fn visit_route_paths(
    menu:&Battle, latent:bool, bound:&std::cell::Cell<u32>,
    visit:&mut impl FnMut(&[u8])->Result<()>,
) -> Result<()> {
    let mut queue=VecDeque::from([(Vec::<u8>::new(),0u8)]);
    let mut visited=HashSet::new();
    let mut tested=HashSet::new();
    while let Some((levels,bp))=queue.pop_front() {
        for next in [bp.checked_add(1).filter(|&n|n<=3),bp.checked_sub(1)].into_iter().flatten() {
            let mut path=levels.clone();path.push(next);
            // Every continuation includes the mandatory night/day shop pair (24).
            if path.len() as u32>bound.get().saturating_sub(24)+3 {continue;}
            let p=if latent {menu.preview_boost_inputs_after_latent(&path)?} else {menu.preview_boost_inputs(&path)?};
            let voices=p.opening_crt.as_ref().unwrap().used_voices.clone();
            if !visited.insert((p.rng.state,voices.clone(),next)) {continue;}
            if next==3 && tested.insert((p.rng.state,voices)) {visit(&path)?;}
            queue.push_back((path,next));
        }
    }
    Ok(())
}

/// Freeze the prior route. Search only R3 boost inputs; replay every later
/// action, including the actual R8 preview branch and victory/drop callbacks.
pub fn predict_with_search(
    arrival: CrtRng,
    world: &WorldState,
    resources: Resources,
    inventory: BTreeMap<String, u32>,
    settings: Settings,
    replay: Option<Vec<u8>>,
) -> Result<Prediction> {
    ensure(
        settings.victory_max_boost_repeats.is_none_or(|n| n <= 256)
            && settings.no_grape_max_boost_repeats.is_none_or(|n| n <= 256),
        "Each Pirro cap must be 0..256 or unlimited",
    )?;
    let run =
        |plan: &[u8]| predict_with_plan(arrival, world, resources.clone(), inventory.clone(), plan);
    let mut best = run(&[1, 2, 3])?;
    let mut info = Search {
        settings,
        baseline_outcome: best.state.outcome,
        baseline_heal: best.routine.heal_before_round_eight,
        target_met: target_met(&best),
        survival_met: won(&best),
        limit_reached: false,
        tested_candidates: 0,
        victory_tested_candidates: 0,
        no_grape_tested_candidates: 0,
        victory_plan: vec![1, 2, 3],
        victory_repeats: 0,
        no_grape_total_repeat_cap: settings.no_grape_max_boost_repeats,
        victory_limit_reached: false,
        no_grape_limit_reached: false,
        plan: vec![1, 2, 3],
        status: "Victory without Healing Grape",
    };
    let menu_reached = best
        .steps
        .iter()
        .any(|s| s.round == 3 && s.report.actor == ActorId(0));
    // A missing grape in the baseline's optional healing branch does not
    // invalidate the already-reached R3 menu: its boosts can avoid that branch.
    // Failed commands roll the policy back, so inspect the R8 preview directly.
    let missing_baseline_grape = best.state.round == 7
        && best.state.next_queue.first() == Some(&ActorId(1))
        && inventory.get("healing_grape").copied().unwrap_or(0) == 0
        && best.routine_error.as_deref() == Some("item is out of stock");
    let can_search = menu_reached && (best.routine_error.is_none() || missing_baseline_grape);
    let menu = if !info.target_met && can_search {
        let mut config = BattleConfig::throne_pirro(arrival.state)?;
        let mut starting_resources = resources.clone();
        starting_resources.bp = 1;
        config.players[0].resources = Some(starting_resources);
        config.inventory = inventory.clone();
        let mut menu = Battle::new_pirro(config, ItemCatalog::builtin()?, false)?;
        let mut policy = PirroRoutine::default();
        while menu.state().round != 3 || menu.current_actor() != Some(ActorId(0)) {
            policy.step(&mut menu)?;
        }
        Some(menu)
    } else {
        None
    };
    // A required no-grape outcome includes victory. Search both together before
    // optional budgets; do not let a finite victory allowance veto it.
    let require_no_grape = settings.no_grape_max_boost_repeats.is_none();
    if require_no_grape
        && !target_met(&best)
        && let Some(menu) = &menu
    {
        let (solution, count) = search_paths(menu, false, None, &run, &|p| Ok(target_met(p)))?;
        info.no_grape_tested_candidates = count;
        if let Some((prediction, plan)) = solution {
            best = prediction;
            info.plan = plan;
        }
    }
    if !require_no_grape
        && !won(&best)
        && let Some(menu) = &menu
    {
        let (solution, count) = search_paths(
            menu,
            false,
            settings.victory_max_boost_repeats,
            &run,
            &|p| Ok(won(p)),
        )?;
        info.victory_tested_candidates = count;
        if let Some((prediction, plan)) = solution {
            best = prediction;
            info.plan = plan;
        }
    }
    info.victory_plan = info.plan.clone();
    info.victory_repeats = describe_path(&info.victory_plan)?.extra_repeats;
    info.victory_limit_reached =
        !won(&best) && can_search && settings.victory_max_boost_repeats.is_some();
    info.no_grape_total_repeat_cap = settings
        .no_grape_max_boost_repeats
        .map(|extra| info.victory_repeats + extra);
    if let Some(plan) = &replay {
        if won(&best) {
            let inputs = describe_path(plan)?;
            ensure(
                info.no_grape_total_repeat_cap
                    .is_none_or(|cap| inputs.extra_repeats <= cap),
                "Pirro replay exceeds event budgets",
            )?;
            let candidate = run(plan)?;
            ensure(won(&candidate), "Pirro replay must preserve victory")?;
            ensure(
                !require_no_grape || target_met(&candidate),
                "Pirro replay violates required no-grape outcome",
            )?;
            best = candidate;
            info.plan = plan.clone();
        } else {
            ensure(
                plan == &info.victory_plan,
                "Pirro replay cannot bypass the victory budget",
            )?;
        }
    } else if !require_no_grape
        && won(&best)
        && !target_met(&best)
        && let Some(menu) = &menu
    {
        let (solution, count) =
            search_paths(menu, false, info.no_grape_total_repeat_cap, &run, &|p| {
                Ok(target_met(p))
            })?;
        info.no_grape_tested_candidates = count;
        if let Some((prediction, plan)) = solution {
            best = prediction;
            info.plan = plan;
        }
    }
    info.target_met = target_met(&best);
    info.survival_met = won(&best);
    info.tested_candidates = info.victory_tested_candidates + info.no_grape_tested_candidates;
    info.no_grape_limit_reached = info.survival_met
        && !info.target_met
        && can_search
        && settings.no_grape_max_boost_repeats.is_some()
        && replay.is_none();
    info.limit_reached = info.victory_limit_reached || info.no_grape_limit_reached;
    info.status = if info.target_met {
        "Victory without Healing Grape"
    } else if !can_search {
        "Search unavailable: routine stopped or R3 menu not reached"
    } else if require_no_grape {
        "Required no-grape victory unavailable"
    } else if info.survival_met {
        "No no-grape victory found; keep the healing routine"
    } else {
        "No winning plan found"
    };
    best.boost_search = Some(info);
    Ok(best)
}
