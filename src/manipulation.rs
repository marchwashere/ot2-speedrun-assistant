//! Search extra boost CRT draws at a single player command menu.
//! Replay combat through both exit resets; an end-of-battle RNG offset would
//! miss changed hits, voices, enemy turns, drops and conditional draw counts.
//! Prefer a one-enemy-action second fight within its event budget, retaining
//! encounter avoidance when the additional manipulation exceeds that budget.
use crate::{
    CrtRng, Result, ensure,
    route::{throne_ambush, throne_checkpoint, throne_reset_reach},
    second,
};
use ot2_battle_engine::{
    Battle, BattleConfig, Command, Event, ItemCatalog, Outcome, Side, StepReport,
    routine::ThroneOpeningRoutine,
};
use serde::Serialize;
use std::collections::VecDeque;

pub const INITIAL_SEARCH_ADVANCES: u32 = 256;
pub const MAX_REPEAT_CAP: u32 = 256;
pub const DEFAULT_MAX_BOOST_REPEATS: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct EventRepeatCaps {
    /// None requires this outcome; Some(n) is an optional improvement budget.
    pub encounter: Option<u32>,
    pub second_fight: Option<u32>,
}
impl Default for EventRepeatCaps {
    fn default() -> Self {
        Self {
            encounter: Some(DEFAULT_MAX_BOOST_REPEATS),
            second_fight: Some(DEFAULT_MAX_BOOST_REPEATS),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct BoostPlan {
    /// One-based party command-menu index, including Defend turns.
    pub player_menu: u32,
    pub advances: u32,
    /// Preserve the constrained input path when replaying/exporting a plan.
    pub repeat_cap: Option<u32>,
}
impl BoostPlan {
    pub fn validate(self) -> Result<()> {
        ensure(
            (self.advances == 0 && self.player_menu == 0)
                || (self.advances > 0 && (1..=6).contains(&self.player_menu)),
            "Specify a player menu from 1 to 6 with positive extra advances, or zero for both",
        )
    }
    pub fn step(
        self,
        player_menu: u32,
        battle: &mut Battle,
        routine: &mut ThroneOpeningRoutine,
    ) -> Result<(StepReport, Option<BoostInputs>)> {
        self.validate()?;
        if self.advances == 0 || self.player_menu != player_menu {
            return Ok((routine.step(battle)?, None));
        }
        let actor = battle
            .current_actor()
            .ok_or_else(|| crate::SimError("battle is over".into()))?;
        let inputs = self.inputs(
            battle.state().actors[actor.0].resources.bp,
            normal_bp(routine, battle)?,
        )?;
        let report = routine.step_with_boost_inputs(battle, &inputs.levels)?;
        Ok((report, Some(inputs)))
    }
    pub fn inputs(self, available_bp: u8, normal_bp: u8) -> Result<BoostInputs> {
        self.validate()?;
        if let Some(cap) = self.repeat_cap {
            optimal_inputs_with_cap(self.advances, available_bp, normal_bp, cap)?
                .ok_or_else(|| crate::SimError("Boost plan exceeds its repeat cap".into()))
        } else {
            optimal_inputs(self.advances, available_bp, normal_bp)
        }
    }
}

fn normal_bp(routine: &ThroneOpeningRoutine, battle: &Battle) -> Result<u8> {
    Ok(match routine.command(battle)? {
        Command::Boost { bp, .. } => bp,
        _ => 0,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExtraBoost {
    pub from: u8,
    pub to: u8,
    pub count: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoostInputs {
    /// Selected BP after every button press, including the normal action boost.
    pub levels: Vec<u8>,
    pub normal_bp: u8,
    pub input_count: usize,
    pub extra_input_count: usize,
    pub extra_repeats: u32,
    pub setup: Vec<u8>,
    pub extras: Vec<ExtraBoost>,
    /// Starts with selected BP after the last extra boost; ends at normal_bp.
    pub resume: Vec<u8>,
}

/// Breadth-first search over (CRT draws, selected BP). Every edge is one button
/// press. All three source voice sets draw 1/2/3 times on rising to BP 1/2/3;
/// their rows are 100% and not PlayOnce. Lowering BP draws nothing.
pub fn optimal_inputs(extra: u32, available_bp: u8, final_bp: u8) -> Result<BoostInputs> {
    // Pure button paths depend only on these three inputs. Bound cache storage,
    // never the draw count or required-outcome search.
    thread_local! {static PATHS:std::cell::RefCell<std::collections::BTreeMap<(u32,u8,u8),BoostInputs>>=std::cell::RefCell::new(std::collections::BTreeMap::new());}
    let key=(extra,available_bp.min(3),final_bp);
    if extra<=256 {
        if let Some(path)=PATHS.with(|p|p.borrow().get(&key).cloned()) {return Ok(path);}
    }
    let path=optimal_inputs_uncached(extra,available_bp,final_bp)?;
    if extra<=256 {PATHS.with(|p|p.borrow_mut().insert(key,path.clone()));}
    Ok(path)
}
fn optimal_inputs_uncached(extra:u32,available_bp:u8,final_bp:u8)->Result<BoostInputs> {
    let _profile=crate::profile::span("boost shortest path");
    let maximum = available_bp.min(3);
    ensure(final_bp <= maximum, "Invalid boost input target")?;
    let target = extra as usize + usize::from(final_bp) * usize::from(final_bp + 1) / 2;
    let mut previous = vec![None; (target + 1) * 4];
    let mut distance = vec![usize::MAX; previous.len()];
    distance[0] = 0;
    let mut queue = VecDeque::from([0usize]);
    previous[0] = Some(0);
    let goal = target * 4 + final_bp as usize;
    while let Some(node) = queue.pop_front() {
        if node == goal {
            break;
        }
        let (draws, bp) = (node / 4, (node % 4) as u8);
        for next in [
            bp.checked_add(1).filter(|&x| x <= maximum),
            bp.checked_sub(1),
        ]
        .into_iter()
        .flatten()
        {
            let consumed = draws + if next > bp { next as usize } else { 0 };
            if consumed > target {
                continue;
            }
            let next_node = consumed * 4 + next as usize;
            if previous[next_node].is_none() {
                previous[next_node] = Some(node);
                distance[next_node] = distance[node] + 1;
                queue.push_back(next_node);
            }
        }
    }
    ensure(
        previous[goal].is_some(),
        "Cannot produce requested boost advances with available BP",
    )?;
    let levels = readable_shortest_path(&distance, &[goal], maximum,
        |node| (node % 4) as u8,
        |node, next| {
            let draws = node / 4 + if next > (node % 4) as u8 { next as usize } else { 0 };
            (draws <= target).then_some(draws * 4 + next as usize)
        });
    describe_inputs(extra, final_bp, levels)
}

/// Among minimum-button paths, prefer single-level repeated boosts, then fewer lines.
/// Each instruction is a monotone descent, or a repeated upward climb with
/// returns to its starting BP between repetitions. Optimize the executed path,
/// not just its label, so replay/export and the displayed inputs stay identical.
fn readable_shortest_path(
    distance: &[usize], goals: &[usize], maximum: u8,
    bp: impl Fn(usize) -> u8,
    next_node: impl Fn(usize, u8) -> Option<usize>,
) -> Vec<u8> {
    let length = distance[goals[0]];
    let mut nodes: Vec<_> = (0..distance.len()).filter(|&n| distance[n] <= length).collect();
    nodes.sort_unstable_by_key(|&n| std::cmp::Reverse(distance[n]));
    let mut lines = vec![usize::MAX; distance.len()];
    let mut wide = vec![usize::MAX; distance.len()];
    let mut choice = vec![None; distance.len()];
    for &g in goals { lines[g] = 0; wide[g] = 0; }
    let advance = |mut node: usize, target: u8| -> Option<usize> {
        while bp(node) != target {
            let next = if bp(node) < target { bp(node)+1 } else { bp(node)-1 };
            let dest = next_node(node, next)?;
            if distance[dest] > length || distance[dest] != distance[node]+1 { return None; }
            node = dest;
        }
        Some(node)
    };
    for node in nodes {
        if lines[node] == 0 { continue; }
        let from = bp(node);
        for to in 0..=maximum {
            if to == from { continue; }
            let Some(mut end) = advance(node,to) else { continue; };
            let mut count = 1;
            loop {
                let penalty=usize::from(from.abs_diff(to).saturating_sub(1))*(count-1);
                if lines[end] != usize::MAX && (wide[end]+penalty,lines[end]+1) < (wide[node],lines[node]) {
                    wide[node]=wide[end]+penalty;
                    lines[node] = lines[end]+1;
                    choice[node] = Some((end,to,count));
                }
                if to < from { break; }
                let Some(repeat) = advance(end,from).and_then(|n| advance(n,to)) else { break; };
                end = repeat;
                count += 1;
            }
        }
    }
    let mut levels = Vec::with_capacity(length);
    let mut node = 0;
    while let Some((end,to,count)) = choice[node] {
        let from = bp(node);
        let mut current = from;
        for repeat in 0..count {
            if repeat > 0 {
                while current > from { current -= 1; levels.push(current); }
            }
            while current != to {
                current = if current < to { current+1 } else { current-1 };
                levels.push(current);
            }
        }
        node = end;
    }
    assert_eq!(levels.len(),length,"shortest boost instruction path");
    levels
}

fn extra_repeats(levels: &[u8], final_bp: u8) -> u32 {
    ((levels.len() - usize::from(final_bp))/2) as u32
}

/// One repeat is one cancel/reboost pair: two presses beyond normal boosting.
/// Longer climbs cannot evade the budget by counting several presses as one.
pub fn optimal_inputs_with_cap(extra:u32, available_bp:u8, final_bp:u8, cap:u32) -> Result<Option<BoostInputs>> {
    let best=optimal_inputs(extra,available_bp,final_bp)?;
    Ok((best.extra_repeats<=cap).then_some(best))
}

pub(crate) fn describe_inputs(extra: u32, final_bp: u8, levels: Vec<u8>) -> Result<BoostInputs> {
    let mut path = vec![0];
    path.extend(&levels);
    // Separate a contiguous extra-draw interval from the ordinary boost inputs.
    // Setup + resume account for exactly the action's original boost draws.
    let mut interval = None;
    'find: for start in 0..path.len() {
        let mut draws = 0;
        for end in start..path.len() {
            if end > start && path[end] > path[end - 1] {
                draws += u32::from(path[end]);
            }
            if draws == extra {
                interval = Some((start, end));
                break 'find;
            }
            if draws > extra {
                break;
            }
        }
    }
    let (start, end) =
        interval.ok_or_else(|| crate::SimError("Cannot describe extra boosts".into()))?;
    let mut extras: Vec<ExtraBoost> = Vec::new();
    let mut i = start;
    while i < end {
        if path[i + 1] < path[i] {
            i += 1;
            continue;
        }
        let from = path[i];
        let mut j = i + 1;
        while j < end && path[j + 1] > path[j] && (from == 0 || j == i) {
            j += 1;
        }
        let to = path[j];
        if let Some(last) = extras.last_mut().filter(|e| e.from == from && e.to == to) {
            last.count += 1;
        } else {
            extras.push(ExtraBoost { from, to, count: 1 });
        }
        i = j;
    }
    Ok(BoostInputs {
        extra_repeats: extra_repeats(&levels, final_bp),
        input_count: levels.len(),
        extra_input_count: levels.len() - final_bp as usize,
        levels,
        normal_bp: final_bp,
        setup: path[..=start].to_vec(),
        extras,
        resume: path[end..].to_vec(),
    })
}

#[derive(Debug, Serialize)]
pub struct SecondFight {
    pub entry: CrtRng,
    pub after: CrtRng,
    pub party_level: u8,
    pub assessment: second::Assessment,
}

#[derive(Debug, Serialize)]
pub struct Solution {
    pub plan: BoostPlan,
    pub inputs: Option<BoostInputs>,
    pub actor: Option<String>,
    pub round: Option<u32>,
    pub menu_rng: Option<CrtRng>,
    pub after_inputs_rng: Option<CrtRng>,
    pub final_rng: CrtRng,
    pub encounter_counter: u32,
    pub walking_remaining: u32,
    pub damage_received: u64,
    pub enemy_turns: u32,
    pub second_fight: SecondFight,
}
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub target: &'static str,
    pub event_caps: Option<EventRepeatCaps>,
    pub encounter_limit_reached: bool,
    pub second_fight_limit_reached: bool,
    pub second_fight_search_exhausted: bool,
    pub encounter_dodge_repeats: u32,
    pub second_fight_extra_repeats: Option<u32>,
    pub max_extra_advances: u32,
    pub tested_candidates: u64,
    pub baseline_counter: u32,
    pub minimum_walking_counter: u32,
    pub minimum_running_counter: u32,
    pub maximum_counter: u32,
    pub running_possible: bool,
    pub solution: Option<Solution>,
}
#[derive(Clone, Default)]
struct Progress {
    damage: u64,
    enemy_turns: u32,
    steps: usize,
    // Preserve the prefix at every candidate menu so EXP/overkill and the
    // resulting second-fight loadout come from the complete adjusted fight.
    reports: Vec<StepReport>,
}
impl Progress {
    fn observe(&mut self, battle: &Battle, report: &StepReport) {
        self.steps += 1;
        self.reports.push(report.clone());
        self.enemy_turns += u32::from(battle.side(report.actor) == Some(Side::Enemy));
        for event in &report.events {
            if let Event::Damage { target, amount, .. } = event {
                if battle.side(*target) == Some(Side::Player) {
                    self.damage += u64::from(*amount);
                }
            }
        }
    }
}
struct Menu {
    battle: Battle,
    routine: ThroneOpeningRoutine,
    progress: Progress,
    player_menu: u32,
    original_rng: CrtRng,
}
fn counter(battle: &Battle) -> Result<u32> {
    Ok(crate::route::throne_post_battle_options(battle.state())?
        .current_counter
        .expect("exit counter"))
}

/// Called only after verifying that walking reaches the checkpoint. Reuse the
/// route adapter so the search and displayed replay share event draws, levels,
/// HP/SP restoration, inventory and the second battle's actual action reports.
fn predict_second(battle: &Battle, progress: &Progress) -> Result<SecondFight> {
    let checkpoint = throne_checkpoint(battle.state().rng);
    let ambush = throne_ambush(&checkpoint);
    let prediction = second::after_rejoin(ambush.after_rejoin, battle, &progress.reports)?;
    Ok(SecondFight {
        entry: prediction.entry,
        after: prediction.after,
        party_level: prediction.party_level,
        assessment: prediction.assessment,
    })
}

pub fn search(seed: u32) -> Result<SearchResult> {
    search_with_event_caps(seed, EventRepeatCaps::default())
}

pub fn search_with_event_caps(seed: u32, caps: EventRepeatCaps) -> Result<SearchResult> {
    ensure(
        caps.encounter.is_none_or(|cap| cap <= MAX_REPEAT_CAP)
            && caps.second_fight.is_none_or(|cap| cap <= MAX_REPEAT_CAP),
        "Each event's boost repeat cap must be between 0 and 256",
    )?;
    // Required fight-two outcome includes its reachable-checkpoint prerequisite.
    // A finite optional checkpoint budget must not prevent this joint search.
    if caps.second_fight.is_none() {
        let mut result = search_event(seed, None, true)?;
        result.event_caps = Some(caps);
        result.second_fight_extra_repeats = result
            .solution
            .as_ref()
            .map(|s| s.inputs.as_ref().map_or(0, |i| i.extra_repeats));
        result.second_fight_search_exhausted = result.solution.is_none();
        return Ok(result);
    }
    // Establish encounter avoidance within its own cap. Fight two can add its
    // separate allowance to the actual dodge cost, never the unused allowance.
    let mut fallback = search_event(seed, caps.encounter, false)?;
    fallback.event_caps = Some(caps);
    let Some(baseline) = fallback.solution.as_ref() else {
        fallback.encounter_limit_reached = caps.encounter.is_some();
        return Ok(fallback);
    };
    let dodge_repeats = baseline
        .inputs
        .as_ref()
        .map_or(0, |inputs| inputs.extra_repeats);
    fallback.encounter_dodge_repeats = dodge_repeats;
    if baseline.second_fight.assessment.order_conditions_met
        && baseline.second_fight.assessment.only_one_enemy_action
    {
        fallback.target = "safe_checkpoint_and_one_enemy_action";
        fallback.second_fight_extra_repeats = Some(0);
        return Ok(fallback);
    }
    let total_cap = caps
        .second_fight
        .map(|cap| dodge_repeats.saturating_add(cap));
    let mut joint = search_event(seed, total_cap, true)?;
    joint.event_caps = Some(caps);
    joint.encounter_dodge_repeats = dodge_repeats;
    if let Some(solution) = joint.solution.as_ref() {
        joint.second_fight_extra_repeats = Some(
            solution
                .inputs
                .as_ref()
                .map_or(0, |inputs| inputs.extra_repeats)
                .saturating_sub(dodge_repeats),
        );
        joint.tested_candidates += fallback.tested_candidates;
        return Ok(joint);
    }
    fallback.second_fight_limit_reached = caps.second_fight.is_some();
    fallback.second_fight_search_exhausted = true;
    fallback.max_extra_advances = fallback.max_extra_advances.max(joint.max_extra_advances);
    fallback.tested_candidates += joint.tested_candidates;
    Ok(fallback)
}

fn search_event(
    seed: u32,
    repeat_cap: Option<u32>,
    require_one_action: bool,
) -> Result<SearchResult> {
    if let Some(cap) = repeat_cap {
        // A climb can draw at most 1+2+3 CRT values; include the ordinary
        // final climb. This bounds the entire capped search, not just a window.
        let range = cap.saturating_add(1).saturating_mul(6);
        search_constrained(seed, range, usize::MAX, repeat_cap, require_one_action)
    } else {
        search_unlimited(seed, INITIAL_SEARCH_ADVANCES, require_one_action)
    }
}

/// Grow the search until a match is optimal or all distinct CRT offsets have
/// been covered. CRT has a 2^32-state period; repeating it adds no new outcome.
fn search_unlimited(
    seed: u32,
    initial_range: u32,
    require_one_action: bool,
) -> Result<SearchResult> {
    let mut range = initial_range.max(1);
    let mut tested = 0;
    loop {
        let mut result = search_constrained(seed, range, usize::MAX, None, require_one_action)?;
        tested += result.tested_candidates;
        result.tested_candidates = tested;
        if let Some(solution) = &result.solution {
            // At most 3 CRT draws per upward press, and at most 3 ordinary
            // action presses. Beyond this bound no plan can tie the best cost.
            let inputs = solution
                .inputs
                .as_ref()
                .map_or(0, |i| i.extra_input_count as u64);
            let bound = ((inputs + 3) * 3).min(u64::from(u32::MAX)) as u32;
            if solution.plan.advances == 0 || range >= bound {
                return Ok(result);
            }
            range = range.saturating_mul(2).max(bound);
        } else if range == u32::MAX {
            return Ok(result);
        } else {
            range = range.saturating_mul(2);
        }
    }
}

#[cfg(test)]
fn search_limit(seed: u32, limit: u32) -> Result<SearchResult> {
    search_budget(seed, limit, usize::MAX)
}
#[cfg(test)]
fn search_budget(seed: u32, limit: u32, max_extra_inputs: usize) -> Result<SearchResult> {
    search_constrained(seed, limit, max_extra_inputs, None, true)
}
fn search_constrained(
    seed: u32,
    limit: u32,
    max_extra_inputs: usize,
    repeat_cap: Option<u32>,
    require_one_action: bool,
) -> Result<SearchResult> {
    let acceptable = |second: &SecondFight| {
        !require_one_action
            || (second.assessment.order_conditions_met && second.assessment.only_one_enemy_action)
    };
    let mut battle = Battle::new_opening(
        BattleConfig::throne_opening(seed)?,
        ItemCatalog::builtin()?,
        false,
    )?;
    let mut routine = ThroneOpeningRoutine::default();
    let mut progress = Progress::default();
    let mut menus = Vec::new();
    let mut player_menu = 0;
    while battle.state().outcome == Outcome::Ongoing {
        ensure(progress.steps < 16, "Routine exceeded its action budget")?;
        let actor = battle.current_actor().expect("ongoing actor");
        if battle.side(actor) == Some(Side::Player) {
            player_menu += 1;
            if battle.state().actors[actor.0].resources.bp > 0 {
                menus.push(Menu {
                    original_rng: battle.state().rng,
                    battle: battle.clone(),
                    routine: routine.clone(),
                    progress: progress.clone(),
                    player_menu,
                });
            }
        }
        let report = routine.step(&mut battle)?;
        progress.observe(&battle, &report);
    }
    let baseline_counter = counter(&battle)?;
    let reach = throne_reset_reach(baseline_counter)?;
    let profile = crate::route::throne_field_profile();
    let c = &crate::encounter::catalog().maps[&profile.options.map].counter;
    let maximum_counter = ((c.min + c.range_a.saturating_sub(1) + c.range_b.saturating_sub(1))
        as f32
        * c.first_ratio) as u32;
    let mut output = SearchResult {
        target: if require_one_action {
            "safe_checkpoint_and_one_enemy_action"
        } else {
            "safe_checkpoint"
        },
        event_caps: None,
        encounter_limit_reached: false,
        second_fight_limit_reached: false,
        second_fight_search_exhausted: false,
        encounter_dodge_repeats: 0,
        second_fight_extra_repeats: None,
        max_extra_advances: limit,
        tested_candidates: 0,
        baseline_counter,
        minimum_walking_counter: reach.walking.minimum_start_counter,
        minimum_running_counter: reach.running.minimum_start_counter,
        maximum_counter,
        running_possible: maximum_counter >= reach.running.minimum_start_counter,
        solution: None,
    };
    if reach.walking.reaches_reset {
        let second_fight = predict_second(&battle, &progress)?;
        if acceptable(&second_fight) {
            output.solution = Some(Solution {
                plan: BoostPlan::default(),
                inputs: None,
                actor: None,
                round: None,
                menu_rng: None,
                after_inputs_rng: None,
                final_rng: battle.state().rng,
                encounter_counter: baseline_counter,
                walking_remaining: reach.walking.counter_before_reset.unwrap(),
                damage_received: progress.damage,
                enemy_turns: progress.enemy_turns,
                second_fight,
            });
            return Ok(output);
        }
    }
    // Optimize actual button presses, including the return to the action BP.
    // For ties use fewer extra draws, then the latest eligible command menu.
    let mut candidates = Vec::new();
    for (index, menu) in menus.iter().enumerate() {
        let actor = menu.battle.current_actor().expect("player menu");
        let bp = normal_bp(&menu.routine, &menu.battle)?;
        for advances in 1..=limit {
            let available = menu.battle.state().actors[actor.0].resources.bp;
            let inputs = if let Some(cap) = repeat_cap {
                let Some(inputs) = optimal_inputs_with_cap(advances, available, bp, cap)? else {
                    continue;
                };
                inputs
            } else {
                optimal_inputs(advances, available, bp)?
            };
            if inputs.extra_input_count <= max_extra_inputs {
                candidates.push((index, advances, inputs));
            }
        }
    }
    candidates.sort_by_key(|(i, advances, inputs)| {
        (
            inputs.extra_input_count,
            *advances,
            std::cmp::Reverse(menus[*i].player_menu),
        )
    });
    for (index, cycles, inputs) in candidates {
        let menu = &menus[index];
        let mut trial = menu.battle.clone();
        let mut policy = menu.routine.clone();
        let mut progress = menu.progress.clone();
        output.tested_candidates += 1;
        let report = policy.step_with_boost_inputs(&mut trial, &inputs.levels)?;
        progress.observe(&trial, &report);
        let mut supported = true;
        while trial.state().outcome == Outcome::Ongoing && progress.steps < 16 {
            match policy.step(&mut trial) {
                Ok(report) => progress.observe(&trial, &report),
                Err(_) => {
                    supported = false;
                    break;
                }
            }
        }
        if !supported || trial.state().outcome != Outcome::Victory {
            continue;
        }
        let encounter_counter = counter(&trial)?;
        let reach = throne_reset_reach(encounter_counter)?;
        if reach.walking.reaches_reset {
            let second_fight = predict_second(&trial, &progress)?;
            if !acceptable(&second_fight) {
                continue;
            }
            let actor = menu.battle.current_actor().expect("player menu");
            output.solution = Some(Solution {
                plan: BoostPlan {
                    player_menu: menu.player_menu,
                    advances: cycles,
                    repeat_cap,
                },
                after_inputs_rng: Some({
                    let mut rng = menu.original_rng;
                    rng.advance(
                        u64::from(cycles)
                            + u64::from(inputs.normal_bp) * u64::from(inputs.normal_bp + 1) / 2,
                    );
                    rng
                }),
                inputs: Some(inputs),
                actor: Some(menu.battle.actor_name(actor).expect("actor name").into()),
                round: Some(menu.battle.state().round),
                menu_rng: Some(menu.original_rng),
                final_rng: trial.state().rng,
                encounter_counter,
                walking_remaining: reach.walking.counter_before_reset.unwrap(),
                damage_received: progress.damage,
                enemy_turns: progress.enemy_turns,
                second_fight,
            });
            return Ok(output);
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    #[test]
    fn cached_boost_paths_match_uncached_planner_and_large_counts() {
        for available in 1..=3 {
            for final_bp in 0..=available {
                for extra in [0,1,2,3,7,16,31,64,255,256,257,300] {
                    let expected=super::optimal_inputs_uncached(extra,available,final_bp).unwrap();
                    assert_eq!(super::optimal_inputs(extra,available,final_bp).unwrap(),expected);
                    assert_eq!(super::optimal_inputs(extra,available,final_bp).unwrap(),expected);
                }
            }
        }
        assert!(super::optimal_inputs(1,0,0).is_err());
        assert!(super::optimal_inputs(0,1,2).is_err());
    }
    #[test]
    fn mug_tie_prefers_two_grouped_instructions() {
        let expected = vec![1,0,1,2,1,2,1,2,1,2];
        let p = super::optimal_inputs(7,2,2).unwrap();
        assert_eq!(p.levels,expected);
        assert_eq!(p.input_count,10);
        assert_eq!(p.extra_repeats,4);
        assert_eq!(super::optimal_inputs_with_cap(7,2,2,4).unwrap().unwrap().levels,expected);
        // A tighter repeat budget must retain its own legal path.
        assert!(super::optimal_inputs_with_cap(7,2,2,3).unwrap().is_none());
    }
    use super::*;
    #[test]
    fn repeat_cap_counts_cancel_reboost_pairs() {
        assert_eq!(optimal_inputs(3, 2, 2).unwrap().extra_repeats, 2);
        assert_eq!(optimal_inputs(28, 2, 2).unwrap().extra_repeats, 14);
        let inputs = optimal_inputs_with_cap(12, 2, 2, 6).unwrap().unwrap();
        assert_eq!(inputs.extra_repeats, 6);
        assert_eq!(inputs.input_count, 14);
        assert_eq!(inputs.levels,vec![1,2,1,2,1,2,1,2,1,2,1,2,1,2]);
        assert_eq!(inputs.levels.last(), Some(&2));
        assert!(optimal_inputs_with_cap(12, 2, 2, 3).unwrap().is_none());
        assert!(optimal_inputs_with_cap(1, 2, 2, 0).unwrap().is_none());
        assert_eq!(
            optimal_inputs_with_cap(0, 2, 2, 0)
                .unwrap()
                .unwrap()
                .extra_repeats,
            0
        );
        assert!(
            optimal_inputs_with_cap(0, 2, 2, u32::MAX)
                .unwrap()
                .is_some()
        );
        let plan = BoostPlan {
            player_menu: 6,
            advances: 12,
            repeat_cap: Some(6),
        };
        assert_eq!(plan.inputs(2, 2).unwrap(), inputs);
    }
    #[test]
    fn event_budget_excludes_the_encounter_dodge_cost_and_falls_back() {
        let default = search(battle_seed(4916)).unwrap();
        assert_eq!(default.event_caps, Some(EventRepeatCaps::default()));
        assert!(!default.second_fight_limit_reached);
        let limited = search_with_event_caps(
            battle_seed(4916),
            EventRepeatCaps {
                encounter: Some(2),
                second_fight: Some(2),
            },
        )
        .unwrap();
        assert_eq!(limited.encounter_dodge_repeats, 2);
        assert!(limited.second_fight_limit_reached);
        assert_eq!(limited.solution.unwrap().plan.advances, 3);
        let joint = search_with_event_caps(
            battle_seed(4916),
            EventRepeatCaps {
                encounter: Some(2),
                second_fight: Some(4),
            },
        )
        .unwrap();
        assert!(!joint.second_fight_limit_reached);
        assert_eq!(joint.encounter_dodge_repeats, 2);
        assert_eq!(joint.second_fight_extra_repeats, Some(4));
        let solution = joint.solution.unwrap();
        assert_eq!(solution.inputs.as_ref().unwrap().extra_repeats, 6);
        assert_eq!(solution.plan.repeat_cap, Some(6));
        assert!(solution.second_fight.assessment.only_one_enemy_action);
        assert!(
            search_with_event_caps(
                battle_seed(4916),
                EventRepeatCaps {
                    encounter: Some(2),
                    second_fight: Some(2)
                }
            )
            .unwrap()
            .second_fight_limit_reached
        );
        let free = search_with_event_caps(
            battle_seed(19),
            EventRepeatCaps {
                encounter: Some(0),
                second_fight: Some(0),
            },
        )
        .unwrap();
        assert!(!free.second_fight_limit_reached);
        assert_eq!(free.second_fight_extra_repeats, Some(0));
        assert_eq!(free.solution.unwrap().plan, BoostPlan::default());
    }

    #[test]
    fn event_caps_are_independent_and_unused_budget_is_not_shared() {
        let blocked = search_with_event_caps(
            battle_seed(4916),
            EventRepeatCaps {
                encounter: Some(0),
                second_fight: Some(14),
            },
        )
        .unwrap();
        assert!(blocked.encounter_limit_reached);
        assert!(!blocked.second_fight_limit_reached);
        assert!(blocked.solution.is_none());
        for encounter in [2, 10] {
            let fallback = search_with_event_caps(
                battle_seed(4916),
                EventRepeatCaps {
                    encounter: Some(encounter),
                    second_fight: Some(2),
                },
            )
            .unwrap();
            assert!(!fallback.encounter_limit_reached);
            assert!(fallback.second_fight_limit_reached);
            assert_eq!(fallback.encounter_dodge_repeats, 2);
            assert_eq!(fallback.solution.unwrap().plan.advances, 3);
        }
    }

    #[test]
    fn required_second_fight_solves_checkpoint_prerequisite_before_optional_caps() {
        let caps = EventRepeatCaps {
            encounter: Some(2),
            second_fight: None,
        };
        let joint = search_with_event_caps(battle_seed(4916), caps).unwrap();
        assert_eq!(joint.event_caps, Some(caps));
        assert_eq!(joint.encounter_dodge_repeats, 0);
        assert_eq!(joint.second_fight_extra_repeats, Some(6));
        let solution = joint.solution.unwrap();
        assert_eq!(solution.plan.advances, 12);
        assert_eq!(solution.plan.repeat_cap, None);
        assert_eq!(solution.inputs.unwrap().input_count, 14);
        assert!(solution.second_fight.assessment.only_one_enemy_action);
        let required = search_with_event_caps(
            battle_seed(4916),
            EventRepeatCaps {
                encounter: Some(0),
                second_fight: None,
            },
        )
        .unwrap();
        assert!(!required.encounter_limit_reached);
        assert_eq!(required.solution.unwrap().plan, solution.plan);
        let fallback = search_with_event_caps(
            battle_seed(4916),
            EventRepeatCaps {
                encounter: None,
                second_fight: Some(0),
            },
        )
        .unwrap();
        assert_eq!(fallback.encounter_dodge_repeats, 2);
        assert!(fallback.second_fight_limit_reached);
        assert_eq!(fallback.solution.unwrap().plan.advances, 3);
    }

    #[test]
    fn unlimited_search_grows_and_boost_paths_can_exceed_old_window() {
        // Start below this fixture's necessary +12 draws, exercising expansion
        // and the proof that an unseen cheaper input path cannot be missed.
        let result = search_unlimited(battle_seed(4916), 8, true).unwrap();
        let solution = result.solution.unwrap();
        assert_eq!(solution.plan.advances, 12);
        assert!(result.max_extra_advances >= 45);
        let plan = BoostPlan {
            player_menu: 6,
            advances: 300,
            repeat_cap: None,
        };
        plan.validate().unwrap();
        let inputs = plan.inputs(2, 2).unwrap();
        let mut bp = 0;
        let mut draws = 0;
        for &next in &inputs.levels {
            if next > bp {
                draws += u32::from(next);
            }
            bp = next;
        }
        assert_eq!(bp, 2);
        assert_eq!(draws, 303); // +300 and the action's ordinary +3
    }

    #[test]
    fn capped_inputs_match_exhaustive_legal_button_paths() {
        // Independent enumeration records shortest legal paths of at most
        // twelve inputs, including direction changes and normal final boosts.
        fn visit(
            bp: u8,
            draws: u32,
            steps: usize,
            climbs: u32,
            previous: u8,
            costs: &mut std::collections::BTreeMap<(u32, u8, u32), usize>,
        ) {
            let repeats = (steps-u32::from(bp) as usize)/2;
            let repeats = repeats as u32;
            costs
                .entry((draws, bp, repeats))
                .and_modify(|n| *n = (*n).min(steps))
                .or_insert(steps);
            if steps == 12 {
                return;
            }
            if bp < 2 {
                visit(
                    bp + 1,
                    draws + u32::from(bp + 1),
                    steps + 1,
                    climbs + u32::from(bp <= previous),
                    bp,
                    costs,
                );
            }
            if bp > 0 {
                visit(bp - 1, draws, steps + 1, climbs, bp, costs);
            }
        }
        let mut costs = std::collections::BTreeMap::new();
        visit(0, 0, 0, 0, 0, &mut costs);
        for cap in 0..=4 {
            for extra in 0..=12 {
                for final_bp in 0..=2 {
                    let draws = extra + u32::from(final_bp) * u32::from(final_bp + 1) / 2;
                    let expected = costs
                        .iter()
                        .filter(|((d, b, r), _)| *d == draws && *b == final_bp && *r <= cap)
                        .map(|(_, v)| *v)
                        .min();
                    let inputs = optimal_inputs_with_cap(extra, 2, final_bp, cap).unwrap();
                    assert_eq!(
                        inputs.as_ref().map(|p| p.input_count).filter(|n| *n <= 12),
                        expected,
                        "extra {extra}, BP {final_bp}, cap {cap}"
                    );
                    if let Some(inputs) = inputs {
                        assert!(inputs.extra_repeats <= cap);
                        let mut actual = 0;
                        let mut bp = 0_u8;
                        for next in inputs.levels {
                            assert_eq!(bp.abs_diff(next), 1);
                            if next > bp {
                                actual += u32::from(next);
                            }
                            bp = next;
                        }
                        assert_eq!(actual, draws);
                        assert_eq!(bp, final_bp);
                    }
                }
            }
        }
    }
    #[test]
    fn instructions_reconstruct_the_exact_optimized_inputs() {
        for available in 1..=3 {
            for normal in 0..=available {
                for extra in 1..=256 {
                    let p = optimal_inputs(extra, available, normal).unwrap();
                    let mut rebuilt = p.setup[1..].to_vec();
                    let mut bp = *p.setup.last().unwrap();
                    for group in &p.extras {
                        assert!(
                            [(0, 1), (0, 2), (0, 3), (1, 2), (2, 3)]
                                .contains(&(group.from, group.to))
                        );
                        for _ in 0..group.count {
                            assert!(bp >= group.from);
                            while bp > group.from {
                                bp -= 1;
                                rebuilt.push(bp);
                            }
                            while bp < group.to {
                                bp += 1;
                                rebuilt.push(bp);
                            }
                        }
                    }
                    assert_eq!(p.resume[0], bp);
                    rebuilt.extend(&p.resume[1..]);
                    assert_eq!(rebuilt, p.levels);
                    let mut current = 0u8;
                    let mut draws = 0u32;
                    for &level in &p.levels {
                        assert!(level <= available && current.abs_diff(level) == 1);
                        if level > current {
                            draws += u32::from(level);
                        }
                        current = level;
                    }
                    assert_eq!(current, normal);
                    assert_eq!(draws, extra + u32::from(normal) * u32::from(normal + 1) / 2);
                }
            }
        }
    }
    #[test]
    fn shortest_inputs_match_exhaustive_button_paths() {
        fn visit(
            bp: u8,
            draws: u32,
            presses: usize,
            max: u8,
            costs: &mut std::collections::BTreeMap<(u32, u8), usize>,
        ) {
            costs
                .entry((draws, bp))
                .and_modify(|v| *v = (*v).min(presses))
                .or_insert(presses);
            if presses == 12 {
                return;
            }
            if bp < max {
                visit(bp + 1, draws + u32::from(bp + 1), presses + 1, max, costs);
            }
            if bp > 0 {
                visit(bp - 1, draws, presses + 1, max, costs);
            }
        }
        for maximum in 1..=3 {
            let mut costs = std::collections::BTreeMap::new();
            visit(0, 0, 0, maximum, &mut costs);
            for ((draws, bp), cost) in costs {
                let normal_draws = u32::from(bp) * u32::from(bp + 1) / 2;
                if draws >= normal_draws {
                    assert_eq!(
                        optimal_inputs(draws - normal_draws, maximum, bp)
                            .unwrap()
                            .input_count,
                        cost
                    );
                }
            }
        }
        // Two extra draws can share the action's initial 0->1 input.
        assert_eq!(optimal_inputs(2, 2, 2).unwrap().levels, [1, 2, 1, 2]);
    }
    fn battle_seed(title_advances: u64) -> u32 {
        crate::opening::prepare(0, title_advances, false)
            .unwrap()
            .battle_state
    }
    #[test]
    fn safe_counter_alone_does_not_satisfy_the_joint_target() {
        let baseline = search_limit(battle_seed(4920), 0).unwrap();
        assert_eq!(baseline.baseline_counter, 698);
        assert!(baseline.solution.is_none());
        let r = search_limit(battle_seed(4920), INITIAL_SEARCH_ADVANCES).unwrap();
        let s = r.solution.unwrap();
        assert_eq!(
            s.plan,
            BoostPlan {
                player_menu: 6,
                advances: 8,
                repeat_cap: None,
            }
        );
        assert!(s.encounter_counter >= 685);
        assert!(s.second_fight.assessment.only_one_enemy_action);
        assert!(s.second_fight.assessment.order_conditions_met);
        assert!(r.tested_candidates > 0);
        assert!(!r.running_possible);
        assert_eq!(
            (
                r.minimum_walking_counter,
                r.minimum_running_counter,
                r.maximum_counter
            ),
            (685, 913, 899)
        );
    }
    #[test]
    fn zero_extra_inputs_are_kept_when_both_targets_already_pass() {
        let r = search(battle_seed(19)).unwrap();
        let s = r.solution.unwrap();
        assert_eq!(r.tested_candidates, 0);
        assert_eq!(s.plan, BoostPlan::default());
        assert!(s.inputs.is_none());
        assert_eq!(s.encounter_counter, 731);
        assert!(s.second_fight.assessment.order_conditions_met);
        assert!(s.second_fight.assessment.only_one_enemy_action);
    }
    #[test]
    fn solution_replays_from_the_original_seed_and_is_minimal_at_one_menu() {
        for advances in [4916, 4918, 4920, 4922] {
            let seed = battle_seed(advances);
            let r = search_limit(seed, INITIAL_SEARCH_ADVANCES).unwrap();
            let s = r.solution.expect("solution in bounded search");
            let mut battle = Battle::new_opening(
                BattleConfig::throne_opening(seed).unwrap(),
                ItemCatalog::builtin().unwrap(),
                true,
            )
            .unwrap();
            let mut routine = ThroneOpeningRoutine::default();
            let mut menu = 0;
            let mut progress = Progress::default();
            while battle.state().outcome == Outcome::Ongoing {
                if battle.side(battle.current_actor().unwrap()) == Some(Side::Player) {
                    menu += 1;
                }
                let report = if battle.side(battle.current_actor().unwrap()) == Some(Side::Player) {
                    s.plan.step(menu, &mut battle, &mut routine).unwrap().0
                } else {
                    routine.step(&mut battle).unwrap()
                };
                progress.observe(&battle, &report);
            }
            assert_eq!(battle.state().rng, s.final_rng);
            assert_eq!(counter(&battle).unwrap(), s.encounter_counter);
            assert_eq!(progress.damage, s.damage_received);
            assert_eq!(progress.enemy_turns, s.enemy_turns);
            assert!(s.encounter_counter >= 685);
            let checkpoint = throne_checkpoint(battle.state().rng);
            let ambush = throne_ambush(&checkpoint);
            let second =
                second::after_rejoin(ambush.after_rejoin, &battle, &progress.reports).unwrap();
            assert_eq!(second.entry, s.second_fight.entry);
            assert_eq!(second.after, s.second_fight.after);
            assert_eq!(second.party_level, s.second_fight.party_level);
            assert_eq!(second.assessment, s.second_fight.assessment);
            assert!(second.assessment.order_conditions_met);
            assert!(second.assessment.only_one_enemy_action);
            assert_eq!(second.state.outcome, Outcome::Victory);
            assert_eq!(second.entry.draws - battle.state().rng.draws, 9);
            if s.plan.advances > 0 {
                assert!(
                    search_budget(
                        seed,
                        INITIAL_SEARCH_ADVANCES,
                        s.inputs.as_ref().unwrap().extra_input_count - 1
                    )
                    .unwrap()
                    .solution
                    .is_none()
                );
                assert_eq!(
                    s.after_inputs_rng.unwrap().draws - s.menu_rng.unwrap().draws,
                    u64::from(s.plan.advances) + {
                        let bp = u64::from(s.inputs.as_ref().unwrap().normal_bp);
                        bp * (bp + 1) / 2
                    }
                );
            }
        }
    }
    #[test]
    fn encounter_only_solution_is_rejected_and_uses_minimum_joint_inputs() {
        let r = search_limit(battle_seed(4916), INITIAL_SEARCH_ADVANCES).unwrap();
        let s = r.solution.unwrap();
        // The +3 encounter-only plan is safe to walk but misses the fight target. The
        // joint search must continue, without rewriting the player's routine.
        assert_eq!(
            s.plan,
            BoostPlan {
                player_menu: 6,
                advances: 12,
                repeat_cap: None,
            }
        );
        assert_eq!(s.inputs.as_ref().unwrap().extra_input_count, 12);
        assert_eq!(s.encounter_counter, 760);
        assert_eq!(s.second_fight.assessment.enemy_actions, 1);
        assert_eq!(s.second_fight.entry.state, 0x76E70570);
        assert!(
            search_limit(battle_seed(4916), 3)
                .unwrap()
                .solution
                .is_none()
        );
    }
    #[test]
    fn zero_budget_and_invalid_plans_are_explicit() {
        assert!(
            search_limit(battle_seed(4916), 0)
                .unwrap()
                .solution
                .is_none()
        );
        for plan in [
            BoostPlan {
                player_menu: 0,
                advances: 1,
                repeat_cap: None,
            },
            BoostPlan {
                player_menu: 7,
                advances: 1,
                repeat_cap: None,
            },
            BoostPlan {
                player_menu: 1,
                advances: 0,
                repeat_cap: None,
            },
        ] {
            assert!(plan.validate().is_err());
        }
    }
}
