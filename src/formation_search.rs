//! Formation is an outcome of real RNG replay, not a whitelist. Compare the
//! preceding controllable inputs using the cost of their downstream continuation.
use crate::{Result,ensure,highroad,flats,harbor,flats_escape,early_cait,pirro,pirro_aftermath,second,
    escape_search::FieldSearch};
use ot2_battle_engine::{Outcome,escape::Situation};
use std::cell::Cell;
use serde::Serialize;

#[derive(Debug,Serialize)]
pub struct Report {
    pub baseline_cost:u32,
    pub selected_cost:u32,
    pub tested_continuations:u64,
    pub reused_continuations:u64,
    pub verification:&'static str,
}

pub struct Continuation {
    pub flats:flats::Prediction,
    pub harbor:harbor::Prediction,
    pub escape:flats_escape::Prediction,
}
fn downstream_cost(p:&flats_escape::Prediction)->Option<u32> {
    let mut cost=p.early_weighted_input_cost?;
    let early=p.early_cait.as_ref()?;
    if let Some(after)=&early.after_rare {
        cost+=after.passage.escape.weighted_input_cost;
        cost+=after.passage.winterbloom.as_ref()?.input_cost();
    } else {
        let before=early.snows_approach.as_ref()?.predicted_before_osvald;
        let a=p.snows_escape.as_ref()?;
        let selected=if before {a.before_osvald.as_ref()?} else {a.after_osvald.as_ref()?};
        cost+=selected.escape.weighted_input_cost;
        if let Some(w)=&selected.western {cost+=w.escape.weighted_input_cost+w.winterbloom.input_cost();}
        if let Some(w)=&selected.winterbloom_after_eastern {cost+=w.input_cost();}
    }
    Some(cost)
}
fn continuation(h:&highroad::Prediction,fs:flats_escape::Settings,es:early_cait::Settings)->Result<Option<Continuation>> {
    let Some(field)=&h.field_return else {return Ok(None)};
    let flats=flats::after_highroad(field)?;
    let Some(harbor)=harbor::after_flats(&flats)? else {return Ok(None)};
    let escape=flats_escape::until_osvald_cait(harbor.after,&harbor.world_after,h.level,fs,es)?;
    Ok(downstream_cost(&escape).map(|_|Continuation {flats,harbor,escape}))
}

/// Freeze R3 and all earlier decisions. Compare R8 and Highroad Escape futures,
/// including the actual later Flats/clerics/Snows strategy. The initial valid
/// route supplies the search bound, so Required outcomes are not capped.
pub fn optimize(
    boss:&mut pirro::Prediction, second:&second::Prediction,
    ending:&mut pirro_aftermath::Prediction, highroad:&mut highroad::Prediction,
    fs:flats_escape::Settings, es:early_cait::Settings, replay:bool,
) -> Result<Option<Continuation>> {
    let _profile=crate::profile::span("formation optimize");
    let Some(mut best)=continuation(highroad,fs,es)? else {return Ok(None)};
    let baseline_path=highroad.approach_search.as_ref().map(|s|s.plan.clone()).unwrap_or(vec![1,2,3]);
    let cost=Cell::new((baseline_path.len()-3) as u32
        +highroad.escape.as_ref().unwrap().weighted_input_cost+downstream_cost(&best.escape).unwrap());
    let baseline_cost=cost.get();
    let mut selected=None;
    let r3=boss.boost_search.as_ref().map(|s|s.plan.clone())
        .or_else(||boss.manipulation.as_ref().map(|m|m.inputs.levels.clone())).unwrap_or(vec![1,2,3]);
    let mut world=boss.world_at_entry.clone();world.set_flag(8017,false);
    let inventory=pirro::replay_inventory(boss)?;
    // Keep optional successes already secured by the initial route.
    let keep=early_cait::outcomes_for(best.escape.early_cait.as_ref().unwrap(),es.require_cait);
    let mut constrained_es=es;let mut constrained_fs=fs;
    if keep[0] {constrained_es.clerics_max_boost_repeats=None;}
    if keep[1] {constrained_fs.snow_max_boost_repeats=None;}
    if keep[2] && es.require_cait {constrained_fs.osvald_max_boost_repeats=None;}
    constrained_es.cait_max_boost_repeats=None;
    let settings=highroad.settings;
    let mut tested=0u64;
    let mut reused=0u64;
    let mut frontier=32u32.min(baseline_cost);
    cost.set(frontier);
    loop {
    let mut seen=std::collections::BTreeMap::new();
    let mut visit=|path:&[u8]|->Result<()> {crate::world::without_diagnostics(|| {
        let extra=path.len().saturating_sub(3) as u32;
        if extra+14>=cost.get() {return Ok(())}
        let candidate_boss=pirro::predict_with_plans(boss.arrival,&world,
            boss.initial_state.actors[0].resources.clone(),inventory.clone(),&r3,path)?;
        if candidate_boss.state.outcome!=Outcome::Victory || candidate_boss.routine_error.is_some()
            || candidate_boss.routine.heal_before_round_eight!=Some(false) {return Ok(())}
        let mut candidate_ending=pirro_aftermath::after_battle(&candidate_boss)?.unwrap();
        highroad::carry_progression(&mut candidate_ending.world_after,second,&candidate_boss)?;
        let level=highroad::route_level(second,&candidate_boss)?;
        let h=highroad::predict_internal(&candidate_ending,level,settings,false)?;
        let situation=match h.encounter.situation.as_deref() {
            Some("Normal")=>Situation::Normal,Some("Preemptive")=>Situation::Preemptive,_=>return Ok(())
        };
        let ids:Vec<_>=h.encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
        let volume=&crate::encounter::catalog().volumes[h.encounter.volume.as_ref().unwrap()];
        let danger=volume.encounters.iter().rev().find(|r|r.table!="None" && r.progress<=1).unwrap().danger;
        let mut search=FieldSearch::new(h.before_battle_initialization,situation,level,&ids,danger)?;
        let mut escape_cost=0;
        while extra+escape_cost+14<cost.get() {
            for mut escape in search.candidates(escape_cost,settings.escape_max_boost_repeats.is_some(),None)? {
                tested+=1;
                let field=highroad::return_to_field(&h.world_after_al,escape.attempt.as_ref().unwrap())?;
                // Different upstream paths frequently converge on the same real
                // field state. A more expensive prefix cannot improve its future.
                let key=(field.after.state,field.after.draws,crate::state_key::WorldKey::new(&field.world_after));
                let prefix=extra+escape_cost;
                if seen.get(&key).is_some_and(|&prior|prior<=prefix) {reused+=1;continue;}
                seen.insert(key,prefix);
                let f=flats::after_highroad(&field)?;
                let Some(harbor)=harbor::after_flats(&f)? else {continue};
                let (_,encounter)=flats_escape::first_encounter(harbor.after,&harbor.world_after,level)?;
                if !matches!(encounter.situation.as_deref(),Some("Normal"|"Preemptive")) {continue;}
                let bound=cost.get()-extra-escape_cost-14-1;
                let Some(e)=flats_escape::within_route_cost(harbor.after,&harbor.world_after,level,
                    constrained_fs,constrained_es,bound)? else {continue};
                let Some(downstream)=downstream_cost(&e) else {continue};
                let total=extra+escape_cost+downstream;
                if total>=cost.get() {continue;}
                let mut picked=highroad::predict_internal(&candidate_ending,level,settings,false)?;
                escape.flats_search=Some(highroad::FlatsSearch {max_extra_boost_repeats:settings.flats_max_boost_repeats,
                    escape_repeats:escape.boost_repeat_count,added_repeats:0,tested_candidates:tested,target_met:true,
                    baseline_counter:f.world_after.encounter_counter,selected_counter:f.world_after.encounter_counter,
                    required_counter:f.harbor_reach.walking.minimum_start_counter,status:"Harbor walking preserved by formation lookahead"});
                escape.flats_approach_search=Some(highroad::FlatsApproachSearch {
                    max_extra_boost_repeats:settings.flats_approach_max_boost_repeats,prior_repeats:escape.boost_repeat_count,
                    added_repeats:0,tested_candidates:tested,target_met:true,status:"Non-surprise Flats opening preserved"});
                picked.escape=Some(escape);picked.field_return=Some(field);
                picked.approach_search=Some(highroad::ApproachSearch {max_boost_repeats:settings.approach_max_boost_repeats,
                    break_max_boost_repeats:Some(0),unboosted_break_met:flats_escape::unboosted_opening_break(
                        picked.before_battle_initialization,&picked.encounter,level)?,tested_candidates:tested,target_met:true,
                    plan:path.to_vec(),status:"Formation selected by downstream manipulation cost; no break requirement"});
                cost.set(total);best=Continuation {flats:f,harbor,escape:e};
                selected=Some((path.to_vec(),picked,bound));
                break;
            }
            escape_cost+=1;
        }
        Ok(())
    })};
    if replay || boss.routine.heal_before_round_eight!=Some(false) {
        visit(&baseline_path)?;
    } else {
        let menu=pirro::r8_menu(boss,&r3)?;
        pirro::visit_route_paths(&menu,menu.can_use_latent(),&cost,&mut visit)?;
    }
    drop(visit);
    if selected.is_some() || frontier>=baseline_cost {break;}
    frontier=frontier.saturating_mul(2).min(baseline_cost);
    cost.set(frontier);
    }
    if selected.is_none() {cost.set(baseline_cost);}
    if let Some((path,mut h,bound))=selected {
        let mut p=pirro::predict_with_plans(boss.arrival,&world,
            boss.initial_state.actors[0].resources.clone(),inventory,&r3,&path)?;
        let mut after=pirro_aftermath::after_battle(&p)?.unwrap();
        highroad::carry_progression(&mut after.world_after,second,&p)?;
        // Trials omit only loader diagnostics. Rebuild the selected boundary and
        // its exact bounded continuation so exported traces and search metadata
        // match a normal detailed run, including both player timing overrides.
        let mut picked=highroad::predict_internal(&after,h.level,settings,false)?;
        picked.escape=h.escape.take();picked.approach_search=h.approach_search.take();
        let field=highroad::return_to_field(&picked.world_after_al,
            picked.escape.as_ref().unwrap().attempt.as_ref().unwrap())?;
        let f=flats::after_highroad(&field)?;
        let harbor=harbor::after_flats(&f)?.ok_or_else(||crate::SimError("Selected formation lost harbor access during detailed replay".into()))?;
        let escape=flats_escape::within_route_cost(harbor.after,&harbor.world_after,h.level,
            constrained_fs,constrained_es,bound)?.ok_or_else(||crate::SimError("Selected formation lost its bounded continuation during detailed replay".into()))?;
        ensure(downstream_cost(&escape)==downstream_cost(&best.escape),"Detailed formation replay changed the selected cost")?;
        picked.field_return=Some(field);best=Continuation {flats:f,harbor,escape};
        p.boost_search=boss.boost_search.take();*boss=p;*ending=after;*highroad=picked;
    }
    highroad.formation_search=Some(Report {baseline_cost,selected_cost:cost.get(),tested_continuations:tested,
        reused_continuations:reused,verification:"Compare supported R8 boost paths and Highroad Escape strategies using real downstream Flats/clerics/Snows continuations. Cost: extra boost inputs +10 per break +7 per time switch. Earlier decisions stay frozen; this is not an exhaustive optimizer over every possible battle action."});
    ensure(downstream_cost(&best.escape).is_some(),"Formation search lost its complete continuation")?;
    Ok(Some(best))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multiple_enemies_are_valid_escape_candidates_without_any_break() {
        let ids=["ENE_NML_FLD_CTY_010","ENE_NML_FLD_CTY_120"];
        let mut search=FieldSearch::new(crate::CrtRng::new(0),Situation::Normal,5,&ids,5).unwrap();
        for cost in 0..=40 {
            let candidates=search.candidates(cost,true,None).unwrap();
            if let Some(p)=candidates.first() {
                assert!(p.attempt.as_ref().unwrap().success);
                assert!(p.break_action.is_none());
                assert_eq!(p.menu.current_order.len(),3);
                return;
            }
        }
        panic!("Two-enemy first Escape must remain searchable");
    }
}
