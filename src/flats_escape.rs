//! Harbor return -> first Flats encounter -> Escape -> Abandoned Village entrance.
//! World/route orchestration only; all battle draws come from the battle crate.
use crate::{
    CrtRng, Result, SimError,
    encounter::{self, Boundary, PredictionOptions},
    ensure,
    highroad::{self, Escape, FieldReturn},
    osvald,
    route::MovementBudget,
    village_return,
    world::WorldState,
};
use ot2_battle_engine::escape::Situation;
use serde::Serialize;

/// Measured ~165 running updates, padded by the user to 170. Starts on returning
/// from harbor and ends at the Abandoned Village transition, not its loaded map.
pub const RUN_UPDATES: u32 = 170;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Settings {
    pub max_boost_repeats: Option<u32>,
    /// None requires the post-village walk; finite values are optional extra repeats.
    pub snow_max_boost_repeats: Option<u32>,
    pub osvald_max_boost_repeats: Option<u32>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            max_boost_repeats: Some(4),
            snow_max_boost_repeats: None,
            osvald_max_boost_repeats: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SnowSearch {
    pub max_extra_boost_repeats: Option<u32>,
    pub base_repeats: u32,
    pub added_repeats: u32,
    pub tested_candidates: u64,
    pub target_met: bool,
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub arrival: CrtRng,
    pub settings: Settings,
    pub level: u32,
    pub encounter: encounter::Prediction,
    pub before_battle_initialization: CrtRng,
    pub run_updates: u32,
    pub first_encounter_update: u32,
    pub remaining_updates: u32,
    pub required_counter: u32,
    pub escape: Option<Escape>,
    pub field_return: Option<FieldReturn>,
    pub remaining_run: Option<MovementBudget>,
    pub at_village_entrance: Option<CrtRng>,
    pub world_at_village_entrance: Option<WorldState>,
    pub village_return: Option<village_return::Prediction>,
    pub village_visit: Option<village_return::Visit>,
    pub snow_search: Option<SnowSearch>,
    pub osvald: Option<osvald::Prediction>,
    pub osvald_search: Option<SnowSearch>,
    pub status: &'static str,
    pub verification: &'static str,
}

/// Field-only selection, shared with the upstream Highroad search. No Escape
/// search is nested inside candidate selection.
pub fn first_encounter(
    arrival: CrtRng,
    world: &WorldState,
    level: u32,
) -> Result<(CrtRng, encounter::Prediction)> {
    ensure(
        world.map == "Fld_Cty_1_2"
            && world.time == 2
            && world.first_encounter
            && world.party == [7]
            && world.visited_maps.contains("Fld_Cty_1_1"),
        "Flats approach requires solo daytime Throné after tagging harbor and returning",
    )?;
    let options = PredictionOptions {
        map: world.map.clone(),
        zone: "ENT_FLD_CTY_1_2_LND".into(),
        period: "DayTime".into(),
        progress: 1,
        first_encounter: true,
        boundary: Boundary::CurrentCounter,
        current_counter: Some(world.encounter_counter),
        party_levels: vec![level],
        dash: true,
        ..Default::default()
    };
    let mut entry = arrival;
    let prediction = encounter::catalog().predict(&mut entry, &options)?;
    Ok((entry, prediction))
}

pub fn after_harbor(
    h: &crate::harbor::Prediction,
    level: u32,
    settings: Settings,
) -> Result<Prediction> {
    predict(h.after, &h.world_after, level, settings)
}

pub fn predict(
    arrival: CrtRng,
    world: &WorldState,
    level: u32,
    settings: Settings,
) -> Result<Prediction> {
    predict_to_boundary(arrival, world, level, settings, false)
}

pub fn until_first_village(arrival:CrtRng, world:&WorldState, level:u32, settings:Settings) -> Result<Prediction> {
    predict_to_boundary(arrival, world, level, settings, true)
}

fn predict_to_boundary(arrival:CrtRng, world:&WorldState, level:u32, settings:Settings, stop_at_village:bool) -> Result<Prediction> {
    ensure(
        settings.max_boost_repeats.is_none_or(|n| n <= 256)
            && settings.snow_max_boost_repeats.is_none_or(|n| n <= 256)
            && settings.osvald_max_boost_repeats.is_none_or(|n| n <= 256),
        "Flats limits must be 0..256, Required outcome or Boost only",
    )?;
    let (entry, encounter) = first_encounter(arrival, world, level)?;
    let trigger = encounter
        .counter_updates
        .ok_or_else(|| SimError("Missing Flats encounter distance".into()))?;
    let decrement = encounter.decrement.unwrap();
    ensure(
        trigger <= RUN_UPDATES,
        "Village approach ends before this encounter",
    )?;
    let remaining = RUN_UPDATES - trigger;
    let required_counter = remaining * decrement + 1;
    let counter = &encounter::catalog().maps[&world.map].counter;
    ensure(
        required_counter <= counter.min + counter.range_a - 1 + counter.range_b - 1,
        "Remaining village run exceeds the maximum reset counter",
    )?;
    let situation = match encounter.situation.as_deref() {
        Some("Normal") => Situation::Normal,
        Some("Preemptive") => Situation::Preemptive,
        Some("Surprise") => Situation::Surprise,
        _ => return Err(SimError("Unknown Flats encounter situation".into())),
    };
    let mut result = Prediction {
        arrival,
        settings,
        level,
        encounter,
        before_battle_initialization: entry,
        run_updates: RUN_UPDATES,
        first_encounter_update: trigger,
        remaining_updates: remaining,
        required_counter,
        escape: None,
        field_return: None,
        remaining_run: None,
        at_village_entrance: None,
        world_at_village_entrance: None,
        village_return: None,
        village_visit: None,
        snow_search: None,
        osvald: None,
        osvald_search: None,
        status: "Surprise opening: increase the Highroad Flats-approach allowance",
        verification: "Source-derived, awaiting live verification. 170 running updates from harbor return to Abandoned Village entrance (measured ~165 plus five-update allowance). Daytime, unchanged equipment, solo route level, no intervening CRT actions. Search direct Escape within the limit, then actual immediate single-enemy break, otherwise unlimited boosts. Every selected strategy must also leave a strictly positive counter at the village entrance. A break whose reset is too short is rejected. Uses the shared field-restoration callbacks; never just offsets the final RNG. The village-approach boundary is before loading; the separate village_return then tags the village, returns to Flats and predicts the padded walk to Southern Cape Cold Snows.",
    };
    if situation == Situation::Surprise {
        return Ok(result);
    }
    let volume = &encounter::catalog().volumes[result.encounter.volume.as_ref().unwrap()];
    let danger = volume
        .encounters
        .iter()
        .rev()
        .find(|r| r.table != "None" && r.progress <= 1)
        .ok_or_else(|| SimError("Missing Flats danger row".into()))?
        .danger;
    let enemies: Vec<_> = result
        .encounter
        .enemies
        .iter()
        .map(|e| e.enemy.id.as_str())
        .collect();
    let restore = |attempt: &ot2_battle_engine::field_escape::FieldEscapeAttempt| {
        highroad::restore_field(world, attempt, "Fld_Cty_1_2", "Flats Escape")
    };
    let project = |attempt: &ot2_battle_engine::field_escape::FieldEscapeAttempt| -> Result<Option<village_return::Prediction>> {
        let field = restore(attempt)?;
        if field.world_after.encounter_counter < required_counter { return Ok(None); }
        let mut at_exit = field.world_after;
        at_exit.encounter_counter -= remaining * decrement;
        village_return::with_world_state(field.after, at_exit).map(Some)
    };
    let choose = |total_limit, require_snow, require_osvald| {
        highroad::escape_plan_with_limit(
            entry,
            situation,
            level,
            &enemies,
            danger,
            settings.max_boost_repeats,
            total_limit,
            |a| {
                if require_snow || require_osvald {
                    let Some(p) = project(a)? else {
                        return Ok(false);
                    };
                    if !p.snow_reach.walking.reaches_reset {
                        return Ok(false);
                    }
                    if require_osvald {
                        Ok(osvald::after_village(&p)?.is_some_and(|o| o.walking.reaches_reset))
                    } else {
                        Ok(true)
                    }
                } else {
                    Ok(restore(a)?.world_after.encounter_counter >= required_counter)
                }
            },
        )
    };
    // Required goals gate both direct Escape and break strategies. Optional
    // walking starts from the actual escape/village-run plan, never its cap.
    let require_osvald = !stop_at_village && settings.osvald_max_boost_repeats.is_none();
    let require_snow = !stop_at_village && (settings.snow_max_boost_repeats.is_none() || require_osvald);
    let mut escape = choose(None, require_snow, require_osvald)?;
    if stop_at_village {
        result.verification="Source-derived route through first Abandoned Village entry/tag and tutorial dismissal. Search only first Escape and a safe village run; no lookahead toward Snows, Osvald or subsequent events. Shared Loader evaluates the village load and carries CRT, counters, progression and residency. No return to Flats is simulated.";
        if let Some(attempt) = &escape.attempt {
            let field = restore(attempt)?;
            let budget = MovementBudget::new(field.world_after.encounter_counter,remaining,decrement);
            ensure(budget.reaches_reset,"Selected Flats plan cannot reach the village")?;
            let mut at_exit=field.world_after.clone();
            at_exit.encounter_counter=budget.counter_before_reset.unwrap();
            result.village_visit=Some(village_return::visit(field.after,at_exit.clone())?);
            result.at_village_entrance=Some(field.after);
            result.world_at_village_entrance=Some(at_exit);
            result.remaining_run=Some(budget);
            result.field_return=Some(field);
            result.status="First Abandoned Village visit complete; route stops here";
        }
        result.escape=Some(escape);
        return Ok(result);
    }
    if let Some(attempt) = &escape.attempt {
        let base = escape.inputs.as_ref().map_or(0, |i| i.extra_repeats);
        let mut met = project(attempt)?.unwrap().snow_reach.walking.reaches_reset;
        let mut search = SnowSearch {
            max_extra_boost_repeats: settings.snow_max_boost_repeats,
            base_repeats: base,
            added_repeats: 0,
            tested_candidates: escape.tested_candidates,
            target_met: met,
            status: "Escape, village run and post-village walk satisfied",
        };
        if !met && let Some(extra) = settings.snow_max_boost_repeats {
            let candidate = choose(Some(base.saturating_add(extra)), true, require_osvald)?;
            search.tested_candidates += candidate.tested_candidates;
            if candidate.attempt.is_some() {
                search.added_repeats = candidate
                    .inputs
                    .as_ref()
                    .map_or(0, |i| i.extra_repeats)
                    .saturating_sub(base);
                escape = candidate;
                met = true;
            } else {
                search.status = "Optional post-village walk not found within budget; keeping Escape and village run";
            }
        }
        search.target_met = met;
        result.snow_search = Some(search);
    }
    if let Some(attempt) = &escape.attempt {
        let base = escape.inputs.as_ref().map_or(0, |i| i.extra_repeats);
        let village = project(attempt)?.unwrap();
        let keep_snow = require_snow || village.snow_reach.walking.reaches_reset;
        let mut met = osvald::after_village(&village)?.is_some_and(|o| o.walking.reaches_reset);
        let mut search = SnowSearch {
            max_extra_boost_repeats: settings.osvald_max_boost_repeats,
            base_repeats: base,
            added_repeats: 0,
            tested_candidates: escape.tested_candidates,
            target_met: met,
            status: "Safe village run and both walks through Osvald satisfied",
        };
        if !met && let Some(extra) = settings.osvald_max_boost_repeats {
            let candidate = choose(Some(base.saturating_add(extra)), keep_snow, true)?;
            search.tested_candidates += candidate.tested_candidates;
            if candidate.attempt.is_some() {
                search.added_repeats = candidate
                    .inputs
                    .as_ref()
                    .map_or(0, |i| i.extra_repeats)
                    .saturating_sub(base);
                escape = candidate;
                met = true;
            } else {
                search.status =
                    "Optional Osvald walk not found within budget; keeping the preceding plan";
            }
        }
        search.target_met = met;
        result.osvald_search = Some(search);
    }
    if let Some(attempt) = &escape.attempt {
        let field = restore(attempt)?;
        let budget = MovementBudget::new(field.world_after.encounter_counter, remaining, decrement);
        ensure(
            budget.reaches_reset,
            "Selected Flats plan cannot reach the village",
        )?;
        let mut world_at_exit = field.world_after.clone();
        world_at_exit.encounter_counter = budget.counter_before_reset.unwrap();
        result.at_village_entrance = Some(field.after);
        result.world_at_village_entrance = Some(world_at_exit);
        result.village_return = project(attempt)?;
        result.osvald = osvald::after_village(result.village_return.as_ref().unwrap())?;
        if let Some(s) = &mut result.snow_search {
            s.target_met = result
                .village_return
                .as_ref()
                .unwrap()
                .snow_reach
                .walking
                .reaches_reset;
            if s.target_met {
                s.status = "Escape, village run and post-village walk satisfied";
            }
        }
        result.remaining_run = Some(budget);
        result.field_return = Some(field);
        result.status = "Escape succeeds; run to Abandoned Village without a second encounter";
    } else {
        result.status = "No joint Escape/run solution in the CRT cycle";
    }
    result.escape = Some(escape);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world;
    use highroad::EscapeMethod;
    #[test]
    fn village_fork_stops_at_tag_and_ignores_all_later_search_targets() {
        let w = world();
        let mut covered = 0;
        for seed in 0..24 {
            let entry = CrtRng { state: seed, draws: 6050 };
            let settings = Settings { max_boost_repeats: Some(4),
                snow_max_boost_repeats: Some(0), osvald_max_boost_repeats: Some(0) };
            let p = until_first_village(entry, &w, 5, settings).unwrap();
            let required = until_first_village(entry, &w, 5, Settings {
                snow_max_boost_repeats: None, osvald_max_boost_repeats: None, ..settings
            }).unwrap();
            assert!(p.village_return.is_none() && p.osvald.is_none());
            assert!(p.snow_search.is_none() && p.osvald_search.is_none());
            assert_eq!(serde_json::to_value(&p.escape).unwrap(), serde_json::to_value(&required.escape).unwrap());
            let Some(v) = &p.village_visit else {
                assert_eq!(p.encounter.situation.as_deref(), Some("Surprise"));
                continue;
            };
            covered += 1;
            assert!(p.escape.as_ref().unwrap().attempt.as_ref().unwrap().success);
            assert!(p.remaining_run.as_ref().unwrap().reaches_reset);
            assert_eq!(v.world_after.map, village_return::VILLAGE);
            assert!(v.world_after.visited_maps.contains(village_return::VILLAGE));
            assert_eq!(v.world_after.party, [7]);
            let mut loader = world::Loader::new(world::catalog(),
                p.world_at_village_entrance.clone().unwrap(), p.at_village_entrance.unwrap()).unwrap();
            loader.visit_map(village_return::VILLAGE).unwrap();
            loader.dismiss_tutorial(62).unwrap();
            assert_eq!(loader.rng, v.after);
            assert_eq!(serde_json::to_value(loader.state).unwrap(), serde_json::to_value(&v.world_after).unwrap());
            assert_eq!(serde_json::to_value(v).unwrap(), serde_json::to_value(required.village_visit.unwrap()).unwrap());
            // The full route still uses precisely this arrival before its return load.
            let full = predict(entry, &w, 5, settings).unwrap();
            let old = full.village_return.unwrap();
            assert_eq!(old.at_village, v.after);
            assert_eq!(serde_json::to_value(old.world_at_village).unwrap(), serde_json::to_value(&v.world_after).unwrap());
        }
        assert!(covered > 0);
    }
    fn world() -> WorldState {
        let mut w = world::throne_after_second_state().unwrap();
        w.map = "Fld_Cty_1_2".into();
        w.time = 2;
        w.party = vec![7];
        w.first_encounter = true;
        w.active_scenarios.clear();
        w.prologues.clear();
        w.story_types.clear();
        w.encounter_counter = 515;
        w.visited_maps.insert("Fld_Cty_1_1".into());
        w.resident_levels = world::catalog().maps[&w.map]
            .streams
            .iter()
            .cloned()
            .collect();
        w
    }
    #[test]
    fn distance_and_fallbacks_replay_without_mutating_arrival() {
        let w = world();
        let original = serde_json::to_value(&w).unwrap();
        let mut covered = [false; 4];
        for seed in 0..160 {
            let rng = CrtRng {
                state: seed,
                draws: 6050,
            };
            let p = predict(
                rng,
                &w,
                5,
                Settings {
                    max_boost_repeats: Some(0),
                    snow_max_boost_repeats: Some(0),
                    osvald_max_boost_repeats: Some(0),
                },
            )
            .unwrap();
            assert_eq!(
                (
                    p.first_encounter_update,
                    p.remaining_updates,
                    p.required_counter
                ),
                (43, 127, 1525)
            );
            if p.encounter.situation.as_deref() == Some("Surprise") {
                covered[3] = true;
                assert!(
                    p.escape.is_none()
                        && p.field_return.is_none()
                        && p.at_village_entrance.is_none()
                );
                continue;
            }
            let s = p.escape.as_ref().unwrap();
            match s.method {
                EscapeMethod::WithinLimit => covered[0] = true,
                EscapeMethod::BreakThenEscape => covered[1] = true,
                EscapeMethod::UnlimitedFallback => covered[2] = true,
                _ => panic!("Unexpected method"),
            }
            let r = p.field_return.as_ref().unwrap();
            assert!(s.attempt.as_ref().unwrap().success);
            assert_eq!(r.arrival, s.attempt.as_ref().unwrap().after);
            assert!(r.world_after.encounter_counter >= 1525);
            assert!(!r.world_after.first_encounter);
            assert_eq!(r.after.draws - r.arrival.draws, 2); // Flats has no gondolas.
            assert_eq!(p.at_village_entrance, Some(r.after));
            assert_eq!(
                p.remaining_run.as_ref().unwrap().counter_before_reset,
                Some(r.world_after.encounter_counter - 1524)
            );
            assert_eq!(
                p.world_at_village_entrance
                    .as_ref()
                    .unwrap()
                    .encounter_counter,
                r.world_after.encounter_counter - 1524
            );
            if let Some(inputs) = &s.inputs {
                let replay = s.menu.clone().attempt(&inputs.levels, 5).unwrap();
                assert_eq!(replay.after, s.attempt.as_ref().unwrap().after);
                // Ascending repeat search must reject all cheaper direct paths.
                for n in 0..inputs.extra_repeats {
                    let a = s
                        .menu
                        .clone()
                        .attempt(&(0..n).flat_map(|_| [1, 0]).collect::<Vec<_>>(), 5)
                        .unwrap();
                    if a.success {
                        let f = highroad::restore_field(&w, &a, "Fld_Cty_1_2", "test").unwrap();
                        assert!(f.world_after.encounter_counter < p.required_counter);
                    }
                }
            } else {
                assert_eq!(s.break_action.as_ref().unwrap().at_attack_menu, s.menu.rng);
                assert_eq!(s.attempt.as_ref().unwrap().chance_percent, 100);
            }
        }
        assert_eq!(covered, [true; 4]);
        assert_eq!(serde_json::to_value(&w).unwrap(), original);
    }

    #[test]
    fn village_walk_search_preserves_escape_and_run_and_replays_both_map_loads() {
        let w = world();
        let mut changed = 0;
        for i in 0..24 {
            let mut entry = CrtRng::new(0);
            entry.advance(i * 7919);
            let baseline = predict(
                entry,
                &w,
                5,
                Settings {
                    max_boost_repeats: None,
                    snow_max_boost_repeats: Some(0),
                    osvald_max_boost_repeats: Some(0),
                },
            )
            .unwrap();
            if baseline.escape.is_none() {
                continue;
            }
            let p = predict(
                entry,
                &w,
                5,
                Settings {
                    max_boost_repeats: None,
                    snow_max_boost_repeats: None,
                    osvald_max_boost_repeats: Some(0),
                },
            )
            .unwrap();
            assert_eq!(
                p.before_battle_initialization,
                baseline.before_battle_initialization
            );
            let selected = p.escape.as_ref().unwrap();
            let v = p.village_return.as_ref().unwrap();
            assert!(selected.attempt.as_ref().unwrap().success);
            assert!(p.remaining_run.as_ref().unwrap().reaches_reset);
            assert!(v.snow_reach.walking.reaches_reset);
            assert!(v.world_after.first_encounter);
            assert!((500..=899).contains(&v.world_after.encounter_counter));
            assert!(
                v.world_at_village
                    .visited_maps
                    .contains(village_return::VILLAGE)
            );
            let mut loader = crate::world::Loader::new(
                crate::world::catalog(),
                p.world_at_village_entrance.clone().unwrap(),
                p.at_village_entrance.unwrap(),
            )
            .unwrap();
            loader.visit_map(village_return::VILLAGE).unwrap();
            loader.dismiss_tutorial(62).unwrap();
            assert_eq!(loader.rng, v.at_village);
            loader.visit_map("Fld_Cty_1_2").unwrap();
            assert_eq!(loader.rng, v.after);
            assert_eq!(
                serde_json::to_value(loader.state).unwrap(),
                serde_json::to_value(&v.world_after).unwrap()
            );
            let mut previous = v.arrival;
            for trace in &v.trace {
                assert_eq!(trace.before, previous);
                previous = trace.after;
            }
            assert_eq!(previous, v.after);
            if !baseline
                .village_return
                .as_ref()
                .unwrap()
                .snow_reach
                .walking
                .reaches_reset
            {
                changed += 1;
                let old = baseline
                    .escape
                    .as_ref()
                    .unwrap()
                    .inputs
                    .as_ref()
                    .unwrap()
                    .extra_repeats;
                let n = selected.inputs.as_ref().unwrap().extra_repeats;
                assert!(n > old);
                // Every cheaper successful direct Escape must fail one route condition.
                for repeats in 0..n {
                    let a = selected
                        .menu
                        .clone()
                        .attempt(&(0..repeats).flat_map(|_| [1, 0]).collect::<Vec<_>>(), 5)
                        .unwrap();
                    if !a.success {
                        continue;
                    }
                    let f = highroad::restore_field(&w, &a, "Fld_Cty_1_2", "test").unwrap();
                    if f.world_after.encounter_counter < p.required_counter {
                        continue;
                    }
                    let mut at_exit = f.world_after;
                    at_exit.encounter_counter -= p.remaining_updates * 12;
                    assert!(
                        !village_return::with_world_state(f.after, at_exit)
                            .unwrap()
                            .snow_reach
                            .walking
                            .reaches_reset
                    );
                }
                let enough = predict(
                    entry,
                    &w,
                    5,
                    Settings {
                        max_boost_repeats: None,
                        snow_max_boost_repeats: Some(n - old),
                        osvald_max_boost_repeats: Some(0),
                    },
                )
                .unwrap();
                assert_eq!(enough.village_return.unwrap().after, v.after);
                let short = predict(
                    entry,
                    &w,
                    5,
                    Settings {
                        max_boost_repeats: None,
                        snow_max_boost_repeats: Some(n - old - 1),
                        osvald_max_boost_repeats: Some(0),
                    },
                )
                .unwrap();
                assert!(!short.snow_search.unwrap().target_met);
                assert_eq!(
                    short.field_return.unwrap().after,
                    baseline.field_return.unwrap().after
                );
            }
        }
        assert!(
            changed > 0,
            "Must exercise an adjustment for the newly added target"
        );
    }

    #[test]
    fn required_osvald_walk_includes_snow_prerequisite_and_optional_cap_keeps_plan() {
        let w = world();
        let mut changed = 0;
        for i in 0..32 {
            let mut entry = CrtRng::new(0);
            entry.advance(i * 7919);
            let settings = Settings {
                max_boost_repeats: None,
                snow_max_boost_repeats: Some(0),
                osvald_max_boost_repeats: Some(0),
            };
            let baseline = predict(entry, &w, 5, settings).unwrap();
            let Some(old_escape) = &baseline.escape else {
                continue;
            };
            let old = old_escape.inputs.as_ref().unwrap().extra_repeats;
            let p = predict(
                entry,
                &w,
                5,
                Settings {
                    osvald_max_boost_repeats: None,
                    ..settings
                },
            )
            .unwrap();
            assert!(
                p.village_return
                    .as_ref()
                    .unwrap()
                    .snow_reach
                    .walking
                    .reaches_reset
            );
            let o = p.osvald.as_ref().unwrap();
            assert!(o.walking.reaches_reset && o.after_recruitment.is_some());
            let selected = p.escape.as_ref().unwrap();
            let n = selected.inputs.as_ref().unwrap().extra_repeats;
            if n == old {
                continue;
            }
            changed += 1;
            assert!(n > old);
            for repeats in old..n {
                let a = selected
                    .menu
                    .clone()
                    .attempt(&(0..repeats).flat_map(|_| [1, 0]).collect::<Vec<_>>(), 5)
                    .unwrap();
                if !a.success {
                    continue;
                }
                let f = highroad::restore_field(&w, &a, "Fld_Cty_1_2", "test").unwrap();
                if f.world_after.encounter_counter < p.required_counter {
                    continue;
                }
                let mut exit = f.world_after;
                exit.encounter_counter -= p.remaining_updates * 12;
                let v = village_return::with_world_state(f.after, exit).unwrap();
                assert!(
                    !osvald::after_village(&v)
                        .unwrap()
                        .is_some_and(|o| o.walking.reaches_reset)
                );
            }
            let enough = predict(
                entry,
                &w,
                5,
                Settings {
                    osvald_max_boost_repeats: Some(n - old),
                    ..settings
                },
            )
            .unwrap();
            assert_eq!(
                enough.osvald.unwrap().after_recruitment,
                o.after_recruitment
            );
            let short = predict(
                entry,
                &w,
                5,
                Settings {
                    osvald_max_boost_repeats: Some(n - old - 1),
                    ..settings
                },
            )
            .unwrap();
            assert!(!short.osvald_search.unwrap().target_met);
            assert_eq!(
                short.field_return.unwrap().after,
                baseline.field_return.unwrap().after
            );
        }
        assert!(changed > 0);
        // Break preference must meet both walks too, rather than bypass them.
        let mut breaks = 0;
        for seed in 0..64 {
            let p = predict(
                CrtRng::new(seed),
                &w,
                5,
                Settings {
                    max_boost_repeats: Some(0),
                    ..Default::default()
                },
            )
            .unwrap();
            if let Some(s) = p.escape {
                assert!(p.osvald.unwrap().after_recruitment.is_some());
                if matches!(s.method, EscapeMethod::BreakThenEscape) {
                    breaks += 1;
                }
            }
        }
        assert!(breaks > 0);
    }

    #[test]
    fn distances_use_update_ceil_and_keep_positive_remainder() {
        let mut w = world();
        for (counter, trigger, required) in [
            (504, 42, 1537),
            (505, 43, 1525),
            (516, 43, 1525),
            (517, 44, 1513),
            (899, 75, 1141),
        ] {
            w.encounter_counter = counter;
            let p = predict(CrtRng::new(7), &w, 5, Settings::default()).unwrap();
            assert_eq!(
                (p.first_encounter_update, p.required_counter),
                (trigger, required)
            );
        }
        w.time = 4;
        assert!(predict(CrtRng::new(0), &w, 5, Settings::default()).is_err());
    }
}
