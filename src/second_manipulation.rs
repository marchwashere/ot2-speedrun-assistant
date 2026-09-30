//! Second-fight boost inputs control the first mansion encounter's situation.
//! Freeze the first-fight plan; replay combat and every intervening field event.
use crate::{CrtRng, Result, ensure, manipulation::BoostPlan, second};
use ot2_battle_engine::{Battle, Outcome, Side, StepReport};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Search {
    pub max_boost_repeats: Option<u32>,
    pub baseline_situation: Option<String>,
    pub situation: Option<String>,
    pub target_met: bool,
    pub limit_reached: bool,
    pub tested_candidates: u64,
    pub plan: BoostPlan,
}

fn situation(p: &second::Prediction) -> Result<Option<String>> {
    p.mansion
        .as_ref()
        .map(|m| {
            let a = &m.lookout_ambush;
            let (_, encounter) = crate::mansion_escape::first_encounter(a.after, &a.world_after)?;
            Ok(encounter.situation)
        })
        .transpose()
        .map(Option::flatten)
}
fn safe(value: &Option<String>) -> bool {
    matches!(value.as_deref(), Some("Normal" | "Preemptive"))
}

/// One adjustment at any reached party menu. Ties: fewer CRT draws, later menu.
/// None is an unlimited repetition budget, not an arbitrary finite window.
pub fn predict(
    arrival: CrtRng,
    opening: &Battle,
    reports: &[StepReport],
    cap: Option<u32>,
    replay: Option<BoostPlan>,
) -> Result<second::Prediction> {
    ensure(
        cap.is_none_or(|n| n <= 256),
        "Mansion approach cap must be 0..256 or unlimited",
    )?;
    let mut best = second::after_rejoin(arrival, opening, reports)?;
    let baseline_situation = situation(&best)?;
    let mut info = Search {
        max_boost_repeats: cap,
        situation: baseline_situation.clone(),
        target_met: safe(&baseline_situation),
        baseline_situation,
        limit_reached: false,
        tested_candidates: 0,
        plan: BoostPlan::default(),
    };
    if let Some(plan) = replay {
        if plan.advances > 0 {
            best = second::after_rejoin_with_plan(arrival, opening, reports, plan)?;
        }
        info.plan = plan;
        info.situation = situation(&best)?;
        info.target_met = safe(&info.situation);
    } else if !info.target_met && best.state.outcome == Outcome::Victory && best.mansion.is_some() {
        let preserve_one_action = best.assessment.only_one_enemy_action;
        let preserve_order = best.assessment.order_conditions_met;
        // All inputs before the selected menu remain the baseline. Resources
        // after the previous step include BP granted on entering a new round.
        let mut menus = Vec::new();
        let mut actors = &best.initial_state.actors;
        let mut menu = 0;
        for step in &best.steps {
            let actor = step.report.actor.0;
            if best.actors[actor].side == Side::Player {
                menu += 1;
                let available = actors[actor].resources.bp;
                if available > 0 {
                    menus.push((menu, available, if step.round == 1 { 0 } else { 2 }));
                }
            }
            actors = &step.actors_after;
        }
        let max_draws = cap.map_or(u32::MAX, |n| (n + 1) * 6);
        let mut best_cost = usize::MAX;
        for advances in 1..=max_draws {
            // Each press draws at most three values, with at most three normal
            // presses excluded from the cost. Higher offsets cannot win ties.
            if best_cost != usize::MAX && u64::from(advances) > (best_cost as u64 + 3) * 3 {
                break;
            }
            for &(player_menu, available, normal) in menus.iter().rev() {
                let plan = BoostPlan {
                    player_menu,
                    advances,
                    repeat_cap: cap,
                };
                let inputs = if let Some(cap) = cap {
                    crate::manipulation::optimal_inputs_with_cap(advances, available, normal, cap)?
                } else {
                    Some(plan.inputs(available, normal)?)
                };
                let Some(inputs) = inputs else { continue };
                if inputs.extra_input_count >= best_cost {
                    continue;
                }
                info.tested_candidates += 1;
                let trial = second::after_rejoin_with_plan(arrival, opening, reports, plan)?;
                if trial.state.outcome != Outcome::Victory
                    || trial.routine_error.is_some()
                    || (preserve_one_action && !trial.assessment.only_one_enemy_action)
                    || (preserve_order && !trial.assessment.order_conditions_met)
                {
                    continue;
                }
                let candidate_situation = situation(&trial)?;
                if !safe(&candidate_situation) {
                    continue;
                }
                best_cost = inputs.extra_input_count;
                info.plan = plan;
                info.situation = candidate_situation;
                info.target_met = true;
                best = trial;
            }
        }
    }
    info.limit_reached = !info.target_met && cap.is_some() && best.mansion.is_some();
    best.mansion_approach_search = Some(info);
    Ok(best)
}
