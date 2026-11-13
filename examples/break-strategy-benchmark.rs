//! Paired A/B benchmark. No changes to the production strategy switch.
use ot2_speedrun_assistant::{CrtRng,Result,SimError,world::{self,Loader,WorldState},snows_escape::Entry,winterbloom,winter_voyage};
use serde_json::{Value,json};
use std::{io::Write,time::Instant};

fn entry(seed:u32)->Result<Entry> {
    let fixture:Value=serde_json::from_str(include_str!("../data/source/live-winterbloom-night-20261009.json"))?;
    let mut w:WorldState=serde_json::from_value(fixture["world_before"].clone())?;
    w.galdera_route=false;
    let start=CrtRng::new(seed);
    let mut l=Loader::new(world::catalog(),w,start)?;
    l.visit_map(winterbloom::MAP)?;
    Ok(Entry{arrival:start,after:l.rng,world_after:l.state,trace:l.trace,decisions:l.decisions})
}
fn n(v:&Value)->u64 {v.as_u64().unwrap_or(0)}
fn downstream_cost(v:&Value)->u64 {
    let o=&v["cropdale_exit"]["oresrush"];let r=&o["ryu"];
    let switches=|k:&str|o[k].as_array().map_or(0,Vec::len) as u64;
    let mut total=(switches("switches_before_partitio").saturating_sub(3)+switches("switches_after_partitio").saturating_sub(2))*12;
    total+=n(&r["hikari_search"]["total_input_cost"]);
    let inventor=&r["canalbrine"]["conning"]["pomegranate"]["inventor"];
    total+=n(&inventor["new_delsta_adjustment"]["input_cost"]);
    let mut leg=&inventor["crackridge_return"];
    while leg.is_object() && !leg["gravell"].is_object() {leg=&leg["southern"];}
    assert_eq!(leg["gravell"]["fight"]["evaluation"]["result"],"accepted","Missing completed EXP100 endpoint");
    if let Some(actions)=leg["gravell"]["fight"]["actions"].as_array() {
        for a in actions {if a["action"]=="thunder_soulstone" {total+=a["inputs"].as_array().map_or(0,Vec::len) as u64;}}
    }
    total
}
fn run(seed:u32,old:bool,full:bool)->Result<Value> {
    winter_voyage::benchmark_require_harbor(old);
    let e=entry(seed)?;
    let start=Instant::now();
    let p=winterbloom::with_evasive(&e)?.ok_or_else(||SimError("Ambushed at shared entry".into()))?;
    let winter_cost=p.input_cost();
    let second=p.predicted_second_encounter;
    let voyage=if second {p.voyage_with_second.unwrap()}else{p.voyage_without_second.unwrap()};
    let harbor_cost=voyage.escape.weighted_input_cost;
    let cropdale_cost=voyage.cropdale_exit.escape.weighted_input_cost;
    let location=if voyage.escape.break_action.is_some(){"harbor"}else{"cropdale"};
    assert!(voyage.escape.break_action.as_ref().is_some_and(|b|b.attack_actor==0)||voyage.cropdale_exit.escape.break_action.as_ref().is_some_and(|b|b.attack_actor==0));
    let local_seconds=start.elapsed().as_secs_f64();
    let mut v=serde_json::to_value(voyage)?;
    if full {ot2_speedrun_assistant::conning::extend_selected_routes(&mut v)?;}
    let full_seconds=start.elapsed().as_secs_f64();
    let local_cost=u64::from(winter_cost+harbor_cost+cropdale_cost);
    Ok(json!({"seed":seed,"hex":format!("{seed:08X}"),"strategy":if old{"harbor_required"}else{"cropdale_fallback"},"second":second,"break_location":location,
        "winter_cost":winter_cost,"harbor_cost":harbor_cost,"cropdale_cost":cropdale_cost,"local_cost":local_cost,"local_seconds":local_seconds,
        "through_exp100_cost":if full{Some(local_cost+downstream_cost(&v))}else{None},"full_seconds":if full{Some(full_seconds)}else{None}}))
}
fn main() {
    std::thread::Builder::new().stack_size(64*1024*1024).spawn(|| {
        let args:Vec<_>=std::env::args().collect();
        let count:usize=args.get(1).map_or(32,|x|x.parse().unwrap());
        let full=args.get(2).is_some_and(|x|x=="full");
        let path=args.get(3).map(String::as_str).unwrap_or("outputs/break-strategy-benchmark.jsonl");
        std::fs::create_dir_all("outputs").unwrap();
        let mut output=std::fs::File::create(path).unwrap();
        // Deterministic SplitMix64 samples; reject only the common ambushed opening.
        let mut sampler=20261112u64;let mut seeds=Vec::new();let mut rejected=0;
        while seeds.len()<count {
            sampler=sampler.wrapping_add(0x9e3779b97f4a7c15);let mut z=sampler;
            z=(z^(z>>30)).wrapping_mul(0xbf58476d1ce4e5b9);z=(z^(z>>27)).wrapping_mul(0x94d049bb133111eb);z^=z>>31;
            let seed=(z>>32) as u32;let e=entry(seed).unwrap();
            if winterbloom::encounter_at(e.after,&e.world_after).unwrap().situation.as_deref()==Some("Surprise") {rejected+=1;continue;}
            seeds.push(seed);
        }
        eprintln!("{count} paired seeds; {rejected} common ambush entries excluded; full={full}");
        // Warm both strategies' lazy tables before recording search time.
        for old in [true,false] {run(seeds[0],old,false).unwrap();}
        for (i,&seed) in seeds.iter().enumerate() {
            for old in if i%2==0 {[true,false]}else{[false,true]} {
                let row=match run(seed,old,full) {Ok(v)=>v,Err(e)=>json!({"seed":seed,"strategy":if old{"harbor_required"}else{"cropdale_fallback"},"error":e.to_string()})};
                writeln!(output,"{row}").unwrap();output.flush().unwrap();
                eprintln!("{}/{} {} {} cost={} time={}",i+1,count,row["hex"],row["strategy"],row["local_cost"],row["full_seconds"].as_f64().unwrap_or_else(||row["local_seconds"].as_f64().unwrap_or(0.0)));
            }
        }
    }).unwrap().join().unwrap();
}
