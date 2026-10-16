//! First Southern Cape Cold Snows encounter -> actual Escape/recruitment ->
//! Eastern Cape Cold Snows. Every battle draw comes from the battle core.
use crate::{CrtRng,Result,SimError,ensure,early_cait::{Prediction as Early,EncounterBranch},
    encounter::{self,Boundary,PredictionOptions},escape_search::FieldSearch,highroad::{Escape,FieldReturn},
    osvald,route::MovementBudget,world::{self,Loader,Trace,Decision,WorldState}};
use ot2_battle_engine::escape::Situation;
use serde::Serialize;

pub const DESTINATION:&str="Fld_Snw_1_3";
pub const ONWARD_RUN_UPDATES:u32=85;
pub const ONWARD_WALK_UPDATES:u32=90;
pub const ONWARD_LEEWAY_UPDATES:u32=2;

#[derive(Debug,Serialize)]
pub struct Alternatives {
    pub before_osvald:Option<Prediction>,
    pub after_osvald:Option<Prediction>,
    pub before_error:Option<String>,
    pub after_error:Option<String>,
}
#[derive(Debug,Serialize)]
pub struct Prediction {
    pub before_osvald:bool,
    pub escape:Escape,
    pub field_return:FieldReturn,
    pub recruitment:Option<osvald::Recruitment>,
    pub remaining_to_osvald:Option<MovementBudget>,
    pub onward:Onward,
    pub eastern_entry:Entry,
    pub eastern_encounter:encounter::Prediction,
    pub rare_victory:Option<crate::snows_rare::Victory>,
    pub western:Option<crate::eastern_escape::Prediction>,
    pub winterbloom_after_eastern:Option<crate::winterbloom::Prediction>,
    pub western_entry_after_eastern:Option<Entry>,
    pub verification:&'static str,
}
#[derive(Debug,Serialize)]
pub struct Onward {
    pub start:String,
    pub destination:&'static str,
    pub initial_counter:u32,
    pub walking:MovementBudget,
    pub running:MovementBudget,
    pub running_distance_percent:f64,
    pub walking_distance_percent:f64,
    pub estimated_cost:u32,
    pub estimated_counter_at_exit:u32,
}
#[derive(Debug,Clone,Serialize)]
pub struct Entry {
    pub arrival:CrtRng,
    pub after:CrtRng,
    pub world_after:WorldState,
    pub trace:Vec<Trace>,
    pub decisions:Vec<Decision>,
}

pub fn alternatives(early:&Early,level:u32)->Result<Alternatives> {
    let a=early.snows_approach.as_ref().ok_or_else(||SimError("Missing Snows timing branches".into()))?;
    let before=predict_route(&a.before_osvald,level,true);
    let after=if a.predicted_before_osvald {None} else {Some(predict_route(&a.after_osvald,level,false))};
    let (before_osvald,before_error)=match before {Ok(p)=>(Some(p),None),Err(e)=>(None,Some(e.to_string()))};
    let (after_osvald,after_error)=match after {Some(Ok(p))=>(Some(p),None),Some(Err(e))=>(None,Some(e.to_string())),None=>(None,None)};
    Ok(Alternatives {before_osvald,after_osvald,before_error,after_error})
}

/// Trials need only the selected timing; overrides are reconstructed for display.
pub fn trial_alternatives(early:&Early,level:u32,limit:Option<u32>)->Result<Alternatives> {
    let a=early.snows_approach.as_ref().ok_or_else(||SimError("Missing Snows timing branches".into()))?;
    let before=a.predicted_before_osvald;
    let p=predict_route_bounded(if before {&a.before_osvald}else{&a.after_osvald},level,before,limit)?;
    let mut out=Alternatives {before_osvald:None,after_osvald:None,before_error:None,after_error:None};
    if before {out.before_osvald=p;}else{out.after_osvald=p;}
    Ok(out)
}

fn decrement(time:u8,dash:bool)->Result<u32> {
    decrement_with_support(time,dash,false)
}
fn decrement_with_support(time:u8,dash:bool,evasive:bool)->Result<u32> {
    encounter::catalog().counter_decrement(&PredictionOptions {map:osvald::MAP.into(),
        zone:"ENT_FLD_SNW_1_2_LND".into(),period:if time==2 {"DayTime"} else {"Night"}.into(),dash,ability_percent:if evasive {50}else{100},..Default::default()})
}

fn onward_updates(before:bool,trigger:u32)->(u32,u32) {
    let run_updates=if before {ONWARD_RUN_UPDATES} else {ONWARD_RUN_UPDATES.saturating_sub(trigger)};
    // After recruitment, run until the encounter. Convert the untraveled
    // distance using the two measured whole-route densities, rounding up.
    let walk_updates=if before {ONWARD_WALK_UPDATES} else {(ONWARD_WALK_UPDATES*run_updates).div_ceil(ONWARD_RUN_UPDATES)};
    // Add the full allowance once, after converting the remaining distance.
    (walk_updates+ONWARD_LEEWAY_UPDATES,run_updates+ONWARD_LEEWAY_UPDATES)
}

pub fn onward(counter:u32,time:u8,before:bool,trigger:u32)->Result<Onward> {
    onward_with_support(counter,time,before,trigger,false)
}
fn onward_with_support(counter:u32,time:u8,before:bool,trigger:u32,evasive:bool)->Result<Onward> {
    let (walk_updates,run_updates)=onward_updates(before,trigger);
    let walking=MovementBudget::new(counter,walk_updates,decrement_with_support(time,false,evasive)?);
    let running=MovementBudget::new(counter,run_updates,decrement_with_support(time,true,evasive)?);
    ensure(walking.reaches_reset,"Cannot reach Eastern Cape Cold Snows even walking after this reset")?;
    let tenths=if running.reaches_reset {1000} else {
        ensure(running.cost>walking.cost,"Invalid measured movement costs")?;
        ((u64::from(counter-1-walking.cost)*1000)/u64::from(running.cost-walking.cost)).min(1000) as u32
    };
    let cost=walking.cost+((running.cost-walking.cost)*tenths).div_ceil(1000);
    Ok(Onward {start:if before {"Osvald recruitment spot".into()} else {"Southern Cape Cold Snows encounter exit".into()},
        destination:"Eastern Cape Cold Snows entrance",initial_counter:counter,walking,running,
        running_distance_percent:f64::from(tenths)/10.0,walking_distance_percent:f64::from(1000-tenths)/10.0,
        estimated_cost:cost,estimated_counter_at_exit:counter-cost})
}

#[derive(Debug,Serialize)]
pub struct AfterRare {
    pub onward:Onward,
    pub eastern_entry:Entry,
    pub eastern_encounter:encounter::Prediction,
    pub passage:crate::eastern_escape::Passage,
}

/// Project the actual victory, support skill and map load, before searching Escape.
/// Upstream Southern rare selection must reject a surprised Eastern opening here.
pub fn southern_rare_entry(v:&crate::snows_rare::Victory,trigger:u32,_level:u32)->Result<(Onward,Entry,encounter::Prediction)> {
    let mut state=v.world_after.clone();
    let onward=onward_with_support(state.encounter_counter,state.time,false,trigger,state.evasive_maneuvers)?;
    state.encounter_counter=onward.estimated_counter_at_exit;
    let mut l=Loader::new(world::catalog(),state,v.after)?;
    l.visit_map(DESTINATION)?;
    let entry=Entry {arrival:v.after,after:l.rng,world_after:l.state,trace:l.trace,decisions:l.decisions};
    let mut rng=entry.after;
    let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions {
        map:DESTINATION.into(),zone:"ENT_FLD_SNW_1_3_LND".into(),period:if entry.world_after.time==2 {"DayTime"}else{"Night"}.into(),progress:entry.world_after.main_progress_points,
        first_encounter:entry.world_after.first_encounter,boundary:Boundary::CurrentCounter,current_counter:Some(entry.world_after.encounter_counter),
        party_levels:v.party_levels_after.clone(),dash:true,ability_percent:if entry.world_after.evasive_maneuvers {50}else{100},..Default::default()})?;
    Ok((onward,entry,encounter))
}

pub fn after_southern_rare(v:&crate::snows_rare::Victory,trigger:u32,level:u32)->Result<AfterRare> {
    let (onward,eastern_entry,eastern_encounter)=southern_rare_entry(v,trigger,level)?;
    let passage=crate::eastern_escape::escape_to_western(&eastern_entry,&eastern_encounter,level)?;
    Ok(AfterRare {onward,eastern_entry,eastern_encounter,passage})
}

fn continue_eastern_rare(p:&mut Prediction,v:&crate::snows_rare::Victory)->Result<bool> {
                    // Evasive Maneuvers changes the remaining Eastern traversal.
                    let mut loader=Loader::new(world::catalog(),v.world_after.clone(),v.after)?;
                    let left=crate::eastern_escape::RUN_UPDATES.saturating_sub(p.eastern_encounter.counter_updates.unwrap());
                    let d=encounter::catalog().counter_decrement(&PredictionOptions {map:DESTINATION.into(),zone:"ENT_FLD_SNW_1_3_LND".into(),period:if loader.state.time==2 {"DayTime"}else{"Night"}.into(),dash:true,ability_percent:50,..Default::default()})?;
                    if loader.state.encounter_counter<=left*d {return Ok(false);}
                    loader.state.encounter_counter-=left*d;
                    loader.visit_map(crate::eastern_escape::DESTINATION)?;
                    let west=Entry {arrival:v.after,after:loader.rng,world_after:loader.state,trace:loader.trace,decisions:loader.decisions};
                    let Some(route)=crate::winterbloom::with_evasive(&west)? else {return Ok(false)};
                    // This passage has no Eastern Escape: keep the actual rare
                    // return in p.rare_victory and store only the Western route.
                    p.winterbloom_after_eastern=Some(route);
                    p.western_entry_after_eastern=Some(west);
    Ok(true)
}

pub fn predict(branch:&EncounterBranch,level:u32,before:bool)->Result<Prediction> {
    predict_route(branch,level,before)
}

struct CachedContinuation {
    prediction:Option<Prediction>,
    eastern:Option<crate::eastern_escape::Search>,
}
impl CachedContinuation {
    fn at_cost(&mut self,level:u32,cost:u32)->Result<Option<Prediction>> {
        let p=self.prediction.as_ref().unwrap();
        if crate::snows_rare::target_met(&p.eastern_encounter) {
            if cost%2!=0 {return Ok(None);}
            let init=CrtRng {state:p.eastern_encounter.final_state,draws:p.eastern_entry.after.draws+p.eastern_encounter.draws};
            if let Some(v)=crate::snows_rare::kill_with_repeats(init,&p.eastern_entry.world_after,level,&p.eastern_encounter,cost/2)? {
                if !continue_eastern_rare(self.prediction.as_mut().unwrap(),&v)? {return Ok(None);}
                let mut p=self.prediction.take().unwrap();p.rare_victory=Some(v);return Ok(Some(p));
            }
        }
        if self.eastern.is_none() {
            self.eastern=Some(crate::eastern_escape::Search::new(&p.eastern_entry,&p.eastern_encounter,level)?);
        }
        let Some(w)=self.eastern.as_mut().unwrap().at_cost(cost)? else {return Ok(None)};
        let mut p=self.prediction.take().unwrap();p.western=Some(w);Ok(Some(p))
    }
}

/// Cost layers jointly allocate inputs between the Southern and Eastern fights.
/// An earlier rare ends the route immediately; Western is a fallback only.
pub fn predict_route(branch:&EncounterBranch,level:u32,before:bool)->Result<Prediction> {
    predict_route_bounded(branch,level,before,None)?.ok_or_else(||SimError("Rare route exhausted".into()))
}
pub fn predict_route_bounded(branch:&EncounterBranch,level:u32,before:bool,limit:Option<u32>)->Result<Option<Prediction>> {
    let _profile=crate::profile::span("Snows route");
    let situation=match branch.encounter.situation.as_deref() {Some("Normal")=>Situation::Normal,Some("Preemptive")=>Situation::Preemptive,_=>return Err(SimError("Snows Escape requires the no-surprise target".into()))};
    let levels=if before {vec![level]}else{vec![level,1]};
    let enemies:Vec<_>=branch.encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
    let danger=branch.encounter.encounter_level.unwrap();
    let trigger=branch.encounter.counter_updates.unwrap();
    let mut search=FieldSearch::with_history(branch.before_battle_initialization,situation,&levels,&enemies,danger,branch.world.time==4,&branch.world.enemy_knowledge)?;
    let mut carried=branch.world.clone();carried.sync_progression()?;
    search.enable_spell_breaks(carried.party.iter().map(|id|carried.character_progression[id].character.clone()).collect());
    // Every Southern allocation is immutable. Its battle, field restoration,
    // recruitment and Eastern load do not depend on the later Eastern budget.
    let mut layers:Vec<Vec<CachedContinuation>>=vec![];
    for total in 0..=limit.unwrap_or(u32::MAX-1) {
        for southern_cost in 0..=total {
            if layers.len()<=southern_cost as usize {
            let mut layer=vec![];
            let selected=search.visit_unboosted_break_candidates(southern_cost,|candidate| {
                let a=candidate.attempt.as_ref().unwrap();
                let mut state=branch.world.clone();state.first_encounter=false;state.enemy_knowledge=candidate.enemy_knowledge();
                crate::highroad::apply_escape_resources(&mut state,candidate.attempt.as_ref().unwrap())?;
                let mut l=Loader::new(world::catalog(),state,a.after)?;
                l.restore_field_after_battle("Snows Escape field return")?;
                if !before && l.state.encounter_counter<=onward_updates(false,trigger).1*decrement(l.state.time,true)? {return Ok(None);}
                let returned=FieldReturn {arrival:a.after,after:l.rng,trace:l.trace,world_after:l.state};
                let p=match continuation(branch,level,before,candidate,returned) {Ok(p)=>p,Err(e) if e.0.starts_with("Cannot reach Eastern")=>return Ok(None),Err(e)=>return Err(e)};
                let mut cached=CachedContinuation {prediction:Some(p),eastern:None};
                // New Southern layers first appear with an Eastern budget of
                // zero. Stop before constructing siblings if this one wins.
                if let Some(p)=cached.at_cost(level,0)? {return Ok(Some(p));}
                layer.push(cached);
                Ok(None)
            })?;
            if let Some(p)=selected {return Ok(Some(p));}
            layers.push(layer);
            continue;
            }
            for candidate in &mut layers[southern_cost as usize] {
                if let Some(p)=candidate.at_cost(level,total-southern_cost)? {return Ok(Some(p));}
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
fn predict_with_target(branch:&EncounterBranch,level:u32,before:bool,require_cait:bool)->Result<Prediction> {
    ensure(branch.world.map==osvald::MAP && matches!(branch.world.time,2|4),"Invalid Snows field boundary")?;
    let situation=match branch.encounter.situation.as_deref() {
        Some("Normal")=>Situation::Normal,Some("Preemptive")=>Situation::Preemptive,
        _=>return Err(SimError("Snows Escape requires the no-surprise target".into())),
    };
    let levels=if before {vec![level]} else {vec![level,1]};
    let enemies:Vec<_>=branch.encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
    let volume=&encounter::catalog().volumes[branch.encounter.volume.as_ref().ok_or_else(||SimError("Missing Snows encounter volume".into()))?];
    let danger=volume.encounters.iter().rev().find(|r|r.table!="None" && r.progress<=1)
        .ok_or_else(||SimError("Missing Snows danger row".into()))?.danger;
    let trigger=branch.encounter.counter_updates.ok_or_else(||SimError("Missing Snows trigger distance".into()))?;
    let mut search=FieldSearch::with_history(branch.before_battle_initialization,situation,&levels,&enemies,danger,branch.world.time==4,&branch.world.enemy_knowledge)?;
    let mut carried=branch.world.clone();carried.sync_progression()?;
    search.enable_spell_breaks(carried.party.iter().map(|id|carried.character_progression[id].character.clone()).collect());
    let mut cost=0u32;
    loop {
        let mut selected=None;
        for mut candidate in search.unboosted_break_candidates(cost)? {
            let attempt=candidate.attempt.as_ref().unwrap();
            let mut state=branch.world.clone(); state.first_encounter=false;state.enemy_knowledge=candidate.enemy_knowledge();
                crate::highroad::apply_escape_resources(&mut state,candidate.attempt.as_ref().unwrap())?;
            let mut l=Loader::new(world::catalog(),state,attempt.after)?;
            l.restore_field_after_battle("Snows Escape field return")?;
            // When the encounter is after recruitment, require the remaining
            // measured running distance to be safe, including nighttime costs.
            if !before && l.state.encounter_counter<=onward_updates(false,trigger).1*decrement(l.state.time,true)? {continue;}
            candidate.tested_candidates=u64::from(cost/2+1);
            let returned=FieldReturn {arrival:attempt.after,after:l.rng,trace:l.trace,world_after:l.state};
            let projected=match continuation(branch,level,before,candidate,returned) {
                Ok(p)=>p,Err(e) if e.0.starts_with("Cannot reach Eastern")=>continue,Err(e)=>return Err(e),
            };
            if require_cait && !eastern_target_met(&projected.eastern_encounter) {continue;}
            selected=Some(projected);break;
        }
        if let Some(p)=selected {return Ok(p); }
        cost=cost.checked_add(1).ok_or_else(||SimError("Snows Escape search exhausted representable cost".into()))?;
    }
}

#[cfg(test)]
fn eastern_target_met(p:&encounter::Prediction)->bool {
    p.rare_family.as_deref()==Some("Cait") && p.enemies.iter().any(|e|e.enemy.name=="Cait")
        && matches!(p.situation.as_deref(),Some("Normal"|"Preemptive"))
}

fn continuation(branch:&EncounterBranch,level:u32,before:bool,escape:Escape,field_return:FieldReturn)->Result<Prediction> {
    let trigger=branch.encounter.counter_updates.unwrap();
    let mut rng=field_return.after;
    let mut state=field_return.world_after.clone();
    let mut recruitment=None;
    let mut remaining_to_osvald=None;
    if before {
        let remaining=osvald::RUN_UPDATES.saturating_sub(trigger);
        let movement=MovementBudget::new(state.encounter_counter,remaining,decrement(state.time,true)?);
        ensure(movement.reaches_reset,"Snows return counter cannot reach Osvald")?;
        state.encounter_counter=movement.counter_before_reset.unwrap();
        let recruited=osvald::recruit_from_field(rng,state)?;
        rng=recruited.after; state=recruited.world_after.clone();
        remaining_to_osvald=Some(movement); recruitment=Some(recruited);
    }
    let onward=onward(state.encounter_counter,state.time,before,trigger)?;
    state.encounter_counter=onward.estimated_counter_at_exit;
    let mut l=Loader::new(world::catalog(),state,rng)?;
    l.visit_map(DESTINATION)?;
    let eastern_entry=Entry {arrival:rng,after:l.rng,world_after:l.state,trace:l.trace,decisions:l.decisions};
    let mut encounter_rng=eastern_entry.after;
    let eastern_encounter=encounter::catalog().predict(&mut encounter_rng,&PredictionOptions {
        map:DESTINATION.into(),zone:"ENT_FLD_SNW_1_3_LND".into(),
        period:if eastern_entry.world_after.time==2 {"DayTime"} else {"Night"}.into(),progress:eastern_entry.world_after.main_progress_points,
        first_encounter:eastern_entry.world_after.first_encounter,boundary:Boundary::CurrentCounter,
        current_counter:Some(eastern_entry.world_after.encounter_counter),party_levels:vec![level,1],dash:true,
        ..Default::default()})?;
    Ok(Prediction {before_osvald:before,escape,field_return,recruitment,remaining_to_osvald,onward,eastern_entry,eastern_encounter,rare_victory:None,western:None,winterbloom_after_eastern:None,western_entry_after_eastern:None,
        verification:"Source-derived; new Snows battle/entry endpoints await retail validation. Minimize boost-button inputs plus 10 for a successful break, which spends zero BP. First Escape must succeed. The pre-Osvald branch carries the actual Escape state into recruitment and its normal B/A reset; no hypothetical recruitment state is propagated. After-Osvald Escape requires a safe remaining full run. Measured onward distance is 85 running / 90 walking updates to Eastern Cape Cold Snows, plus two updates of player-error allowance for each speed (87 running / 92 walking). After an encounter, apply the full two-update allowance once to the remaining distance; mixed-distance shares are interpolated estimates, rounded down with one counter point reserved. Map entry uses the progression-aware shared Loader. Eastern first encounter is projected from that real entry state; the route search requires a Cait and Normal/Preemptive. No intervening time/menu actions are assumed."})
}

#[cfg(test)]
mod tests {
    use super::*;
    // Deliberately retain the uncached cost-layer algorithm as an independent
    // regression oracle for candidate ordering, button paths and all CRT traces.
    fn uncached_route(branch:&EncounterBranch,level:u32,before:bool)->Result<Prediction> {
        let situation=if branch.encounter.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal};
        let levels=if before {vec![level]}else{vec![level,1]};
        let enemies:Vec<_>=branch.encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
        let trigger=branch.encounter.counter_updates.unwrap();
        let mut search=FieldSearch::with_history(branch.before_battle_initialization,situation,&levels,&enemies,branch.encounter.encounter_level.unwrap(),branch.world.time==4,&branch.world.enemy_knowledge)?;
        for total in 0..u32::MAX {
            for cost in 0..=total {
                for candidate in search.unboosted_break_candidates(cost)? {
                    let a=candidate.attempt.as_ref().unwrap();
                    let mut state=branch.world.clone();state.first_encounter=false;state.enemy_knowledge=candidate.enemy_knowledge();
                crate::highroad::apply_escape_resources(&mut state,candidate.attempt.as_ref().unwrap())?;
                    let mut l=Loader::new(world::catalog(),state,a.after)?;
                    l.restore_field_after_battle("Snows Escape field return")?;
                    if !before && l.state.encounter_counter<=onward_updates(false,trigger).1*decrement(l.state.time,true)? {continue;}
                    let f=FieldReturn {arrival:a.after,after:l.rng,trace:l.trace,world_after:l.state};
                    let mut p=match continuation(branch,level,before,candidate,f) {Ok(p)=>p,Err(e) if e.0.starts_with("Cannot reach Eastern")=>continue,Err(e)=>return Err(e)};
                    let init=CrtRng {state:p.eastern_encounter.final_state,draws:p.eastern_entry.after.draws+p.eastern_encounter.draws};
                    if crate::snows_rare::target_met(&p.eastern_encounter) { if (total-cost)%2!=0 {continue;} if let Some(v)=crate::snows_rare::kill_with_repeats(init,&p.eastern_entry.world_after,level,&p.eastern_encounter,(total-cost)/2)? {if !continue_eastern_rare(&mut p,&v)? {continue;}p.rare_victory=Some(v);return Ok(p);} }
                    if let Some(w)=crate::eastern_escape::within_range_eager(&p.eastern_entry,&p.eastern_encounter,level,0,total-cost)? {p.western=Some(w);return Ok(p);}
                }
            }
        }
        unreachable!()
    }
    #[test]
    fn cached_cost_layers_preserve_complete_uncached_routes_and_timing_overrides() {
        let mut checked=0;
        for seed in 0..64 {
            let p=crate::early_cait::project(CrtRng::new(seed),start_world(),5).unwrap();
            let Some(a)=p.snows_approach.as_ref() else {continue};
            for (before,b) in [(true,&a.before_osvald),(false,&a.after_osvald)] {
                if !b.non_surprise {continue;}
                let expected=match uncached_route(b,5,before) {
                    Ok(p)=>p,
                    Err(e)=>{assert_eq!(predict_route(b,5,before).unwrap_err().to_string(),e.to_string());continue;}
                };
                let actual=predict_route(b,5,before).unwrap();
                let cost=actual.escape.weighted_input_cost+actual.rare_victory.as_ref().map_or(0,|v|v.inputs.input_count as u32)+actual.western.as_ref().map_or(0,|p|p.escape.weighted_input_cost+p.rare_victory.inputs.input_count as u32);
                let bounded=predict_route_bounded(b,5,before,Some(cost)).unwrap().unwrap();
                assert_eq!(serde_json::to_value(&bounded).unwrap(),serde_json::to_value(&actual).unwrap());
                if cost>0 {assert!(predict_route_bounded(b,5,before,Some(cost-1)).unwrap().is_none());}
                assert_eq!(serde_json::to_value(actual).unwrap(),serde_json::to_value(expected).unwrap(),"seed {seed}, before={before}");
                checked+=1;
            }
            if checked>=8 {break;}
        }
        assert!(checked>=8);
    }
    // The previous production algorithm retained Southern continuations but
    // eagerly built siblings and recreated the Eastern battle each cost layer.
    fn previous_cached_route(branch:&EncounterBranch,level:u32,before:bool,max_cost:u32)->Result<Option<Prediction>> {
        let situation=if branch.encounter.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal};
        let levels=if before {vec![level]}else{vec![level,1]};
        let enemies:Vec<_>=branch.encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
        let trigger=branch.encounter.counter_updates.unwrap();
        let mut search=FieldSearch::with_history(branch.before_battle_initialization,situation,&levels,&enemies,branch.encounter.encounter_level.unwrap(),branch.world.time==4,&branch.world.enemy_knowledge)?;
        let mut layers:Vec<Vec<Option<Prediction>>>=vec![];
        for total in 0..=max_cost {
            for cost in 0..=total {
                if layers.len()<=cost as usize {
                    let mut layer=vec![];
                    for candidate in search.unboosted_break_candidates(cost)? {
                        let a=candidate.attempt.as_ref().unwrap();
                        let mut state=branch.world.clone();state.first_encounter=false;state.enemy_knowledge=candidate.enemy_knowledge();
                crate::highroad::apply_escape_resources(&mut state,candidate.attempt.as_ref().unwrap())?;
                        let mut l=Loader::new(world::catalog(),state,a.after)?;
                        l.restore_field_after_battle("Snows Escape field return")?;
                        if !before && l.state.encounter_counter<=onward_updates(false,trigger).1*decrement(l.state.time,true)? {continue;}
                        let f=FieldReturn {arrival:a.after,after:l.rng,trace:l.trace,world_after:l.state};
                        let p=match continuation(branch,level,before,candidate,f) {Ok(p)=>p,Err(e) if e.0.starts_with("Cannot reach Eastern")=>continue,Err(e)=>return Err(e)};
                        layer.push(Some(p));
                    }
                    layers.push(layer);
                }
                for candidate in &mut layers[cost as usize] {
                    let mut cached=CachedContinuation {prediction:candidate.take(),eastern:None};
                    if let Some(p)=cached.at_cost(level,total-cost)? {return Ok(Some(p));}
                    *candidate=cached.prediction;
                }
            }
        }
        Ok(None)
    }
    #[test]
    fn streaming_joint_search_matches_previous_complete_routes_for_both_timings() {
        // The active route returns to daytime after clerics, with no later
        // time switch. Cover both encounter/recruitment timings on that route.
        for switches in [0] {
            let mut covered=[false;2];
            for seed in 0..64 {
                let early=crate::early_cait::project_with_switches(CrtRng::new(seed),start_world(),5,switches).unwrap();
                let Some(a)=early.snows_approach.as_ref() else {continue};
                for (before,b) in [(true,&a.before_osvald),(false,&a.after_osvald)] {
                    let index=usize::from(before);
                    if covered[index] || !b.non_surprise {continue;}
                    // Find inexpensive oracle fixtures instead of making the
                    // regular suite run an unbounded exhaustive rare search.
                    let Some(expected)=previous_cached_route(b,5,before,16).unwrap() else {continue};
                    let actual=predict_route(b,5,before).unwrap();
                    assert_eq!(serde_json::to_value(actual).unwrap(),serde_json::to_value(expected).unwrap(),"seed {seed}, before={before}, switches={switches}");
                    covered[index]=true;
                }
                if covered==[true;2] {break;}
            }
            assert_eq!(covered,[true;2]);
        }
    }
    #[test]
    fn southern_rare_keeps_eastern_escape_with_evasive_and_no_later_rare_target() {
        for seed in 0..1024 {
            let early=crate::early_cait::project(CrtRng::new(seed),start_world(),5).unwrap();
            if !early.southern_rare_target {continue;}
            let p=early.encounter.as_ref().unwrap();
            let Some(v)=crate::snows_rare::kill(early.before_battle_initialization.unwrap(),early.world_at_encounter.as_ref().unwrap(),5,p).unwrap() else {continue};
            let (_,entry,e)=southern_rare_entry(&v,p.counter_updates.unwrap(),5).unwrap();
            if !matches!(e.situation.as_deref(),Some("Normal"|"Preemptive")) {continue;}
            let continued=after_southern_rare(&v,p.counter_updates.unwrap(),5).unwrap();
            assert!(continued.onward.running.reaches_reset);
            assert!(entry.world_after.evasive_maneuvers);
            assert_eq!(e.decrement,Some(6));
            let exit=&continued.passage;
            assert!(exit.escape.attempt.as_ref().unwrap().success);
            assert_eq!(exit.remaining_run.decrement,6);
            assert!(exit.remaining_run.reaches_reset);
            assert!(exit.remaining_run.cost<1000,"Even the minimum normal post-encounter counter suffices");
            assert_eq!(exit.western_entry.world_after.map,crate::eastern_escape::DESTINATION);
            assert!(exit.western_entry.world_after.evasive_maneuvers);
            let mut loader=Loader::new(world::catalog(),v.world_after.clone(),v.after).unwrap();
            loader.state.encounter_counter=continued.onward.estimated_counter_at_exit;
            loader.visit_map(DESTINATION).unwrap();
            assert_eq!(loader.rng,continued.eastern_entry.after);
            let mut surprised=e;surprised.situation=Some("Surprise".into());
            assert!(crate::eastern_escape::escape_to_western(&entry,&surprised,5).is_err());
            return;
        }
        panic!("No Southern rare/Eastern safe continuation fixture found");
    }
    #[test]
    fn eastern_cait_target_replays_both_recruitment_timings_and_periods() {
        for switches in [0,1] {
            let mut covered=[false;2];
            for seed in 0..64 {
                let early=crate::early_cait::project_with_switches(CrtRng::new(seed),start_world(),5,switches).unwrap();
                let Some(a)=&early.snows_approach else {continue};
                for (before,branch) in [(true,&a.before_osvald),(false,&a.after_osvald)] {
                    if covered[usize::from(before)] || !branch.non_surprise {continue;}
                    let p=predict_with_target(branch,5,before,true).unwrap();
                    assert!(p.escape.attempt.as_ref().unwrap().success);
                    assert!(eastern_target_met(&p.eastern_encounter));
                    assert!(!p.eastern_encounter.first_override);
                    assert!(p.eastern_encounter.enemies.iter().any(|e|e.enemy.name=="Cait"));
                    let mut r=p.eastern_entry.after;
                    let independent=encounter::catalog().predict(&mut r,&PredictionOptions {
                        map:DESTINATION.into(),zone:"ENT_FLD_SNW_1_3_LND".into(),progress:1,
                        period:if p.eastern_entry.world_after.time==2 {"DayTime"} else {"Night"}.into(),
                        boundary:Boundary::CurrentCounter,current_counter:Some(p.eastern_entry.world_after.encounter_counter),
                        first_encounter:true,party_levels:vec![5,1],dash:true,..Default::default()}).unwrap();
                    assert_eq!(independent.final_state,p.eastern_encounter.final_state);
                    assert_eq!(independent.group,p.eastern_encounter.group);
                    assert_eq!(independent.situation,p.eastern_encounter.situation);
                    assert_eq!(p.eastern_encounter.checkpoints[0].state,p.eastern_entry.after.state);
                    covered[usize::from(before)]=true;
                }
                if covered==[true;2] {break;}
            }
            assert_eq!(covered,[true;2]);
        }
    }
    fn start_world()->WorldState {
        let mut w=world::throne_after_second_state().unwrap();
        // Flats and every continuation occur after Throné's first chapter.
        if w.completed_scenarios.insert(700) {w.main_progress_points+=1;}
        w.set_flag(8019,true);
        w.set_flag(36,true); // actual completed Throné Chapter 1 latent release
        w.map="Fld_Cty_1_2".into();w.time=2;w.party=vec![7];w.bench.clear();
        w.active_scenarios.clear();w.prologues.clear();w.story_types.clear();
        w.first_encounter=false;w.encounter_counter=515;
        w.party_experience=vec![ot2_battle_engine::field_escape::experience_for_level(5).unwrap()];
        w.visited_maps.insert("Fld_Cty_1_1".into());
        w.resident_levels=world::catalog().maps[&w.map].streams.iter().cloned().collect();w
    }
    #[test]
    fn real_escape_recruitment_reset_and_next_load_have_continuous_rng() {
        let mut covered=[false;2];
        let mut broke=false;
        for seed in 0..4096 {
            let early=crate::early_cait::project(CrtRng::new(seed),start_world(),5).unwrap();
            let Some(a)=&early.snows_approach else {continue};
            for (before,branch) in [(true,&a.before_osvald),(false,&a.after_osvald)] {
                if !branch.non_surprise {continue;}
                let p=predict_with_target(branch,5,before,false).unwrap();
                covered[usize::from(before)]=true;
                assert!(p.escape.attempt.as_ref().unwrap().success);
                assert!(!p.field_return.world_after.first_encounter);
                if let Some(b)=&p.escape.break_action {broke=true;assert_eq!(b.boost,0);}
                let mut previous=p.escape.menu.trace.first().unwrap().before;
                for t in &p.escape.attempt.as_ref().unwrap().trace {
                    assert_eq!(t.before,previous);previous=t.after;
                }
                for t in &p.field_return.trace {assert_eq!(t.before,previous);previous=t.after;}
                if let Some(r)=&p.recruitment {
                    assert_eq!(r.arrival,previous);
                    for t in &r.trace {assert_eq!(t.before,previous);previous=t.after;}
                    assert!(!r.world_after.first_encounter);
                    assert!(r.world_after.encounter_counter>=1000);
                    let mut reset=r.arrival;let b=reset.index(400);let a=reset.index(400);
                    assert_eq!(reset,r.after);
                    assert_eq!(r.world_after.encounter_counter,1000+b+a);
                }
                for t in &p.eastern_entry.trace {assert_eq!(t.before,previous);previous=t.after;}
                assert_eq!(previous,p.eastern_entry.after);
                assert_eq!(p.eastern_entry.world_after.map,DESTINATION);
                assert!(p.eastern_entry.world_after.first_encounter);
                assert!((500..=899).contains(&p.eastern_entry.world_after.encounter_counter));
                assert_eq!(p.eastern_entry.world_after.party,[7,6]);
                if !before {assert!(p.onward.running.reaches_reset);}
                let levels=if before {vec![5]} else {vec![5,1]};
                let enemies:Vec<_>=branch.encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
                let mut search=FieldSearch::with_party(branch.before_battle_initialization,
                    if branch.encounter.situation.as_deref()==Some("Preemptive") {Situation::Preemptive} else {Situation::Normal},
                    &levels,&enemies,5,false).unwrap();
                for cost in 0..p.escape.weighted_input_cost {
                    assert!(search.unboosted_break_candidates(cost).unwrap().is_empty(),"Selected a slower Escape strategy");
                }
            }
        }
        assert_eq!(covered,[true;2]);assert!(broke,"Exercise a case where a zero-BP break is faster");
    }
    #[test]
    fn recruitment_counter_reserves_one_point_and_uses_both_measured_speeds() {
        let p=onward(1000,2,true,50).unwrap();
        assert_eq!((p.walking.cost,p.running.cost),(828,1044));
        assert_eq!(p.running_distance_percent,79.1);
        assert_eq!(p.estimated_counter_at_exit,1);
        assert_eq!(onward(1045,2,true,50).unwrap().running_distance_percent,100.0);
        for trigger in 42..=75 {
            assert_eq!(onward_updates(false,trigger),
                ((90*(85-trigger)).div_ceil(85)+2,85-trigger+2));
            assert!(onward(1000,2,false,trigger).unwrap().running.reaches_reset);
        }
    }
    #[test]
    fn nighttime_post_recruitment_escape_still_leaves_a_safe_full_run() {
        for seed in 0..64 {
            let early=crate::early_cait::project_with_switches(CrtRng::new(seed),start_world(),5,1).unwrap();
            let Some(a)=&early.snows_approach else {continue};
            if !a.after_osvald.non_surprise {continue;}
            let p=predict_with_target(&a.after_osvald,5,false,false).unwrap();
            assert_eq!(p.field_return.world_after.time,4);
            assert!(p.onward.running.reaches_reset);
            assert_eq!(p.eastern_entry.world_after.time,4);
            return;
        }
        panic!("No testable nighttime opening");
    }
}
