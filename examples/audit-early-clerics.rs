//! Replay every earlier Flats Escape candidate from an exported route, without
//! searching or changing the exported plan. Usage: cargo run --offline --example
//! audit-early-clerics -- <export.json> [maximum boost repeats]
use ot2_speedrun_assistant::{CrtRng, early_cait, encounter, world::{self, Loader, WorldState}};
use ot2_battle_engine::{escape::Situation, field_escape::{FieldEscapeMenu,first_action_break}};
use serde_json::{Value, json};
use std::{collections::BTreeSet, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let input: Value = serde_json::from_str(&std::fs::read_to_string(args.next().ok_or("Expected export path")?)?)?;
    let p = &input["prediction"]["post_battle_field"]["flats_escape"];
    let limit = args.next().map(|s|s.parse::<u32>()).transpose()?.unwrap_or(123);
    let entry: CrtRng = serde_json::from_value(p["before_battle_initialization"].clone())?;
    let state: WorldState = serde_json::from_value(input["prediction"]["post_battle_field"]["harbor_return"]["world_after"].clone())?;
    let level = p["level"].as_u64().ok_or("Expected level")? as u32;
    let situation = match p["encounter"]["situation"].as_str() {
        Some("Normal") => Situation::Normal, Some("Preemptive") => Situation::Preemptive,
        _ => return Err("Incoming encounter is surprised".into()),
    };
    let enemies: Vec<_> = p["encounter"]["enemies"].as_array().ok_or("Expected enemy slots")?.iter()
        .map(|s| s["enemy"]["id"].as_str().unwrap()).collect();
    let volume = &encounter::catalog().volumes[p["encounter"]["volume"].as_str().unwrap()];
    let danger = volume.encounters.iter().rev().find(|r|r.table!="None"&&r.progress<=1).unwrap().danger;
    let menu = FieldEscapeMenu::new(entry, situation, level, &enemies)?;
    let required = p["required_counter"].as_u64().ok_or("Expected counter threshold")? as u32;
    let distance_cost = required - 1;
    let break_audit = if let Some(plan) = first_action_break(entry,situation,level,&enemies)? {
        let mut w = state.clone(); w.first_encounter = false;
        let mut loader = Loader::new(world::catalog(),w,plan.attempt.after)?;
        loader.restore_field_after_battle("Audit Flats break then Escape")?;
        let counter = loader.state.encounter_counter;
        let route = if counter >= required {
            loader.state.encounter_counter -= distance_cost;
            Some(early_cait::project(loader.rng,loader.state,level)?)
        } else {None};
        json!({"weapon":plan.weapon,"boost":plan.boost,"escape_round":plan.escape_round,
            "escape_chance":plan.attempt.chance_percent,"post_escape_counter":counter,
            "required_counter":required,"village_run_safe":counter>=required,
            "clerics":route.as_ref().map(|r|r.clerics_present),
            "full_flats_walk":route.as_ref().map(|r|r.movement.walking.reaches_reset)})
    } else {Value::Null};
    let mut stages = [0u32;5];
    let mut rows = vec![];
    let mut night_states = BTreeSet::new();
    for repeats in 0..=limit {
        let path: Vec<_> = (0..repeats).flat_map(|_| [1,0]).collect();
        let attempt = menu.clone().attempt(&path,danger)?;
        if !attempt.success { stages[0]+=1; continue; }
        let mut w = state.clone(); w.first_encounter = false;
        let mut loader = Loader::new(world::catalog(),w,attempt.after)?;
        loader.restore_field_after_battle("Audit Flats Escape")?;
        let reset_counter = loader.state.encounter_counter;
        if reset_counter < required { stages[1]+=1; continue; }
        loader.state.encounter_counter -= distance_cost;
        let route = early_cait::project(loader.rng,loader.state,level)?;
        if !route.cleric_reach.walking.reaches_reset { stages[2]+=1; }
        else if !route.clerics_present { stages[3]+=1; }
        else if !route.movement.walking.reaches_reset { stages[4]+=1; }
        if let Some(rng) = route.after_night { night_states.insert(rng.state); }
        rows.push(json!({"repeats":repeats,"post_escape_counter":reset_counter,
            "returned_flats_counter":route.world_at_flats.encounter_counter,
            "reach_clerics":route.cleric_reach.walking.reaches_reset,
            "clerics":route.clerics_present,"full_flats_walk":route.movement.walking.reaches_reset,
            "osvald_reached":route.osvald.as_ref().is_some_and(|o|o.after_recruitment.is_some()),
            "night_roll":route.trace.iter().find(|t|t.subject=="GMK_BLACKMARKET_CONTROL").and_then(|t|t.value),
            "night_state":route.after_night.map(|r|format!("0x{:08X}",r.state))}));
    }
    println!("{}",serde_json::to_string_pretty(&json!({"total_candidates":limit+1,
        "escape_failed":stages[0],"village_run_failed":stages[1],"cleric_approach_failed":stages[2],
        "clerics_absent":stages[3],"clerics_but_full_flats_walk_failed":stages[4],
        "distinct_night_states":night_states.len(),"opening_break":break_audit,"village_safe_candidates":rows}))?);
    Ok(())
}
