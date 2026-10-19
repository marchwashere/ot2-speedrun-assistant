//! Reproduce an identified Galdera route and print its carried JP ledger.
use serde_json::Value;
fn latest(v: &Value, best: &mut Option<(u64, Value)>) {
    if let Some(n) = v["after"]["draws"].as_u64() {
        if v["world_after"]["character_progression"].is_object()
            && best.as_ref().is_none_or(|(old, _)| n > *old)
        { *best = Some((n, v.clone())); }
    }
    match v {
        Value::Object(o) => for x in o.values() { latest(x, best); },
        Value::Array(a) => for x in a { latest(x, best); },
        _ => {}
    }
}
fn ledger(v: &Value) {
    let mut point = None; latest(v, &mut point);
    if let Some((draws, p)) = point {
        eprintln!("checkpoint {draws}, map {}", p["world_after"]["map"]);
        for id in [7, 6, 4, 5, 8] {
            let c = &p["world_after"]["character_progression"][id.to_string()];
            if !c.is_null() {
                eprintln!("{} Lv{}: earned {}, spent {}, jobs {}", c["character"]["name"], c["level"], c["jp_earned"], c["jp_spent"], c["jobs"]["learned_abilities"]);
            }
        }
    }
}
fn verify_jp_endpoint(v:&Value)->bool {
    if v["location"].is_string()&&v["fight"]["state"]["grace"].is_object() {
        let world=serde_json::from_value(v["field_return"]["world_after"].clone()).unwrap();
        eprintln!("Selected {}: {} start, {} JP, {} extra inputs, {} Tropu switches",
            v["location"],v["encounter"]["situation"],v["rewards"]["awarded"]["jp"],v["extra_input_cost"],v["boat"]["extra_switches"]);
        let budgets=ot2_speedrun_assistant::galdera_jp::audit(&world).unwrap();
        eprintln!("Budgets: {}",serde_json::to_string(&budgets).unwrap());
        assert!(budgets.iter().all(|b|b.shortfall==0));
        return true;
    }
    match v {
        Value::Object(o)=>o.values().any(verify_jp_endpoint),
        Value::Array(a)=>a.iter().any(verify_jp_endpoint),
        _=>false,
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let label = args.get(1).map(String::as_str).unwrap_or("baseline");
    let voices = "NNNSNSNNNNSSSSSSSSNNSSSSSNSNSN";
    let id = ot2_speedrun_assistant::voice_identification::identify_observed(voices, Some((true,154)))?;
    eprintln!("Identification: {id}");
    assert_eq!(id["candidate"]["current_state"], 0x3054ba63u32);
    let started = std::time::Instant::now();
    let v = ot2_speedrun_assistant::identified_route::simulate_route_progress(voices, true, 154, true, &mut |message,v| {
        eprintln!("{:.1}s: {message}", started.elapsed().as_secs_f64()); ledger(v);
        std::fs::write(format!("target/jp-audit-3054ba63-{label}-progress.json"), serde_json::to_vec(v).unwrap()).unwrap();
        Ok(())
    })?;
    ledger(&v);
    assert!(verify_jp_endpoint(&v),"Missing accepted JP100 endpoint");
    std::fs::write(format!("target/jp-audit-3054ba63-{label}.json"), serde_json::to_vec(&v)?)?;
    eprintln!("Finished in {:.1}s",started.elapsed().as_secs_f64());
    Ok(())
}
fn main() {
    std::thread::Builder::new().stack_size(32*1024*1024).spawn(||run().unwrap()).unwrap().join().unwrap();
}
