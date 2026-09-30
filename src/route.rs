//! The route owns persistent CRT position and the field counter. Battles remain
//! independent simulations, receiving a state and returning their consumed draws.
use crate::{
    CrtRng, Result,
    encounter::{Boundary, Prediction, PredictionOptions, catalog},
    ensure,
};
use ot2_battle_engine::{ActorId, BattleState, Outcome, Side, StepReport};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Debug, Deserialize, Serialize)]
pub struct ThroneFieldProfile {
    pub id: String,
    pub label: String,
    pub boundary_label: String,
    pub event: String,
    pub valid_until_event: String,
    pub encounter_actor: String,
    pub story_task_progress: u32,
    pub known_flags: BTreeMap<String, bool>,
    pub party: Vec<String>,
    pub options: PredictionOptions,
    pub notes: Vec<String>,
}

pub fn throne_field_profile() -> &'static ThroneFieldProfile {
    static PROFILE: OnceLock<ThroneFieldProfile> = OnceLock::new();
    PROFILE.get_or_init(|| {
        serde_json::from_str(include_str!("../data/throne-field.json"))
            .expect("imported story frame")
    })
}

/// Only the fresh-new-game boundary after MS_TOU_10_0300, before 0400.
/// Story/task progress and global encounter difficulty are distinct values.
pub fn throne_post_battle_options(state: &BattleState) -> Result<PredictionOptions> {
    ensure(
        state.outcome == Outcome::Victory,
        "The opening battle must end in victory",
    )?;
    let after = state
        .opening_crt
        .as_ref()
        .and_then(|c| c.after_battle.as_ref())
        .ok_or_else(|| crate::SimError("Opening battle exit callbacks are unavailable".into()))?;
    ensure(
        state.rng.state == after.rng_after_post_battle_event.state,
        "Expected the frame after the post-battle event reset",
    )?;
    Ok(PredictionOptions {
        current_counter: Some(after.encounter_steps),
        ..throne_field_profile().options.clone()
    })
}

/// Route mode accepts movement/gap changes, but cannot silently mix a later
/// story's encounter settings with this opening frame. Custom mode is separate.
pub fn validate_throne_field_options(options: &PredictionOptions) -> Result<()> {
    let mut expected = throne_field_profile().options.clone();
    expected.current_counter = options.current_counter;
    expected.dash = options.dash;
    expected.intervening_advances = options.intervening_advances.clone();
    ensure(
        options.current_counter.is_some()
            && serde_json::to_value(options)? == serde_json::to_value(expected)?,
        "Post-Throné route settings do not match its story frame. Select Custom area for a different progression, party or map.",
    )
}

#[derive(Debug, Serialize)]
pub struct EnemyTurn {
    pub actor: ActorId,
    pub round: u32,
}

/// Observe actual executed turns, including attacks that miss. Broken/defeated
/// actors removed from the queue never produce a report. No RNG is consumed.
#[derive(Debug, Default, Serialize)]
pub struct BattleAssessment {
    pub no_enemy_actions: bool,
    pub enemy_turns: Vec<EnemyTurn>,
}
impl BattleAssessment {
    pub fn observe(&mut self, round: u32, side: Side, report: &StepReport) {
        if side == Side::Enemy {
            self.enemy_turns.push(EnemyTurn {
                actor: report.actor,
                round,
            });
        }
        self.no_enemy_actions = report.outcome == Outcome::Victory && self.enemy_turns.is_empty();
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WalkMeasurement {
    pub source: String,
    pub counter_before: u32,
    pub counter_after: u32,
    pub assumption: String,
    pub limitation: String,
    pub source_functions: Vec<String>,
    pub source_file: String,
    pub zero_counter_policy: String,
}
fn walk_measurement() -> &'static WalkMeasurement {
    static MEASUREMENT: OnceLock<WalkMeasurement> = OnceLock::new();
    MEASUREMENT.get_or_init(|| {
        serde_json::from_str(include_str!("../data/throne-reset-walk.json")).expect("recorded walk")
    })
}

#[derive(Debug, Serialize)]
pub struct MovementBudget {
    pub updates: u32,
    pub decrement: u32,
    pub cost: u32,
    pub minimum_start_counter: u32,
    pub reaches_reset: bool,
    /// Only defined when no encounter is predicted on the route.
    pub counter_before_reset: Option<u32>,
    pub encounter_on_update: Option<u32>,
}
impl MovementBudget {
    pub(crate) fn new(counter: u32, updates: u32, decrement: u32) -> Self {
        let cost = updates * decrement;
        let reaches_reset = counter > cost;
        Self {
            updates,
            decrement,
            cost,
            minimum_start_counter: cost + 1,
            reaches_reset,
            counter_before_reset: reaches_reset.then(|| counter - cost),
            encounter_on_update: (!reaches_reset).then(|| counter.div_ceil(decrement)),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MixedMovementBudget {
    pub max_running_updates: u32,
    pub walking_updates: u32,
    /// Share of counter updates, not elapsed time; rounded down to 0.1%.
    pub max_running_percent: f64,
    pub cost: u32,
    pub counter_before_reset: u32,
}

#[derive(Debug, Serialize)]
pub struct ResetReach {
    pub initial_counter: u32,
    pub assumed_updates: u32,
    pub walking: MovementBudget,
    pub running: MovementBudget,
    /// None means even walking the whole route reaches the encounter threshold.
    pub mixed: Option<MixedMovementBudget>,
    pub recommendation: &'static str,
    pub measurement: &'static WalkMeasurement,
}

/// Prediction under the user's measured minimum-path assumption, not a movement
/// simulation. Inspecting either speed must never advance the route CRT state.
pub fn throne_reset_reach(counter: u32) -> Result<ResetReach> {
    let measurement = walk_measurement();
    let mut options = throne_field_profile().options.clone();
    let walk = catalog().counter_decrement(&options)?;
    options.dash = true;
    let run = catalog().counter_decrement(&options)?;
    let consumed = measurement.counter_before - measurement.counter_after;
    ensure(
        walk > 0 && run > 0 && consumed.is_multiple_of(walk),
        "Walking measurement does not match the active decrement",
    )?;
    let updates = consumed / walk;
    let walking = MovementBudget::new(counter, updates, walk);
    let running = MovementBudget::new(counter, updates, run);
    ensure(
        updates > 0 && run > walk,
        "Mixed movement requires a positive route and a higher running decrement",
    )?;
    let mixed = walking.reaches_reset.then(|| {
        // Estimate only: the opening route has a walking measurement but no
        // separate running/mixed measurement. UpdatePlayerStep discards excess
        // displacement, so actual update counts depend on movement speed.
        // Reserve at least one point: zero enables encounters.
        let max_running_updates = ((counter - 1 - walking.cost) / (run - walk)).min(updates);
        let cost = walking.cost + max_running_updates * (run - walk);
        MixedMovementBudget {
            max_running_updates,
            walking_updates: updates - max_running_updates,
            max_running_percent: ((max_running_updates * 1000) / updates) as f64 / 10.0,
            cost,
            counter_before_reset: counter - cost,
        }
    });
    let recommendation = if running.reaches_reset {
        "run"
    } else if walking.reaches_reset {
        "walk"
    } else {
        "encounter_expected"
    };
    Ok(ResetReach {
        initial_counter: counter,
        assumed_updates: updates,
        walking,
        running,
        mixed,
        recommendation,
        measurement,
    })
}

#[derive(Debug, Serialize)]
pub struct ThroneCheckpoint {
    pub event: &'static str,
    pub arrival: CrtRng,
    pub after_popup: CrtRng,
    pub encounter_counter: u32,
    pub encounters_enabled: bool,
    pub party: [&'static str; 1],
    pub verification: &'static str,
    pub trace: Vec<ot2_battle_engine::opening_crt::RngSpan>,
}

/// Source-derived MS_TOU_10_0400 projection; caller must first establish that
/// the trigger is reached without an encounter. Movement itself draws no CRT.
/// See docs/throne-checkpoint.md; this boundary awaits live validation.
pub fn throne_checkpoint(arrival: CrtRng) -> ThroneCheckpoint {
    use ot2_battle_engine::opening_crt::RngSpan;
    let mut rng = arrival;
    let mut trace = Vec::new();
    for (reason, actor) in [
        ("checkpoint_pirro_npc_frame", ActorId(1)),
        ("checkpoint_scaracci_npc_frame", ActorId(2)),
    ] {
        let before = rng;
        rng.index(16);
        trace.push(RngSpan {
            reason,
            actor: Some(actor),
            before,
            after: rng,
        });
    }
    let counter = &catalog().maps[&throne_field_profile().options.map].counter;
    let before = rng;
    let b = rng.index(counter.range_b);
    let a = rng.index(counter.range_a);
    trace.push(RngSpan {
        reason: "checkpoint_counter_reset_b_then_a",
        actor: None,
        before,
        after: rng,
    });
    trace.push(RngSpan {
        reason: "checkpoint_ambush_popup",
        actor: None,
        before: rng,
        after: rng,
    });
    ThroneCheckpoint {
        event: "MS_TOU_10_0400",
        arrival,
        after_popup: rng,
        encounter_counter: ((counter.min + b + a) as f32 * counter.first_ratio) as u32,
        encounters_enabled: false,
        party: ["Throné"],
        verification: "Source-derived; checkpoint cutscene and tutorial await live validation",
        trace,
    }
}

#[derive(Debug, Serialize)]
pub struct ThroneAmbush {
    pub event: &'static str,
    pub rejoin_event: &'static str,
    pub before_menu: CrtRng,
    pub at_menu: CrtRng,
    pub after_ambush: CrtRng,
    pub after_rejoin: CrtRng,
    pub ambush_counter: u32,
    pub encounter_counter: u32,
    pub encounters_enabled: bool,
    pub encounter_actor: &'static str,
    pub party: [&'static str; 3],
    pub known_flags: BTreeMap<&'static str, bool>,
    pub verification: &'static str,
    pub trace: Vec<ot2_battle_engine::opening_crt::RngSpan>,
}

/// Continue from the reachable checkpoint tutorial: open the menu once, Ambush
/// the first guard, then skip the rejoin cutscene. The walk has no encounters
/// while flag 25304 is set. This projection does not mutate the battle result.
pub fn throne_ambush(checkpoint: &ThroneCheckpoint) -> ThroneAmbush {
    use ot2_battle_engine::opening_crt::RngSpan;
    let before_menu = checkpoint.after_popup;
    let mut rng = before_menu;
    // K2_SetTimerDelegate calls CRT rand even with zero delay variance.
    rng.advance(1);
    let at_menu = rng;
    let mut trace = vec![
        RngSpan {
            reason: "ambush_menu_action_timer",
            actor: None,
            before: before_menu,
            after: at_menu,
        },
        RngSpan {
            reason: "ambush_scripted_dialogue",
            actor: None,
            before: at_menu,
            after: at_menu,
        },
    ];
    let counter = &catalog().maps[&throne_field_profile().options.map].counter;
    let mut reset = |reason| {
        let before = rng;
        let b = rng.index(counter.range_b);
        let a = rng.index(counter.range_a);
        trace.push(RngSpan {
            reason,
            actor: None,
            before,
            after: rng,
        });
        (
            rng,
            ((counter.min + b + a) as f32 * counter.first_ratio) as u32,
        )
    };
    // 0410 explicitly finishes before starting 0500. Each event resets once.
    let (after_ambush, ambush_counter) = reset("ambush_counter_reset_b_then_a");
    let (after_rejoin, encounter_counter) = reset("rejoin_counter_reset_b_then_a");
    ThroneAmbush {
        event: "MS_TOU_10_0410",
        rejoin_event: "MS_TOU_10_0500",
        before_menu,
        at_menu,
        after_ambush,
        after_rejoin,
        ambush_counter,
        encounter_counter,
        encounters_enabled: true,
        encounter_actor: &throne_field_profile().encounter_actor,
        party: ["Throné", "Pirro", "Scaracci"],
        known_flags: BTreeMap::from([("8003", true), ("8004", true), ("25304", false)]),
        verification: "Source-derived; one menu opening and skipped rejoin cutscene; awaiting live trace validation",
        trace,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteState {
    pub initial_seed: u32,
    pub rng: CrtRng,
    pub encounter_counter: Option<u32>,
}
impl RouteState {
    pub fn new(initial_seed: u32, advances: u64) -> Self {
        let mut rng = CrtRng::new(initial_seed);
        rng.advance(advances);
        Self {
            initial_seed,
            rng,
            encounter_counter: None,
        }
    }
    pub fn advance(&mut self, draws: u64) -> Result<()> {
        ensure(
            self.rng.draws.checked_add(draws).is_some(),
            "Route advance count overflow",
        )?;
        self.rng.advance(draws);
        Ok(())
    }
    pub fn predict_encounter(&mut self, options: &PredictionOptions) -> Result<Prediction> {
        let mut options = options.clone();
        if options.boundary == Boundary::CurrentCounter && options.current_counter.is_none() {
            options.current_counter = self.encounter_counter;
        }
        let p = catalog().predict(&mut self.rng, &options)?;
        self.encounter_counter = Some(p.trigger_reset.as_ref().unwrap_or(&p.counter).value);
        Ok(p)
    }
    /// Commit an independently simulated completed battle. The route must still
    /// be at that battle's entry state. A failed/unfinished result is not applied.
    /// This consumes the battle's exit callbacks once; never reset again here.
    pub fn finish_battle(&mut self, battle_entry: u32, state: &BattleState) -> Result<()> {
        ensure(
            self.rng.state == battle_entry,
            "Battle entry does not match the current route state",
        )?;
        ensure(state.outcome != Outcome::Ongoing, "Battle has not finished")?;
        ensure(
            self.rng.draws.checked_add(state.rng.draws).is_some(),
            "Route advance count overflow",
        )?;
        let mut next = self.rng;
        next.advance(state.rng.draws);
        ensure(
            next.state == state.rng.state,
            "Battle result is inconsistent with its entry CRT state",
        )?;
        self.rng = next;
        // A battle with no supported exit callback leaves the counter unknown.
        self.encounter_counter = state
            .opening_crt
            .as_ref()
            .and_then(|c| c.after_battle.as_ref())
            .map(|a| a.encounter_steps);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ot2_battle_engine::{Battle, BattleConfig, ItemCatalog, routine::ThroneOpeningRoutine};
    #[test]
    fn ambush_preserves_the_menu_draw_and_both_separate_resets() {
        let mut arrival = CrtRng::new(0);
        arrival.advance(5086);
        let checkpoint = throne_checkpoint(arrival);
        assert_eq!(checkpoint.after_popup.state, 0x8BBA_D05A);
        let p = throne_ambush(&checkpoint);
        // Independent CRT recurrence / binary32 counter fixture. The menu
        // result is discarded; the final counter must use draws four and five.
        assert_eq!(p.before_menu, checkpoint.after_popup);
        for (rng, draws, state) in [
            (p.at_menu, 5091, 0x6D5C_15B5),
            (p.after_ambush, 5093, 0x3A61_D9D7),
            (p.after_rejoin, 5095, 0x4ABB_8F09),
        ] {
            assert_eq!(rng, CrtRng { state, draws });
        }
        assert_eq!(p.ambush_counter, 764);
        assert_eq!(p.encounter_counter, 760);
        assert_eq!(
            p.trace
                .iter()
                .map(|s| s.after.draws - s.before.draws)
                .collect::<Vec<_>>(),
            [1, 0, 2, 2]
        );
        assert!(p.trace.windows(2).all(|w| w[0].after == w[1].before));
        assert_eq!(p.trace.last().unwrap().after, p.after_rejoin);
        assert!(p.encounters_enabled);
        assert_eq!(p.known_flags["25304"], false);
        assert_eq!(p.known_flags["8004"], true);
        assert_eq!(p.party, ["Throné", "Pirro", "Scaracci"]);
        assert_eq!(p.encounter_actor, "ENT_DNG_CTY_3_1_LND_1");
        // The checkpoint's disabled state is a separate, unchanged boundary.
        assert_eq!(checkpoint.after_popup.draws, 5090);
        assert!(!checkpoint.encounters_enabled);
    }
    #[test]
    fn checkpoint_draws_npc_frames_before_one_counter_reset_and_no_popup_reset() {
        let mut arrival = CrtRng::new(0);
        arrival.advance(5086);
        let p = throne_checkpoint(arrival);
        let mut expected = arrival;
        expected.advance(2);
        let b = expected.index(400);
        let a = expected.index(400);
        assert_eq!(p.arrival, arrival);
        assert_eq!(p.after_popup, expected);
        assert_eq!(p.after_popup.draws, 5090);
        assert_eq!(p.encounter_counter, (1000 + b + a) / 2);
        assert_eq!(
            p.trace
                .iter()
                .map(|s| s.after.draws - s.before.draws)
                .collect::<Vec<_>>(),
            [1, 1, 2, 0]
        );
        assert!(!p.encounters_enabled);
        assert_eq!(p.party, ["Throné"]);
    }
    #[test]
    fn mixed_movement_keeps_a_positive_counter_and_maximizes_running() {
        for counter in (0..=1000).chain([u32::MAX]) {
            let r = throne_reset_reach(counter).unwrap();
            match r.mixed {
                None => assert!(!r.walking.reaches_reset),
                Some(m) => {
                    assert_eq!(m.max_running_updates + m.walking_updates, 57);
                    assert_eq!(m.cost, m.max_running_updates * 16 + m.walking_updates * 12);
                    assert!(m.cost < counter);
                    assert_eq!(m.counter_before_reset, counter - m.cost);
                    assert!(m.max_running_updates == 57 || m.cost + 4 >= counter);
                    let percent = f64::from(m.max_running_updates) * 100.0 / 57.0;
                    assert!(m.max_running_percent <= percent);
                    assert!(percent - m.max_running_percent < 0.1);
                }
            }
        }
        for (counter, run, walk, percent, remaining) in [
            (685, 0, 57, 0.0, 1),
            (688, 0, 57, 0.0, 4),
            (689, 1, 56, 1.7, 1),
            (775, 22, 35, 38.5, 3),
            (831, 36, 21, 63.1, 3),
            (899, 53, 4, 92.9, 3),
            (912, 56, 1, 98.2, 4),
            (913, 57, 0, 100.0, 1),
        ] {
            let m = throne_reset_reach(counter).unwrap().mixed.unwrap();
            assert_eq!(
                (
                    m.max_running_updates,
                    m.walking_updates,
                    m.max_running_percent,
                    m.counter_before_reset
                ),
                (run, walk, percent, remaining)
            );
        }
    }
    #[test]
    fn observed_walk_budget_and_zero_boundary() {
        let r = throne_reset_reach(830).unwrap();
        assert_eq!(r.assumed_updates, 57);
        assert_eq!(
            (
                r.walking.decrement,
                r.walking.cost,
                r.walking.counter_before_reset
            ),
            (12, 684, Some(146))
        );
        assert_eq!(
            (
                r.running.decrement,
                r.running.cost,
                r.running.encounter_on_update
            ),
            (16, 912, Some(52))
        );
        assert_eq!(r.recommendation, "walk");
        for (counter, walk, run) in [
            (0, false, false),
            (684, false, false),
            (685, true, false),
            (899, true, false),
            (912, true, false),
            (913, true, true),
        ] {
            let r = throne_reset_reach(counter).unwrap();
            assert_eq!(
                (r.walking.reaches_reset, r.running.reaches_reset),
                (walk, run),
                "counter {counter}"
            );
            assert_eq!(
                (
                    r.walking.minimum_start_counter,
                    r.running.minimum_start_counter
                ),
                (685, 913)
            );
        }
    }

    #[test]
    fn enemy_miss_still_counts_as_a_turn() {
        let mut assessment = BattleAssessment::default();
        let mut report = StepReport {
            actor: ActorId(3),
            events: vec![ot2_battle_engine::Event::Miss {
                actor: ActorId(3),
                target: ActorId(0),
            }],
            rng_before: CrtRng::new(0),
            rng_after: CrtRng::new(0),
            outcome: Outcome::Ongoing,
        };
        assessment.observe(1, Side::Enemy, &report);
        report.actor = ActorId(0);
        report.outcome = Outcome::Victory;
        assessment.observe(2, Side::Player, &report);
        assert!(!assessment.no_enemy_actions);
        assert_eq!(assessment.enemy_turns.len(), 1);
    }

    #[test]
    fn standalone_battle_result_continues_route_without_extra_reset() {
        let mut route = RouteState::new(0, 4930);
        let entry = route.rng.state;
        let mut battle = Battle::new_opening(
            BattleConfig::throne_opening(entry).unwrap(),
            ItemCatalog::builtin().unwrap(),
            false,
        )
        .unwrap();
        let before = route.clone();
        assert!(throne_post_battle_options(battle.state()).is_err());
        assert!(route.finish_battle(entry, battle.state()).is_err());
        assert_eq!(route, before);
        let mut routine = ThroneOpeningRoutine::default();
        while battle.state().outcome == Outcome::Ongoing {
            routine.step(&mut battle).unwrap();
        }
        route.finish_battle(entry, battle.state()).unwrap();
        assert_eq!(route.rng.draws, 4930 + battle.state().rng.draws);
        assert_eq!(route.rng.state, battle.state().rng.state);
        assert_eq!(
            route.encounter_counter,
            Some(
                battle
                    .state()
                    .opening_crt
                    .as_ref()
                    .unwrap()
                    .after_battle
                    .as_ref()
                    .unwrap()
                    .encounter_steps
            )
        );
        let before = route.clone();
        assert!(route.finish_battle(entry, battle.state()).is_err());
        assert_eq!(route, before);
        let options = throne_post_battle_options(battle.state()).unwrap();
        validate_throne_field_options(&options).unwrap();
        assert_eq!(
            (before.rng.draws, before.rng.state, before.encounter_counter),
            (5083, 0x194611CD, Some(549))
        );
        let p = route.predict_encounter(&options).unwrap();
        assert!(p.counter.supplied);
        assert_eq!(Some(p.counter.value), before.encounter_counter);
        assert_eq!(route.rng.draws, before.rng.draws + p.draws);
        assert_eq!(p.draws, 6); // Reset B/A, situation, 3 party timing draws.
        assert_eq!((p.decrement, p.counter_updates), (Some(12), Some(46)));
        assert!(p.first_override);
        assert_eq!(p.group_roll, None);
        assert_eq!(
            p.encounter_table.as_deref(),
            Some("ENC_DNG_CTY_3_1_LND_DAY_010")
        );
        assert_eq!(p.enemies.len(), 2);
        assert!(
            p.enemies
                .iter()
                .all(|e| e.enemy.id == "ENE_EVE_THI_CTY_010")
        );
        assert_eq!(route.rng, RouteState::new(0, 5089).rng);
    }

    #[test]
    fn later_story_settings_cannot_be_mixed_into_opening_frame() {
        let mut options = throne_field_profile().options.clone();
        options.current_counter = Some(549);
        options.dash = true;
        options.intervening_advances = "7".into();
        validate_throne_field_options(&options).unwrap();
        for altered in [
            PredictionOptions {
                progress: 1,
                ..options.clone()
            },
            PredictionOptions {
                progress: 30,
                ..options.clone()
            },
            PredictionOptions {
                first_encounter: false,
                ..options.clone()
            },
            PredictionOptions {
                party_levels: vec![1],
                ..options.clone()
            },
            PredictionOptions {
                zone: "ENT_EMPTY".into(),
                ..options.clone()
            },
            PredictionOptions {
                boundary: Boundary::Walk,
                ..options.clone()
            },
        ] {
            assert!(validate_throne_field_options(&altered).is_err());
        }
        let p = throne_field_profile();
        assert_eq!(p.story_task_progress, 30);
        assert_eq!(p.options.progress, 0);
        assert_eq!(p.known_flags.get("25304"), Some(&false));
        assert_eq!(p.valid_until_event, "MS_TOU_10_0400");
    }
}
