//! Eastern first encounter -> one successful Escape -> full run -> Western rare.
use crate::{CrtRng,Result,encounter::{self,Boundary,PredictionOptions},escape_search::FieldSearch,highroad::{Escape,FieldReturn},route::MovementBudget,snows_escape::Entry,snows_rare,world::{self,Loader}};
use ot2_battle_engine::escape::Situation;
use serde::Serialize;
pub const RUN_UPDATES:u32=167;
pub const WALK_UPDATES:u32=178;
pub const DESTINATION:&str="Fld_Snw_2_2";
#[derive(Debug,Serialize)]
pub struct Passage {
    pub escape:Escape,
    pub field_return:FieldReturn,
    pub remaining_run:MovementBudget,
    pub western_entry:Entry,
    pub winterbloom:Option<crate::winterbloom::Prediction>,
}
#[derive(Debug,Serialize)]
pub struct Prediction {
    pub escape:Escape,
    pub field_return:FieldReturn,
    pub remaining_run:MovementBudget,
    pub western_entry:Entry,
    pub western_encounter:encounter::Prediction,
    pub rare_victory:snows_rare::Victory,
    pub winterbloom:crate::winterbloom::Prediction,
}

/// Bounded total-cost layer supplied by the upstream joint search, never an
/// arbitrary required-outcome retry cap. Return the cheapest fitting plan.
pub fn within_cost(entry:&Entry,p:&encounter::Prediction,level:u32,max_cost:u32)->Result<Option<Prediction>> {
    let mut search=Search::new(entry,p,level)?;
    for cost in 0..=max_cost {
        if let Some(p)=search.at_cost(cost)? {return Ok(Some(p));}
    }
    Ok(None)
}

/// One immutable Eastern boundary keeps its battle initialization and opening
/// break cache across the later cost layers of the joint Snows search.
pub(crate) struct Search {
    context:Option<Context>,
}
struct Context {
    field:FieldSearch,
    world:world::WorldState,
    remaining:u32,
    decrement:u32,
    level:u32,
}
impl Search {
    pub(crate) fn new(entry:&Entry,p:&encounter::Prediction,level:u32)->Result<Self> {
        let situation=match p.situation.as_deref() {Some("Normal")=>Situation::Normal,Some("Preemptive")=>Situation::Preemptive,_=>return Ok(Self {context:None})};
        let init=CrtRng {state:p.final_state,draws:entry.after.draws+p.draws};
        let ids:Vec<_>=p.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
        let levels=if entry.world_after.evasive_maneuvers {
            entry.world_after.party_experience.iter().map(|&e|ot2_battle_engine::field_escape::level_for_experience(e)).collect::<Vec<_>>()
        } else {vec![level,1]};
        let mut field=FieldSearch::with_history(init,situation,&levels,&ids,p.encounter_level.unwrap(),entry.world_after.time==4,&entry.world_after.enemy_knowledge)?;
        let mut carried=entry.world_after.clone();carried.sync_progression()?;
        field.enable_spell_breaks(carried.party.iter().map(|id|carried.character_progression[id].character.clone()).collect());
        Ok(Self {context:Some(Context {field,world:entry.world_after.clone(),remaining:RUN_UPDATES.saturating_sub(p.counter_updates.unwrap()),decrement:p.decrement.unwrap(),level})})
    }
    pub(crate) fn at_cost(&mut self,cost:u32)->Result<Option<Prediction>> {
        let _profile=crate::profile::span("Eastern cost layer");
        let Some(context)=self.context.as_ref() else {return Ok(None)};
        let level=context.level;
        for escape_cost in 0..=cost {
            if (cost-escape_cost)%2!=0 {continue;}
            if let Some(p)=self.visit(escape_cost,|passage| western_rare(passage,level,(cost-escape_cost)/2))? {return Ok(Some(p));}
        }
        Ok(None)
    }
    fn first_passage(&mut self,cost:u32)->Result<Option<Passage>> {
        self.visit(cost,|passage|Ok(Some(passage)))
    }
    fn visit<T>(&mut self,cost:u32,mut accept:impl FnMut(Passage)->Result<Option<T>>)->Result<Option<T>> {
        let Some(context)=self.context.as_mut() else {return Ok(None)};
        let Context {field,world,remaining,decrement,..}=context;
        field.visit_unboosted_break_candidates(cost,|candidate| {
            let Some(passage)=passage(world,*remaining,*decrement,candidate)? else {return Ok(None)};
            accept(passage)
        })
    }
}

fn western_rare(passage:Passage,level:u32,repeats:u32)->Result<Option<Prediction>> {
    let west=passage.western_entry;
    let mut rng=west.after;
    let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions {map:DESTINATION.into(),zone:"ENT_FLD_SNW_2_2_LND".into(),period:if west.world_after.time==2 {"DayTime"}else{"Night"}.into(),progress:west.world_after.main_progress_points,first_encounter:west.world_after.first_encounter,boundary:Boundary::CurrentCounter,current_counter:Some(west.world_after.encounter_counter),party_levels:vec![level,1],dash:true,ability_percent:if west.world_after.evasive_maneuvers {50}else{100},..Default::default()})?;
    let Some(victory)=snows_rare::kill_with_repeats(rng,&west.world_after,level,&encounter,repeats)? else {return Ok(None)};
    let Some(winterbloom)=crate::winterbloom::after_rare(&west,encounter.clone(),&victory)? else {return Ok(None)};
    Ok(Some(Prediction {escape:passage.escape,field_return:passage.field_return,remaining_run:passage.remaining_run,western_entry:west,western_encounter:encounter,rare_victory:victory,winterbloom}))
}

/// Once an earlier rare is secured, search only for Escape and safe traversal.
/// No Western rare lottery is part of this continuation.
pub fn escape_to_western(entry:&Entry,p:&encounter::Prediction,level:u32)->Result<Passage> {
    crate::ensure(matches!(p.situation.as_deref(),Some("Normal"|"Preemptive")),"Eastern encounter violates mandatory no-surprise rule")?;
    let mut search=Search::new(entry,p,level)?;
    for cost in 0..u32::MAX {
        if let Some(passage)=search.first_passage(cost)? {return Ok(passage);}
    }
    Err(crate::SimError("Eastern Escape search exhausted representable cost".into()))
}

fn passage(world:&world::WorldState,remaining:u32,decrement:u32,candidate:Escape)->Result<Option<Passage>> {
    let attempt=candidate.attempt.as_ref().unwrap();
    let mut state=world.clone();state.first_encounter=false;state.enemy_knowledge=candidate.enemy_knowledge();
    crate::highroad::apply_escape_resources(&mut state,attempt)?;
    let mut l=Loader::new(world::catalog(),state,attempt.after)?;
    l.restore_field_after_battle("Eastern Cape Cold Snows Escape field return")?;
    let movement=MovementBudget::new(l.state.encounter_counter,remaining,decrement);
    if !movement.reaches_reset {return Ok(None);}
    let returned=FieldReturn {arrival:attempt.after,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace)};
    l.state.encounter_counter=movement.counter_before_reset.unwrap();
    let arrival=l.rng;
    l.visit_map(DESTINATION)?;
    let west=Entry {arrival,after:l.rng,world_after:l.state,trace:l.trace,decisions:l.decisions};
    let winterbloom=if west.world_after.evasive_maneuvers {
        let Some(route)=crate::winterbloom::with_evasive(&west)? else {return Ok(None)};Some(route)
    }else{None};
    Ok(Some(Passage {escape:candidate,field_return:returned,remaining_run:movement,western_entry:west,winterbloom}))
}

// Preserve the eager implementation as a full-result oracle for streaming and
// retained battle contexts; production never builds the unused continuations.
#[cfg(test)]
fn passages_eager(entry:&Entry,p:&encounter::Prediction,level:u32,min_cost:u32,max_cost:u32)->Result<Vec<Passage>> {
    let situation=match p.situation.as_deref() {Some("Normal")=>Situation::Normal,Some("Preemptive")=>Situation::Preemptive,_=>return Ok(vec![])};
    let mut out=vec![];
    let trigger=p.counter_updates.unwrap();
    let remaining=RUN_UPDATES.saturating_sub(trigger);
    let init=CrtRng {state:p.final_state,draws:entry.after.draws+p.draws};
    let ids:Vec<_>=p.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
    let danger=p.encounter_level.unwrap();
    let levels=if entry.world_after.evasive_maneuvers {
        entry.world_after.party_experience.iter().map(|&e|ot2_battle_engine::field_escape::level_for_experience(e)).collect::<Vec<_>>()
    } else {vec![level,1]};
    let mut search=FieldSearch::with_history(init,situation,&levels,&ids,danger,entry.world_after.time==4,&entry.world_after.enemy_knowledge)?;
    let mut carried=entry.world_after.clone();carried.sync_progression()?;
    search.enable_spell_breaks(carried.party.iter().map(|id|carried.character_progression[id].character.clone()).collect());
    for cost in min_cost..=max_cost {
        for candidate in search.unboosted_break_candidates(cost)? {
            let attempt=candidate.attempt.as_ref().unwrap();
            let mut state=entry.world_after.clone();state.first_encounter=false;state.enemy_knowledge=candidate.enemy_knowledge();
            crate::highroad::apply_escape_resources(&mut state,attempt)?;
            let mut l=Loader::new(world::catalog(),state,attempt.after)?;
            l.restore_field_after_battle("Eastern Cape Cold Snows Escape field return")?;
            let movement=MovementBudget::new(l.state.encounter_counter,remaining,p.decrement.unwrap());
            if !movement.reaches_reset {continue;}
            let returned=FieldReturn {arrival:attempt.after,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace)};
            l.state.encounter_counter=movement.counter_before_reset.unwrap();
            let arrival=l.rng;
            l.visit_map(DESTINATION)?;
            let west=Entry {arrival,after:l.rng,world_after:l.state,trace:l.trace,decisions:l.decisions};
            let winterbloom=if west.world_after.evasive_maneuvers {
                let Some(route)=crate::winterbloom::with_evasive(&west)? else {continue};Some(route)
            }else{None};
            out.push(Passage {escape:candidate,field_return:returned,remaining_run:movement,western_entry:west,winterbloom});
        }
    }
    Ok(out)
}

#[cfg(test)]
pub(crate) fn within_range_eager(entry:&Entry,p:&encounter::Prediction,level:u32,min_cost:u32,max_cost:u32)->Result<Option<Prediction>> {
    for cost in min_cost..=max_cost {
        for escape_cost in 0..=cost {
            if (cost-escape_cost)%2!=0 {continue;}
            for passage in passages_eager(entry,p,level,escape_cost,escape_cost)? {
                if let Some(p)=western_rare(passage,level,(cost-escape_cost)/2)? {return Ok(Some(p));}
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eastern_escape_carries_full_counter_and_real_load_to_one_western_rare() {
        let mut state=world::throne_after_second_state().unwrap();
        if state.completed_scenarios.insert(700) {state.main_progress_points+=1;}
        state.set_flag(8019,true);
        for flag in [36,35] {state.set_flag(flag,true);} // completed/skipped Ch1 latent releases
        state.visited_maps.insert(crate::crackridge::NEW_DELSTA_HARBOR.into());
        state.map="Fld_Snw_1_3".into();state.party=vec![7,6];state.time=2;state.first_encounter=true;state.encounter_counter=750;
        state.active_scenarios.clear();state.story_types.clear();state.prologues.clear();
        state.party_experience=vec![ot2_battle_engine::field_escape::experience_for_level(5).unwrap(),0];
        state.resident_levels=world::catalog().maps[&state.map].streams.iter().cloned().collect();
        for seed in 0..128 {
            let entry=Entry {arrival:CrtRng::new(seed),after:CrtRng::new(seed),world_after:state.clone(),trace:vec![],decisions:vec![]};
            let mut rng=entry.after;
            let p=encounter::catalog().predict(&mut rng,&PredictionOptions {map:state.map.clone(),zone:"ENT_FLD_SNW_1_3_LND".into(),period:"DayTime".into(),progress:1,first_encounter:true,boundary:Boundary::CurrentCounter,current_counter:Some(750),party_levels:vec![5,1],dash:true,..Default::default()}).unwrap();
            if snows_rare::target_met(&p) {continue;}
            let Some(w)=within_cost(&entry,&p,5,8).unwrap() else {continue};
            let expected=within_range_eager(&entry,&p,5,0,8).unwrap().unwrap();
            assert_eq!(serde_json::to_value(&w).unwrap(),serde_json::to_value(expected).unwrap());
            assert!(w.escape.attempt.as_ref().unwrap().success);
            assert!(w.remaining_run.reaches_reset);
            assert_eq!(w.remaining_run.updates,RUN_UPDATES-p.counter_updates.unwrap());
            assert_eq!(w.remaining_run.decrement,12);
            assert_eq!(w.western_entry.world_after.map,DESTINATION);
            assert!(snows_rare::target_met(&w.western_encounter));
            assert!(w.rare_victory.evasive_maneuvers);
            let mut loader=Loader::new(world::catalog(),w.field_return.world_after.clone(),w.field_return.after).unwrap();
            loader.state.encounter_counter=w.remaining_run.counter_before_reset.unwrap();
            loader.visit_map(DESTINATION).unwrap();
            assert_eq!(loader.rng,w.western_entry.after);
            assert_eq!(serde_json::to_value(loader.state).unwrap(),serde_json::to_value(&w.western_entry.world_after).unwrap());
            return;
        }
        panic!("No joint Eastern escape/run/Western rare fixture found");
    }

    #[test]
    fn retained_eastern_context_matches_eager_passages_across_costs_and_support() {
        let mut state=world::throne_after_second_state().unwrap();
        if state.completed_scenarios.insert(700) {state.main_progress_points+=1;}
        state.set_flag(8019,true);
        for flag in [36,35] {state.set_flag(flag,true);} // completed/skipped Ch1 latent releases
        state.visited_maps.insert(crate::crackridge::NEW_DELSTA_HARBOR.into());
        state.map="Fld_Snw_1_3".into();state.party=vec![7,6];state.first_encounter=true;state.encounter_counter=750;
        state.active_scenarios.clear();state.story_types.clear();state.prologues.clear();
        state.party_experience=vec![ot2_battle_engine::field_escape::experience_for_level(5).unwrap(),0];
        state.resident_levels=world::catalog().maps[&state.map].streams.iter().cloned().collect();
        let mut successful=0;
        let mut matching_errors=0;
        for time in [2,4] {
            state.time=time;
            for evasive in [false,true] {
                // The active post-rare route stays daytime; use the same
                // downstream world instead of inventing a nighttime voyage.
                if time==4 && evasive {continue;}
                let successful_before=successful;
                state.evasive_maneuvers=evasive;
                let experience=ot2_battle_engine::field_escape::experience_for_level(if evasive {15}else{5}).unwrap();
                state.party_experience=vec![experience,if evasive {experience}else{0}];
                state.sync_progression().unwrap();
                if evasive {state.character_progression.get_mut(&7).unwrap().jp_earned=130;}
                for seed in 0..4 {
                    let entry=Entry {arrival:CrtRng::new(seed),after:CrtRng::new(seed),world_after:state.clone(),trace:vec![],decisions:vec![]};
                    let mut rng=entry.after;
                    let p=encounter::catalog().predict(&mut rng,&PredictionOptions {map:state.map.clone(),zone:"ENT_FLD_SNW_1_3_LND".into(),period:if time==2 {"DayTime"}else{"Night"}.into(),progress:1,first_encounter:true,boundary:Boundary::CurrentCounter,current_counter:Some(750),party_levels:vec![5,1],dash:true,ability_percent:if evasive {50}else{100},..Default::default()}).unwrap();
                    let mut search=Search::new(&entry,&p,5).unwrap();
                    // Repeated and earlier cost layers must remain valid too;
                    // speculative attempts never mutate the immutable boundary.
                    for cost in [0,1,2,4,10,12,4,0] {
                        let expected=passages_eager(&entry,&p,5,cost,cost).map(|all|all.into_iter().next());
                        let actual=search.first_passage(cost);
                        match (actual,expected) {
                            (Ok(actual),Ok(expected))=> {
                                successful+=usize::from(actual.is_some());
                                assert_eq!(serde_json::to_value(actual).unwrap(),serde_json::to_value(expected).unwrap(),"seed {seed}, cost {cost}, time {time}, evasive {evasive}");
                            }
                            (Err(actual),Err(expected))=> {matching_errors+=1;assert_eq!(actual.0,expected.0);}
                            (actual,expected)=>panic!("Different results at seed {seed}, cost {cost}, time {time}, evasive {evasive}: {actual:?} / {expected:?}"),
                        }
                    }
                }
                if time==4 {
                    // At this entry counter the remaining nighttime run costs
                    // more than the largest field reset; both reject it.
                    assert_eq!(successful,successful_before);
                } else {
                    assert!(successful>successful_before,"No completed passage for time {time}, evasive {evasive}");
                }
            }
        }
        assert!(successful>0);
        assert_eq!(matching_errors,0,"All formations reached by this route fixture must now be supported");
    }
}
