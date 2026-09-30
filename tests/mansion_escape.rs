use ot2_battle_engine::{
    Resources,
    escape::{FirstEscapeBattle, Situation},
};
use ot2_speedrun_assistant::{
    CrtRng, city, mansion,
    mansion_escape::{Settings, predict},
    parlor, parlor_revisit,
};

fn route() -> mansion::Prediction {
    let mut rng = CrtRng::new(0);
    rng.advance(5264);
    let c = city::after_second_victory(rng).unwrap();
    let p = parlor::after_city(&c).unwrap();
    let r = parlor_revisit::after_first_visit(&p).unwrap();
    mansion::after_parlor_revisit(&r).unwrap()
}
fn resources() -> Resources {
    Resources {
        hp: 261,
        sp: 28,
        bp: 1,
        latent: 0,
    }
}
#[test]
fn break_fallback_replays_actual_hits_and_preserves_safe_counter() {
    let r=route();
    let w=&r.lookout_ambush.world_after;
    for seed in 0..100 {
        let p=predict(CrtRng::new(seed),w,resources(),Settings {max_boost_repeats:Some(0),..Default::default()}).unwrap();
        let s=p.solution.as_ref().unwrap();
        if !s.break_then_escape {continue;}
        let situation=match p.first_encounter.situation.as_deref().unwrap() {
            "Normal"=>Situation::Normal,"Preemptive"=>Situation::Preemptive,_=>Situation::Surprise,
        };
        let b=FirstEscapeBattle::mansion(p.before_battle_initialization.state,situation,resources(),true).unwrap();
        let replay=b.break_then_escape(&s.inputs.levels).unwrap().unwrap();
        assert_eq!(replay.after.state,s.after_escape.state);
        assert_eq!(replay.state.actors[0].resources,s.attempt.state.actors[0].resources);
        assert!(replay.guaranteed && replay.success);
        assert!(replay.state.actors[1].broken());
        let mut rng=s.after_escape;
        let counter=ot2_speedrun_assistant::encounter::catalog().maps["Dng_Cty_1_1"].counter.reset(&mut rng,false);
        assert_eq!(rng,s.after_field_return);
        assert!(counter>=p.required_counter);
        assert_eq!(s.inputs.normal_bp,1);
        return;
    }
    panic!("No break fallback exercised");
}
#[test]
fn measured_run_carries_counter_and_checks_exit_instead_of_trigger_reset() {
    let r = route();
    let a = &r.lookout_ambush;
    let p = predict(
        a.after,
        &a.world_after,
        resources(),
        Settings {
            max_boost_repeats: None,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(p.entry_counter, 722);
    assert_eq!(p.first_encounter_update, 61);
    assert_eq!(p.remaining_updates, 100);
    assert_eq!(p.required_counter, 1201);
    assert_eq!(p.before_battle_initialization.draws, 5568);
    let s = p.solution.unwrap();
    assert!(s.attempt.success);
    assert_eq!(s.after_field_return.draws, s.after_escape.draws + 2);
    assert_eq!(s.counter_at_trigger, s.counter - 1200);
    assert!(s.counter_at_trigger > 0);
    assert_eq!(s.attempt.state.actors[1].resources.hp, 217);
    assert_eq!(s.attempt.state.actors[1].shields, 2);
    assert!(!s.world_at_trigger.first_encounter);
    assert!(!s.world_at_trigger.flag(8017)); // next cutscene not executed
    let mut rng = p.before_battle_initialization;
    for t in s.trace {
        assert_eq!(t.before, rng);
        rng.advance(t.after.draws - t.before.draws);
        assert_eq!(t.after, rng);
    }
    assert_eq!(rng, s.after_field_return);
}
#[test]
fn run_budget_counts_exact_trigger_update_and_rejects_impossible_distance() {
    let r = route();
    let mut w = r.lookout_ambush.world_after;
    w.encounter_counter = 720;
    let p = predict(
        r.lookout_ambush.after,
        &w,
        resources(),
        Settings {
            max_boost_repeats: None,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        (
            p.first_encounter_update,
            p.remaining_updates,
            p.required_counter
        ),
        (60, 101, 1213)
    );
    w.encounter_counter = 12; // Early encounter leaves too much of the fixed route.
    let p = predict(
        r.lookout_ambush.after,
        &w,
        resources(),
        Settings {
            max_boost_repeats: None,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(p.status, "Remaining run exceeds maximum counter");
    assert!(p.solution.is_none());
    let optional=predict(r.lookout_ambush.after,&w,resources(),Settings {
        max_boost_repeats:Some(4),run_max_boost_repeats:Some(0),
    }).unwrap();
    let escaped=optional.solution.unwrap();
    assert!(escaped.attempt.success);
    assert!(!escaped.reaches_pirro);
    assert_eq!(escaped.counter_at_trigger,0);
    w.first_encounter = false;
    assert!(predict(r.lookout_ambush.after, &w, resources(), Settings::default()).is_err());
}
#[test]
fn zero_cap_does_not_claim_success_and_unlimited_selects_fewest_solo_bp_inputs() {
    let r = route();
    let w = r.lookout_ambush.world_after;
    let mut exercised = false;
    for seed in 0..100 {
        let p = predict(
            CrtRng::new(seed),
            &w,
            resources(),
            Settings {
                max_boost_repeats: None,
                ..Default::default()
            },
        )
        .unwrap();
        if p.available_bp != 1 || p.solution.as_ref().unwrap().extra_advances == 0 {
            continue;
        }
        let limited = predict(
            CrtRng::new(seed),
            &w,
            resources(),
            Settings {
                max_boost_repeats: Some(0),
                ..Default::default()
            },
        )
        .unwrap();
        let chosen=limited.solution.as_ref().unwrap();
        assert!(chosen.attempt.success);
        assert!(chosen.counter_at_trigger>0);
        if !chosen.break_then_escape {
            assert!(chosen.inputs.extra_repeats<=limited.counter_base_repeats);
        } else {
            assert!(chosen.attempt.guaranteed);
            assert!(chosen.attempt.state.actors[1].broken());
        }
        let s = p.solution.unwrap();
        let situation = if p.first_encounter.situation.as_deref() == Some("Preemptive") {
            Situation::Preemptive
        } else {
            Situation::Normal
        };
        let b = FirstEscapeBattle::mansion(
            p.before_battle_initialization.state,
            situation,
            resources(),
            false,
        )
        .unwrap();
        for fewer in 0..s.extra_advances {
            let inputs: Vec<_> = (0..fewer).flat_map(|_| [1, 0]).collect();
            let a = b.clone().attempt(&inputs, 3).unwrap();
            let mut rng = a.after;
            let counter = ot2_speedrun_assistant::encounter::catalog().maps["Dng_Cty_1_1"]
                .counter
                .reset(&mut rng, false);
            assert!(!a.success || counter < p.required_counter);
        }
        exercised = true;
        break;
    }
    assert!(exercised);
}
