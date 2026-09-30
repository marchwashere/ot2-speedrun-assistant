use ot2_battle_engine::{
    Battle, BattleConfig, Event, ItemCatalog, Outcome, Side, routine::ThroneSecondRoutine,
};
use ot2_speedrun_assistant::second::Assessment;

fn make(seed: u32, trace: bool) -> Battle {
    Battle::new_throne_second(
        BattleConfig::throne_second(seed).unwrap(),
        ItemCatalog::builtin().unwrap(),
        trace,
    )
    .unwrap()
}

#[test]
fn order_checks_agree_with_execution_including_breaks_and_double_thrust() {
    let base = make(0, false);
    let mut seen = [false; 8];
    let mut multi_hit_action = false;
    let mut enemy_miss = false;
    for seed in 0..4096_u32 {
        let mut battle = base.restarted(seed.wrapping_mul(0x9E3779B9));
        let before = battle.state().clone();
        let mut assessment = Assessment::new(&battle).unwrap();
        assert_eq!(battle.state(), &before, "assessment must not advance CRT");
        let c = assessment.order_conditions;
        seen[usize::from(c.throne_before_right_round_one)
            | (usize::from(c.scaracci_before_left_round_one) << 1)
            | (usize::from(c.throne_before_leader_round_two) << 2)] = true;
        let mut actual = Vec::new();
        let mut routine = ThroneSecondRoutine;
        let mut round_two_checked = false;
        while battle.state().outcome == Outcome::Ongoing && battle.state().round <= 2 {
            let round = battle.state().round;
            if round == 2 && !round_two_checked {
                round_two_checked = true;
                let order = battle.state().turn_order();
                let position = |name| {
                    order
                        .iter()
                        .position(|id| Some(*id) == battle.actor_id(name))
                        .unwrap()
                };
                assert_eq!(
                    position("throne") < position("leader"),
                    c.throne_before_leader_round_two
                );
            }
            let report = routine.step(&mut battle).unwrap();
            if battle.side(report.actor) == Some(Side::Enemy) {
                actual.push((round, report.actor));
                enemy_miss |= report
                    .events
                    .iter()
                    .any(|e| matches!(e, Event::Miss { .. }));
            }
            assessment.record(&battle, &report);
        }
        assessment.finish(battle.state().outcome);
        assert_eq!(assessment.enemy_actions as usize, actual.len());
        assert_eq!(assessment.only_one_enemy_action, c.all_met(), "seed {seed}");
        if c.all_met() {
            assert_eq!(actual, [(1, battle.actor_id("leader").unwrap())]);
            multi_hit_action |= assessment.damage_hits_received == 2;
        }
    }
    assert!(
        seen.into_iter().all(|seen| seen),
        "exercise all condition combinations"
    );
    assert!(multi_hit_action, "one action must not imply one damage hit");
    assert!(enemy_miss, "missed enemy turns must still count");
}

#[test]
fn assessment_is_trace_independent_and_requires_completed_victory() {
    let seed = 0x1B39896A;
    let mut traced = make(seed, true);
    let mut plain = make(seed, false);
    let mut a = Assessment::new(&traced).unwrap();
    let mut b = Assessment::new(&plain).unwrap();
    assert!(a.order_conditions_met);
    assert!(!a.only_one_enemy_action);
    let mut routine = ThroneSecondRoutine;
    while traced.state().outcome == Outcome::Ongoing {
        let report = routine.step(&mut traced).unwrap();
        assert_eq!(report, routine.step(&mut plain).unwrap());
        a.record(&traced, &report);
        b.record(&plain, &report);
    }
    a.finish(traced.state().outcome);
    b.finish(plain.state().outcome);
    assert_eq!(a, b);
    assert!(a.only_one_enemy_action);
    a.finish(Outcome::Ongoing);
    assert!(!a.only_one_enemy_action);
    a.finish(Outcome::Defeat);
    assert!(!a.only_one_enemy_action);
}
