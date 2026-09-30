//! Continue a GUI route export through successful Highroad Escape to the Flats.
//! Assumes daytime, no intervening action, and no second Highroad encounter.
use ot2_speedrun_assistant::{CrtRng, encounter, flats, world};

// Count exact successful B/A counter pairs over uniform 32-bit CRT states.
// Bit 31 has no effect on either rand() result, so count 2^31 distinct pairs.
// For each upper-15-bit B result, count intervals of the 65,536 low-bit states
// whose next result exceeds the A threshold. This preserves LCG correlation.
fn counter_odds(required: u32) -> (u64, u64) {
    let counter = &encounter::catalog().maps["Fld_Cty_1_2"].counter;
    assert_eq!(
        (
            counter.min,
            counter.range_a,
            counter.range_b,
            counter.first_ratio
        ),
        (1000, 400, 400, 0.5)
    );
    let mut inverse = 1u32;
    for _ in 0..5 {
        inverse = inverse.wrapping_mul(2u32.wrapping_sub(214_013u32.wrapping_mul(inverse)));
    }
    let buckets: Vec<u32> = (0u32..32768)
        .map(|h| {
            // Ask the core's actual float-based index helper for each rand result.
            let before = (h << 16).wrapping_sub(2_531_011).wrapping_mul(inverse);
            CrtRng::new(before).index(counter.range_a)
        })
        .collect();
    let modulus = 1i64 << 31;
    let mut good = 0u64;
    for (h, &b) in buckets.iter().enumerate() {
        let needed = (2 * i64::from(required) - i64::from(counter.min) - i64::from(b)).max(0);
        let first = buckets.partition_point(|&a| i64::from(a) < needed);
        if first == 0 {
            good += 65536;
            continue;
        }
        if first == buckets.len() {
            continue;
        }
        let lower = (first as i64) << 16;
        let base = (214_013 * ((h as i64) << 16) + 2_531_011) % modulus;
        let max_wrap = (base + 214_013 * 65535) / modulus;
        for wrap in 0..=max_wrap {
            let start = (wrap * modulus + lower - base + 214_012)
                .div_euclid(214_013)
                .max(0);
            let end = ((wrap + 1) * modulus - 1 - base)
                .div_euclid(214_013)
                .min(65535);
            if start <= end {
                good += (end - start + 1) as u64;
            }
        }
    }
    (good, 1u64 << 31)
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("path to GUI route export JSON");
    let input: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let highroad = &input["post_battle_field"]["highroad"];
    assert_eq!(highroad["escape"]["attempt"]["success"], true);
    let state: world::WorldState =
        serde_json::from_value(highroad["field_return"]["world_after"].clone()).unwrap();
    let mut after_escape: CrtRng =
        serde_json::from_value(highroad["escape"]["attempt"]["after"].clone()).unwrap();
    let origin: u64 = input["crt_setup"]["battle_entry_advances"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    after_escape.draws += origin;
    let mut after_field_return: CrtRng =
        serde_json::from_value(highroad["field_return"]["after"].clone()).unwrap();
    after_field_return.draws += origin;
    let prediction = flats::with_world_state(after_field_return, state).unwrap();
    let rows: Vec<_> = [("walking", &prediction.harbor_reach.walking), ("running", &prediction.harbor_reach.running)]
        .into_iter()
        .map(|(movement, budget)| {
            let (good, total) = counter_odds(budget.minimum_start_counter);
            serde_json::json!({"movement":movement,
            "updates":budget.updates,"decrement":budget.decrement,"cost":budget.cost,"required_counter":budget.minimum_start_counter,
            "safe":budget.reaches_reset,
            "remaining_counter":i64::from(prediction.world_after.encounter_counter)-i64::from(budget.cost),
            "safe_states_mod_2_31":good,"states_mod_2_31":total,
            "chance_percent":100.0*good as f64/total as f64})
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "verification":prediction.verification,
        "movement_verification":prediction.harbor_reach.verification,
        "after_escape":after_escape,"after_field_return":after_field_return,"after_load":prediction.after,
        "load_advances":prediction.after.draws-after_field_return.draws,
        "counter":prediction.world_after.encounter_counter,"movement":rows,
        "trace":prediction.trace,"decisions":prediction.decisions,"world_after":prediction.world_after
    })).unwrap());
}
