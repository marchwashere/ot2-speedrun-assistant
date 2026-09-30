//! Estimate the user's three order conditions over uniform battle-entry states.
//! Run: cargo run --release --offline --example second-fight-odds -- 1000000 0
use ot2_battle_engine::{Battle, BattleConfig, ItemCatalog, Outcome, routine::ThroneSecondRoutine};
use ot2_speedrun_assistant::second::Assessment;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let samples: u64 = args.next().as_deref().unwrap_or("1000000").parse()?;
    let sample_seed: u64 = args.next().as_deref().unwrap_or("0").parse()?;
    if samples == 0 || args.next().is_some() {
        return Err("usage: second-fight-odds [positive sample count] [sampling seed]".into());
    }
    let base = Battle::new_throne_second(
        BattleConfig::throne_second(0)?,
        ItemCatalog::builtin()?,
        false,
    )?;
    let mut sampler = sample_seed;
    let mut conditions = [0_u64; 3];
    let mut all_conditions = 0_u64;
    let mut only_one_enemy_action = 0_u64;
    let mut mismatches = 0_u64;
    let mut wins = 0_u64;
    let mut failures = 0_u64;
    let mut first_failure = None;
    let mut examples = [None, None];
    let start = std::time::Instant::now();
    for _ in 0..samples {
        // SplitMix64 with a recorded seed: sample the whole 32-bit CRT space,
        // rather than a small consecutive range of initial states.
        sampler = sampler.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = sampler;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        let seed = ((z ^ (z >> 31)) >> 32) as u32;
        let mut battle = base.restarted(seed);
        let mut assessment = Assessment::new(&battle)?;
        let c = assessment.order_conditions;
        for (count, passed) in conditions.iter_mut().zip([
            c.throne_before_right_round_one,
            c.scaracci_before_left_round_one,
            c.throne_before_leader_round_two,
        ]) {
            *count += u64::from(passed);
        }
        all_conditions += u64::from(c.all_met());
        let mut routine = ThroneSecondRoutine;
        for _ in 0..18 {
            if battle.state().outcome != Outcome::Ongoing {
                break;
            }
            match routine.step(&mut battle) {
                Ok(report) => assessment.record(&battle, &report),
                Err(error) => {
                    failures += 1;
                    first_failure.get_or_insert((format!("0x{seed:08X}"), error.to_string()));
                    break;
                }
            }
        }
        assessment.finish(battle.state().outcome);
        wins += u64::from(battle.state().outcome == Outcome::Victory);
        only_one_enemy_action += u64::from(assessment.only_one_enemy_action);
        mismatches += u64::from(assessment.only_one_enemy_action != c.all_met());
        examples[usize::from(c.all_met())].get_or_insert(format!("0x{seed:08X}"));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "population": "Uniform 32-bit CRT states immediately before second-battle initialization; Lv2, fixed routine. Not conditioned on the first battle or boost search.",
            "sampler": "SplitMix64 upper 32 bits, with replacement", "sample_seed": sample_seed,
            "samples": samples, "conditions": conditions, "all_conditions": all_conditions,
            "all_conditions_percent": 100.0 * all_conditions as f64 / samples as f64,
            "only_one_enemy_action": only_one_enemy_action, "order_outcome_mismatches": mismatches,
            "wins": wins, "routine_errors": failures, "first_failure": first_failure,
            "example_fail_state": examples[0], "example_pass_state": examples[1],
            "seconds": start.elapsed().as_secs_f64(),
            "verification": "Simulation estimate using the corrected core; one seed-zero live replay matches all CRT calls. This does not validate the entire sampled population. One enemy action is not necessarily one damage hit."
        }))?
    );
    Ok(())
}
