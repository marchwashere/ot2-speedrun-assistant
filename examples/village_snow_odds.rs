//! Exact baseline odds for the fresh first counter after returning from village.
//! Uniform CRT states before the counter roll; not conditioned on the route/search.
use ot2_speedrun_assistant::{CrtRng, encounter};
// Count exact successful B/A counter pairs over uniform 32-bit CRT states.
// Bit 31 has no effect on either rand() result, so count 2^31 distinct pairs.
// For each upper-15-bit B result, count intervals of the 65,536 low-bit states
// whose next result exceeds the A threshold. This preserves LCG correlation.
fn counter_odds(required: u32, first_counter: bool) -> (u64, u64) {
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
        let needed = (i64::from(required) * if first_counter { 2 } else { 1 }
            - i64::from(counter.min)
            - i64::from(b))
        .max(0);
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
    for (label, required) in [
        ("Measured 70-update walk", 631),
        ("Safe 72-update walk", 649),
        ("Padded 68-update run", 817),
    ] {
        let (good, total) = counter_odds(required, true);
        println!(
            "{label}: counter >= {required}; {good}/{total} = {:.10}% no encounter",
            100.0 * good as f64 / total as f64
        );
    }
}
