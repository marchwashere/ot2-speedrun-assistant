use ot2_battle_engine::{BattleConfig, Outcome};
use ot2_speedrun_assistant::{
    CrtRng, city, mansion, mansion_escape, parlor, parlor_revisit, pirro,
};

fn approach() -> mansion_escape::Prediction {
    let mut r = CrtRng::new(0);
    r.advance(5264);
    let city = city::after_second_victory(r).unwrap();
    let p = parlor::after_city(&city).unwrap();
    let p = parlor_revisit::after_first_visit(&p).unwrap();
    let m = mansion::after_parlor_revisit(&p).unwrap();
    let resources = BattleConfig::throne_pirro(0).unwrap().players[0]
        .resources
        .clone()
        .unwrap();
    mansion_escape::predict(
        m.lookout_ambush.after,
        &m.lookout_ambush.world_after,
        resources,
        Default::default(),
    )
    .unwrap()
}
#[test]
fn serial_boss_entry_preserves_escape_hp_sp_inventory_counter_and_crt() {
    let approach = approach();
    let s = approach.solution.unwrap();
    let mut resources = s.attempt.state.actors[0].resources.clone();
    resources.hp = 240;
    resources.sp = 29;
    let mut inventory = BattleConfig::throne_pirro(0).unwrap().inventory;
    inventory.insert("healing_grape".into(), 3);
    let p = pirro::predict(
        s.after_field_return,
        &s.world_at_trigger,
        resources,
        inventory,
    )
    .unwrap();
    assert_eq!(p.entry, s.after_field_return);
    assert_eq!(p.entry.draws, 5595);
    assert_eq!(p.entry.draws + p.initial_state.rng.draws, 5631);
    assert_eq!(p.initial_state.actors[0].resources.hp, 240);
    assert_eq!(p.initial_state.actors[0].resources.sp, 29);
    assert_eq!(p.initial_state.actors[0].resources.bp, 1);
    assert_eq!(p.initial_state.inventory[0], 3);
    assert!(p.world_at_entry.flag(8017));
    assert!(!s.world_at_trigger.flag(8017));
    assert_eq!(
        p.world_at_entry.encounter_counter,
        s.world_at_trigger.encounter_counter
    );
    let mut r = p.entry;
    r.advance(p.state.rng.draws);
    assert_eq!(r, p.after);
    assert!(p.routine_error.is_none());
    assert_eq!(p.state.outcome, Outcome::Victory);
    assert_eq!(p.after_victory, Some(p.after));
    assert!(p.routine.latent_delayed);
    let latent = p
        .steps
        .iter()
        .find(|s| {
            s.report
                .events
                .iter()
                .any(|e| matches!(e, ot2_battle_engine::Event::LatentActivated { .. }))
        })
        .unwrap();
    assert_eq!(latent.round, 9);
    assert!(latent.report.events.iter().any(
        |e| matches!(e, ot2_battle_engine::Event::Action { name, .. } if name=="wind_soulstone")
    ));
    assert_eq!(
        p.steps
            .iter()
            .filter(|s| s
                .report
                .events
                .iter()
                .any(|e| matches!(e, ot2_battle_engine::Event::LatentActivated { .. })))
            .count(),
        1
    );
}
#[test]
fn missing_stolen_item_stops_without_fabricating_victory() {
    let a = approach();
    let s = a.solution.unwrap();
    let mut inventory = BattleConfig::throne_pirro(0).unwrap().inventory;
    inventory.remove("shadow_soulstone");
    let p = pirro::predict(
        s.after_field_return,
        &s.world_at_trigger,
        s.attempt.state.actors[0].resources.clone(),
        inventory,
    )
    .unwrap();
    assert_eq!(p.state.round, 6);
    assert!(p.routine_error.is_some());
    assert_eq!(p.after_victory, None);
    assert!(
        ot2_speedrun_assistant::pirro_aftermath::after_battle(&p)
            .unwrap()
            .is_none()
    );
}
#[test]
fn authored_1800_handoff_starts_battle_before_event_cleanup() {
    let a: serde_json::Value =
        serde_json::from_str(include_str!("../data/source/throne-pirro-transition.json")).unwrap();
    assert_eq!(a["event"]["EncountReset"], true);
    assert_eq!(a["expected_crt_draws"], 0);
    assert!(
        a["start_battle"]["body"]
            .as_str()
            .unwrap()
            .contains("EVM_BOS_THI_C01_010")
    );
    assert!(
        a["native"]["skipGuard"]
            .as_str()
            .unwrap()
            .contains("return 0;")
    );
}

#[test]
fn r3_search_repairs_defeat_and_heal_branches_and_replays_exactly() {
    let s = approach().solution.unwrap();
    let config = BattleConfig::throne_pirro(0).unwrap();
    let resources = config.players[0].resources.clone().unwrap();
    let mut adjusted = 0;
    for seed in [
        12,
        115,
        353,
        0xAA7C3306,
        0x1F8FF44E,
        0x3E984E25,
        0,
        1,
        2,
        3,
        4,
        5,
        s.after_field_return.state,
    ] {
        let entry = CrtRng {
            state: seed,
            draws: 5595,
        };
        let baseline = pirro::predict(
            entry,
            &s.world_at_trigger,
            resources.clone(),
            config.inventory.clone(),
        )
        .unwrap();
        let ending = ot2_speedrun_assistant::pirro_aftermath::after_battle(&baseline).unwrap();
        assert_eq!(
            ending.is_some(),
            baseline.state.outcome == Outcome::Victory && baseline.routine_error.is_none()
        );
        if let Some(ending) = ending {
            assert_eq!(Some(ending.arrival), baseline.after_victory);
            assert!(ending.after.draws > ending.arrival.draws);
        }
        let prediction = pirro::predict_with_search(
            entry,
            &s.world_at_trigger,
            resources.clone(),
            config.inventory.clone(),
            Default::default(),
            None,
        )
        .unwrap();
        let search = prediction.boost_search.as_ref().unwrap();
        assert!(search.target_met, "seed {seed}: {}", search.status);
        assert_eq!(prediction.state.outcome, Outcome::Victory);
        assert_eq!(prediction.routine.heal_before_round_eight, Some(false));
        assert_eq!(
            prediction.state.inventory[0],
            prediction.initial_state.inventory[0]
        );
        assert_eq!(prediction.entry, baseline.entry);
        assert_eq!(prediction.initial_state, baseline.initial_state);
        let prefix = |p: &pirro::Prediction| {
            p.steps
                .iter()
                .take_while(|step| step.round < 3 || step.report.actor.0 != 0)
                .map(|s| &s.report)
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(prefix(&prediction), prefix(&baseline));
        if let Some(m) = &prediction.manipulation {
            adjusted += 1;
            assert_eq!(m.round, 3);
            assert_eq!(m.round_menu, 1);
            assert_eq!(m.inputs.levels.last(), Some(&3));
            assert!(m.inputs.extra_repeats <= search.no_grape_total_repeat_cap.unwrap());
            let boost_trace = prediction
                .state
                .opening_crt
                .as_ref()
                .unwrap()
                .trace
                .as_ref()
                .unwrap();
            let first_action = boost_trace
                .iter()
                .find(|t| t.reason == "ability_voice" && t.before.draws >= m.rng_before.draws)
                .unwrap();
            assert_eq!(m.rng_after_inputs, first_action.before);
            // Enumerate button paths independently: fixed 1/2/3-draw offset
            // optimization is invalid for Pirro's conditional max-boost voice.
            let mut pending = vec![(Vec::<u8>::new(), 0u8, false, 0u32)];
            while let Some((path, bp, rising, climbs)) = pending.pop() {
                if bp == 3 && path != [1, 2, 3] {
                    let trial = pirro::predict_with_plan(
                        entry,
                        &s.world_at_trigger,
                        resources.clone(),
                        config.inventory.clone(),
                        &path,
                    )
                    .unwrap();
                    let cost = (
                        path.len(),
                        trial.manipulation.as_ref().unwrap().rng_after_inputs.draws,
                    );
                    if cost < (m.inputs.input_count, m.rng_after_inputs.draws) {
                        assert!(
                            !(trial.state.outcome == Outcome::Victory
                                && trial.routine.heal_before_round_eight == Some(false))
                        );
                    }
                }
                if path.len() >= m.inputs.input_count {
                    continue;
                }
                for next in [bp.checked_add(1).filter(|&b| b <= 3), bp.checked_sub(1)]
                    .into_iter()
                    .flatten()
                {
                    let up = next > bp;
                    let next_climbs = climbs + u32::from(up && !rising);
                    if next_climbs > search.no_grape_total_repeat_cap.unwrap() + 1 {
                        continue;
                    }
                    let mut next_path = path.clone();
                    next_path.push(next);
                    pending.push((next_path, next, up, next_climbs));
                }
            }
        } else {
            assert_eq!(baseline.routine.heal_before_round_eight, Some(false));
            assert_eq!(prediction.state, baseline.state);
        }
        assert_eq!(
            prediction
                .steps
                .iter()
                .find(|s| s.round == 8)
                .unwrap()
                .report
                .actor
                .0,
            0
        );
        let replay = pirro::predict_with_search(
            entry,
            &s.world_at_trigger,
            resources.clone(),
            config.inventory.clone(),
            Default::default(),
            Some(search.plan.clone()),
        )
        .unwrap();
        assert_eq!(prediction.state, replay.state);
        assert_eq!(prediction.after, replay.after);
        assert_eq!(
            serde_json::to_value(&prediction.steps).unwrap(),
            serde_json::to_value(&replay.steps).unwrap()
        );
        let mut rng = entry;
        rng.advance(prediction.state.rng.draws);
        assert_eq!(rng, prediction.after);
    }
    assert!(adjusted > 0);
}

#[test]
fn r3_search_reports_zero_cap_failure_and_unlimited_recovers() {
    let s = approach().solution.unwrap();
    let config = BattleConfig::throne_pirro(12).unwrap();
    let run = |cap, inventory| {
        pirro::predict_with_search(
            CrtRng::new(12),
            &s.world_at_trigger,
            config.players[0].resources.clone().unwrap(),
            inventory,
            pirro::Settings {
                victory_max_boost_repeats: cap,
                no_grape_max_boost_repeats: cap,
            },
            None,
        )
        .unwrap()
    };
    let capped = run(Some(0), config.inventory.clone());
    assert_eq!(capped.state.outcome, Outcome::Defeat);
    assert!(capped.boost_search.as_ref().unwrap().limit_reached);
    assert_eq!(capped.boost_search.as_ref().unwrap().tested_candidates, 0);
    let unlimited = run(None, config.inventory.clone());
    assert!(unlimited.boost_search.as_ref().unwrap().target_met);
    assert_eq!(
        unlimited
            .boost_search
            .as_ref()
            .unwrap()
            .settings
            .no_grape_max_boost_repeats,
        None
    );
    let mut missing_item = config.inventory;
    missing_item.remove("shadow_soulstone");
    let blocked = run(None, missing_item);
    assert!(blocked.routine_error.is_some());
    assert_eq!(blocked.boost_search.as_ref().unwrap().tested_candidates, 0);
    assert!(!blocked.boost_search.as_ref().unwrap().target_met);
}

#[test]
fn victory_and_no_grape_budgets_are_independent_and_preserve_a_winning_fallback() {
    let s = approach().solution.unwrap();
    let config = BattleConfig::throne_pirro(0).unwrap();
    let run = |seed, victory, no_grape| {
        pirro::predict_with_search(
            CrtRng::new(seed),
            &s.world_at_trigger,
            config.players[0].resources.clone().unwrap(),
            config.inventory.clone(),
            pirro::Settings {
                victory_max_boost_repeats: victory,
                no_grape_max_boost_repeats: no_grape,
            },
            None,
        )
        .unwrap()
    };
    // Required no-grape includes victory, even when optional victory gets zero.
    let required = run(12, Some(0), None);
    let search = required.boost_search.as_ref().unwrap();
    assert_eq!(required.state.outcome, Outcome::Victory);
    assert_eq!(required.routine.heal_before_round_eight, Some(false));
    assert!(!search.victory_limit_reached);
    assert_eq!(search.victory_tested_candidates, 0);
    assert!(search.no_grape_tested_candidates > 0);

    // This entry already wins with a grape. An unused Unlimited victory
    // allowance must not expand the zero no-grape allowance.
    let fallback = run(0, None, Some(0));
    let search = fallback.boost_search.as_ref().unwrap();
    assert_eq!(fallback.state.outcome, Outcome::Victory);
    assert_eq!(fallback.routine.heal_before_round_eight, Some(true));
    assert_eq!(search.victory_repeats, 0);
    assert_eq!(search.no_grape_total_repeat_cap, Some(0));
    assert!(search.no_grape_limit_reached);
    assert!(!search.victory_limit_reached);
    assert_eq!(search.plan, search.victory_plan);

    // Inverting the limits allows the no-grape adjustment without searching
    // for victory again, since the baseline already wins.
    let adjusted = run(0, Some(0), None);
    let search = adjusted.boost_search.as_ref().unwrap();
    assert!(search.target_met);
    assert_eq!(search.victory_tested_candidates, 0);
    assert_eq!(search.no_grape_total_repeat_cap, None);
    assert!(search.no_grape_tested_candidates > 0);

    let victory_only = run(12, Some(1), Some(0));
    let combined = run(12, Some(100), Some(2));
    let first = victory_only.boost_search.as_ref().unwrap();
    let second = combined.boost_search.as_ref().unwrap();
    assert_eq!(victory_only.state.outcome, Outcome::Victory);
    assert_eq!(first.victory_repeats, 1);
    assert_eq!(second.victory_plan, first.victory_plan);
    assert_eq!(second.no_grape_total_repeat_cap, Some(3));
    assert!(second.target_met);
    assert!(combined.manipulation.as_ref().unwrap().inputs.extra_repeats <= 3);

    // Replay validates the joint required outcome, not the optional victory cap.
    let replay = pirro::predict_with_search(
        CrtRng::new(12),
        &s.world_at_trigger,
        config.players[0].resources.clone().unwrap(),
        config.inventory.clone(),
        pirro::Settings {
            victory_max_boost_repeats: Some(0),
            no_grape_max_boost_repeats: None,
        },
        Some(required.boost_search.as_ref().unwrap().plan.clone()),
    )
    .unwrap();
    assert_eq!(replay.state.outcome, Outcome::Victory);
    assert_eq!(replay.routine.heal_before_round_eight, Some(false));
    let invalid_replay = pirro::predict_with_search(
        CrtRng::new(12),
        &s.world_at_trigger,
        config.players[0].resources.clone().unwrap(),
        config.inventory.clone(),
        pirro::Settings {
            victory_max_boost_repeats: Some(0),
            no_grape_max_boost_repeats: None,
        },
        Some(vec![1, 2, 3]),
    );
    assert!(invalid_replay.is_err());
}
