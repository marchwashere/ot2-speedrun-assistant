//! Western Snows entry -> guild license -> Winterbloom tag, 345 running updates.
//! Movement-error override is a separate immutable second-encounter continuation.
use crate::{CrtRng,Result,SimError,ensure,encounter::{self,Boundary,PredictionOptions},
    escape_search::FieldSearch,highroad::{Escape,FieldReturn},snows_escape::Entry,
    snows_rare,world::{self,Loader,WorldState}};
use ot2_battle_engine::escape::Situation;
use serde::Serialize;
pub const RUN_UPDATES:u32=345;
pub const MAP:&str="Fld_Snw_2_2";
pub const TOWN:&str="Twn_Snw_2_1_A";

#[derive(Debug,Serialize)]
pub struct Flee {
    pub encounter:encounter::Prediction,
    pub escape:Escape,
    pub field_return:FieldReturn,
    pub remaining_updates:u32,
    pub town:Entry,
}
#[derive(Debug,Serialize)]
pub struct Prediction {
    pub evasive_at_entry:bool,
    pub first_encounter:encounter::Prediction,
    pub first_escape:Option<Escape>,
    pub first_field_return:Option<FieldReturn>,
    pub first_is_rare:bool,
    pub first_trigger_updates:u32,
    pub remaining_updates:u32,
    pub predicted_second_encounter:bool,
    pub second_encounter:encounter::Prediction,
    pub without_second:Option<Entry>,
    pub with_second:Flee,
    pub voyage_without_second:Option<crate::winter_voyage::Prediction>,
    pub voyage_with_second:crate::winter_voyage::Prediction,
    pub verification:&'static str,
}
impl Prediction {
    pub fn input_cost(&self)->u32 {
        self.first_escape.as_ref().map_or(0,|s|s.weighted_input_cost)
            +if self.predicted_second_encounter {self.with_second.escape.weighted_input_cost}else{0}
    }
}

pub fn encounter_at(rng:CrtRng,state:&WorldState)->Result<encounter::Prediction> {
    let mut state=state.clone();state.sync_progression()?;
    let mut local=CrtRng::new(rng.state);
    encounter::catalog().predict(&mut local,&PredictionOptions {
        map:MAP.into(),zone:"ENT_FLD_SNW_2_2_LND".into(),
        period:if state.time==2 {"DayTime"}else{"Night"}.into(),progress:state.main_progress_points,
        first_encounter:state.first_encounter,boundary:Boundary::CurrentCounter,
        current_counter:Some(state.encounter_counter),
        party_levels:state.party.iter().map(|id|state.character_progression[id].level).collect(),
        dash:true,ability_percent:if state.evasive_maneuvers {50}else{100},..Default::default()
    })
}
fn init(rng:CrtRng,p:&encounter::Prediction)->CrtRng {
    CrtRng {state:p.final_state,draws:rng.draws+p.draws}
}
fn safe(p:&encounter::Prediction)->bool {matches!(p.situation.as_deref(),Some("Normal"|"Preemptive"))}
fn restore(a:CrtRng,state:&WorldState,history:ot2_battle_engine::knowledge::EnemyHistory)->Result<FieldReturn> {
    let mut state=state.clone();state.first_encounter=false;state.enemy_knowledge=history;
    let mut l=Loader::new(world::catalog(),state,a)?;
    l.restore_field_after_battle("Western Winterbloom Snows Escape field return")?;
    Ok(FieldReturn {arrival:a,after:l.rng,world_after:l.state,trace:l.trace})
}
fn town(rng:CrtRng,state:&WorldState,remaining:u32,decrement:u32)->Result<Entry> {
    ensure(state.encounter_counter>remaining*decrement,"Winterbloom approach would generate another encounter")?;
    let mut state=state.clone();state.encounter_counter-=remaining*decrement;
    let mut l=Loader::new(world::catalog(),state,rng)?;
    // GuildData GUILD_005: fixed dialogue + license flags, no reset or random line.
    l.skipped_event("SYS_SCH_GUILD_0010")?;
    l.skipped_event("SYS_SCH_GUILD_0030")?;
    // GuildMenuWidget.GetLicense -> TaskClear(0) separately grants the
    // source license item after the completion event; its JSON flags alone
    // are not the acquired-license record. Assignment does not consume it.
    l.state.job_licenses.entry(6).or_insert(1);
    l.state.set_flag(100,true); // native SetAcquiredJobLicense's shared saved event flag
    l.dismiss_tutorial(59)?;
    l.boundary("Scholar License acquired; guild interaction preserves the encounter counter");
    l.visit_map(TOWN)?;
    l.skipped_event("MapInStorySelect")?;
    l.boundary("Winterbloom: story prompt dismissed without starting chapter");
    l.manual_time_switch(4,"Winterbloom: switch to night after dismissing story prompt")?;
    Ok(Entry {arrival:rng,after:l.rng,world_after:l.state,trace:l.trace,decisions:l.decisions})
}

fn finish(entry:&Entry,first:encounter::Prediction,first_escape:Option<Escape>,
    first_return:Option<FieldReturn>,first_is_rare:bool,
    after:CrtRng,state:&WorldState)->Result<Option<Prediction>> {
    let _profile=crate::profile::span("Western finish");
    ensure(state.evasive_maneuvers,"Western onward route requires the rare kill and Evasive Maneuvers")?;
    let trigger=first.counter_updates.unwrap();
    ensure(trigger<=RUN_UPDATES,"Western first encounter lies beyond the measured route")?;
    let remaining=RUN_UPDATES-trigger;
    let second=encounter_at(after,state)?;
    // Protect the movement-error branch even when perfect movement avoids it.
    if !safe(&second) {return Ok(None);}
    let second_trigger=second.counter_updates.unwrap();
    let predicted=second_trigger<=remaining;
    let without_second=if predicted {None}else{Some(town(after,state,remaining,second.decrement.unwrap())?)};
    if let Some(destination)=&without_second {
        if !crate::winter_voyage::safe(destination)? {return Ok(None);}
    }
    let mut w=state.clone();w.sync_progression()?;
    let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
    let ids:Vec<_>=second.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
    let situation=if second.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal};
    let mut search=FieldSearch::with_history(init(after,&second),situation,&levels,&ids,second.encounter_level.unwrap(),w.time==4,&w.enemy_knowledge)?;
    let remaining_after=remaining.saturating_sub(second_trigger);
    for cost in 0..u32::MAX {
        for escape in search.unboosted_break_candidates(cost)? {
            let returned=restore(escape.attempt.as_ref().unwrap().after,&w,escape.enemy_knowledge())?;
            if returned.world_after.encounter_counter<=remaining_after*second.decrement.unwrap() {continue;}
            let destination=town(returned.after,&returned.world_after,remaining_after,second.decrement.unwrap())?;
            if !crate::winter_voyage::safe(&destination)? {continue;}
            let voyage_with_second=crate::winter_voyage::predict(&destination)?;
            let voyage_without_second=without_second.as_ref().map(crate::winter_voyage::predict).transpose()?;
            return Ok(Some(Prediction {evasive_at_entry:entry.world_after.evasive_maneuvers,
                first_encounter:first,first_escape,first_field_return:first_return,first_is_rare,
                first_trigger_updates:trigger,remaining_updates:remaining,predicted_second_encounter:predicted,
                second_encounter:second,without_second,
                voyage_without_second,voyage_with_second,
                with_second:Flee {encounter:encounter_at(after,&w)?,escape,field_return:returned,
                    remaining_updates:remaining_after,town:destination},
                verification:"Source-derived Western route; 345 running updates without padding. Extra-encounter override includes excess movement to its real trigger; town/story endpoints await retail validation."}));
        }
    }
    Err(SimError("Western Escape search exhausted representable costs".into()))
}

/// Used by upstream Eastern search: reject a rare candidate with an unsafe next opening.
pub fn after_rare(entry:&Entry,first:encounter::Prediction,v:&snows_rare::Victory)->Result<Option<Prediction>> {
    let after=v.after;let state=v.world_after.clone();
    finish(entry,first,None,None,true,after,&state)
}
/// Already killed a rare earlier: flee BOTH Western encounters that actually occur.
pub fn with_evasive(entry:&Entry)->Result<Option<Prediction>> {
    let first=encounter_at(entry.after,&entry.world_after)?;
    if !safe(&first) {return Ok(None);}
    let mut w=entry.world_after.clone();w.sync_progression()?;
    let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
    let ids:Vec<_>=first.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
    let situation=if first.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal};
    let mut search=FieldSearch::with_history(init(entry.after,&first),situation,&levels,&ids,first.encounter_level.unwrap(),w.time==4,&w.enemy_knowledge)?;
    for cost in 0..u32::MAX {
        for escape in search.unboosted_break_candidates(cost)? {
            let returned=restore(escape.attempt.as_ref().unwrap().after,&w,escape.enemy_knowledge())?;
            if !safe(&encounter_at(returned.after,&returned.world_after)?) {continue;}
            let after=returned.after;let state=returned.world_after.clone();
            if let Some(p)=finish(entry,first.clone(),Some(escape),Some(returned),false,after,&state)? {return Ok(Some(p));}
        }
    }
    Err(SimError("Western first Escape search exhausted representable costs".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entry(seed:u32,evasive:bool,counter:u32)->Entry {
        let mut w=world::throne_after_second_state().unwrap();
        // This late-route fixture has completed Throné's first chapter.
        if w.completed_scenarios.insert(700) {w.main_progress_points+=1;}
        w.set_flag(8019,true);
        for flag in [36,35] {w.set_flag(flag,true);} // completed/skipped Ch1 latent releases
        w.map=MAP.into();w.time=2;w.party=vec![7,6];w.first_encounter=true;
        w.visited_maps.insert(crate::crackridge::NEW_DELSTA_HARBOR.into());
        w.encounter_counter=counter;w.evasive_maneuvers=evasive;
        w.active_scenarios.clear();w.story_types.clear();w.prologues.clear();
        let exp=ot2_battle_engine::field_escape::experience_for_level(if evasive {15}else{5}).unwrap();
        w.party_experience=vec![exp,if evasive {exp}else{0}];w.sync_progression().unwrap();
        if evasive {w.character_progression.get_mut(&7).unwrap().jp_earned=130;}
        w.resident_levels=world::catalog().maps[MAP].streams.iter().cloned().collect();
        let mut loader=Loader::new(world::catalog(),w,CrtRng::new(seed)).unwrap();
        loader.refresh_npcs("Fixture: Western NPCs already loaded").unwrap();
        Entry {arrival:CrtRng::new(seed),after:CrtRng::new(seed),world_after:loader.state,trace:vec![],decisions:vec![]}
    }
    #[test]
    fn retail_winterbloom_load_and_night_switch_20261009() {
        let fixture:serde_json::Value=serde_json::from_str(include_str!("../data/source/live-winterbloom-night-20261009.json")).unwrap();
        let state:WorldState=serde_json::from_value(fixture["world_before"].clone()).unwrap();
        let rng=CrtRng {state:fixture["initial_state"].as_u64().unwrap() as u32,draws:19286216};
        let result=town(rng,&state,fixture["remaining"].as_u64().unwrap() as u32,7).unwrap();
        let popup=result.trace.iter().find(|t|t.reason=="Winterbloom: story prompt dismissed without starting chapter").unwrap();
        assert_eq!(popup.after.draws,19286249);
        assert_eq!(result.after.draws,19286262);
        assert_eq!(result.after.state,fixture["final_state"].as_u64().unwrap() as u32);
        let draws:Vec<_>=result.trace.iter().filter(|t|t.after.draws>t.before.draws).collect();
        let records=fixture["records"].as_array().unwrap();
        assert_eq!(draws.len(),records.len());
        for (actual,expected) in draws.iter().zip(records) {
            assert_eq!(actual.before.draws,expected["position"].as_u64().unwrap());
            assert_eq!(actual.before.state,expected["before"].as_u64().unwrap() as u32);
            if let Some(npc)=expected["npc"].as_str() {assert_eq!(actual.subject,npc);}
        }
    }
    #[test]
    fn fixed_winterbloom_night_then_harbor_day_preserve_counters() {
        let e=entry(17,true,1798);
        let town=town(e.after,&e.world_after,1,7).unwrap();
        assert_eq!(town.world_after.time,4);
        assert!(town.world_after.active_scenarios.is_empty());
        let mut before=Loader::new(world::catalog(),e.world_after.clone(),e.after).unwrap();
        before.state.encounter_counter-=7;
        before.skipped_event("SYS_SCH_GUILD_0010").unwrap();
        before.skipped_event("SYS_SCH_GUILD_0030").unwrap();
        before.state.job_licenses.entry(6).or_insert(1);
        before.state.set_flag(100,true);
        before.dismiss_tutorial(59).unwrap();
        before.visit_map(TOWN).unwrap();
        before.skipped_event("MapInStorySelect").unwrap();
        assert_eq!(town.world_after.encounter_counter,before.state.encounter_counter);
        assert!(town.after.draws>before.rng.draws);
        let actual=crate::winter_voyage::arrival(&town).unwrap();
        let mut harbor=Loader::new(world::catalog(),town.world_after.clone(),town.after).unwrap();
        harbor.visit_map(crate::crackridge::NEW_DELSTA_HARBOR).unwrap();
        assert_eq!(harbor.state.time,4);
        let counter=harbor.state.encounter_counter;
        harbor.manual_time_switch(2,"Expected Harbor day switch").unwrap();
        assert_eq!(actual.harbor.after,harbor.rng);
        assert_eq!(actual.harbor.world_after.encounter_counter,counter);
        assert_eq!(actual.harbor.world_after.time,2);
        assert_eq!(actual.crackridge.world_after.time,2);
    }
    #[test]
    fn escape_uses_carried_levels_and_both_town_branches_are_immutable() {
        let mut covered=[false;2];
        for seed in 0..256 {
            let e=entry(seed,true,899);
            let Some(p)=with_evasive(&e).unwrap() else {continue};
            covered[usize::from(p.predicted_second_encounter)]=true;
            assert_eq!(p.first_encounter.decrement,Some(7));
            assert_eq!(p.first_trigger_updates,129);
            assert_eq!(p.remaining_updates,216);
            assert!(safe(&p.first_encounter)&&safe(&p.second_encounter));
            assert!(p.first_escape.as_ref().unwrap().attempt.as_ref().unwrap().success);
            assert!(p.with_second.escape.attempt.as_ref().unwrap().success);
            assert_eq!(p.with_second.town.world_after.map,TOWN);
            assert!(p.with_second.town.world_after.active_scenarios.is_empty());
            assert_eq!(p.with_second.town.world_after.character_progression[&6].experience,e.world_after.character_progression[&6].experience);
            for onward in p.voyage_without_second.iter().chain(std::iter::once(&p.voyage_with_second)) {
                assert!(safe(&onward.arrival.encounter));
                assert!(onward.escape.attempt.as_ref().unwrap().success);
                assert_eq!(onward.escape.menu.party_levels,[15,15]);
                assert_eq!(onward.destination.world_after.map,crate::winter_voyage::DESTINATION);
                assert!(onward.cropdale_reach.initial_counter>onward.cropdale_reach.walking_cost);
                assert!(onward.cropdale_reach.estimated_counter_at_exit>0);
                let crop=&onward.cropdale;
                assert_eq!(crop.entry.world_after.map,"Twn_Fst_1_1_A");
                assert_eq!(crop.entry.arrival,onward.destination.after);
                let flags:Vec<_>=crop.entry.trace.iter().filter(|t|t.reason=="Flag initial animation speed").collect();
                assert_eq!(flags.len(),4);
                for t in flags {assert_eq!(t.after.draws-t.before.draws,1);}
                assert_eq!(crop.recruitment.arrival,crop.time_switches.first().map(|s|s.after).unwrap_or(crop.entry.after));
                assert_eq!(crop.recruitment.world_after.party,[7,6,8]);
                assert_eq!(crop.recruitment.world_after.character_progression[&8].level,1);
                assert_eq!(crop.recruitment.world_after.character_progression[&8].experience,0);
                assert_eq!(crop.recruitment.world_after.character_progression[&7].experience,e.world_after.character_progression[&7].experience);
                assert!(!crop.recruitment.world_after.flag(2901));
                assert!(crop.recruitment.world_after.flag(6525));
                assert!(crop.recruitment.after.draws>crop.entry.after.draws);
                assert!(crop.recruitment.trace.iter().any(|t|t.reason.contains("FinishPre")));
                let exit=&onward.cropdale_exit;
                assert!(safe(&exit.encounter));
                assert_eq!(exit.trail.arrival,crop.departure().after);
                assert_eq!(exit.escape.menu.party_levels,[15,15,1]);
                assert!(exit.escape.attempt.as_ref().unwrap().success);
                assert_eq!(exit.southern.world_after.map,crate::winter_voyage::SOUTHERN_CROPDALE);
                assert_eq!(exit.southern.arrival,exit.field_return.after);
                assert_eq!(exit.southern.world_after.party,[7,6,8]);
                assert_eq!(exit.southern.world_after.character_progression[&8].experience,0);
                assert!(exit.southern.world_after.first_encounter);
                if let Some(b)=exit.escape.break_action.as_ref() {
                    assert_eq!(b.attempt.after,exit.escape.attempt.as_ref().unwrap().after);
                    assert!(b.attempt.success);
                } else {
                let replay=exit.escape.menu.clone().attempt(&exit.escape.inputs.as_ref().unwrap().levels,exit.encounter.encounter_level.unwrap()).unwrap();
                assert_eq!(replay.after,exit.escape.attempt.as_ref().unwrap().after);
                }
                assert_eq!(onward.field_return.world_after.character_progression[&6].experience,e.world_after.character_progression[&6].experience);
                if let Some(b)=onward.escape.break_action.as_ref() {
                    assert_eq!(b.attempt.after,onward.escape.attempt.as_ref().unwrap().after);
                    assert!(b.attempt.success);
                } else {
                let replay=onward.escape.menu.clone().attempt(&onward.escape.inputs.as_ref().unwrap().levels,onward.arrival.encounter.encounter_level.unwrap()).unwrap();
                assert_eq!(replay.after,onward.escape.attempt.as_ref().unwrap().after);
                }
            }
            assert_eq!(e.world_after.encounter_counter,899);
            assert_eq!(p.without_second.is_none(),p.predicted_second_encounter);
            if covered==[true;2] {break;}
        }
        assert_eq!(covered,[true;2]);
    }
    #[test]
    fn western_rare_equips_evasive_only_after_first_encounter_and_forces_second() {
        for seed in 0..1024 {
            let e=entry(seed,false,700);
            let p=encounter_at(e.after,&e.world_after).unwrap();
            let Some(v)=snows_rare::kill(init(e.after,&p),&e.world_after,5,&p).unwrap() else {continue};
            let Some(route)=after_rare(&e,p,&v).unwrap() else {continue};
            assert!(!route.evasive_at_entry);
            assert!(route.first_is_rare && route.predicted_second_encounter);
            assert_eq!(route.first_encounter.decrement,Some(15));
            assert_eq!(route.second_encounter.decrement,Some(7));
            assert_eq!(route.with_second.town.world_after.character_progression[&6].level,v.party_levels_after[1]);
            return;
        }
        panic!("No Western rare fixture");
    }
    #[test]
    fn guild_and_declined_story_do_not_reset_counter_or_draw_rng() {
        let e=entry(1,true,700);
        let mut l=Loader::new(world::catalog(),e.world_after,e.after).unwrap();
        for event in ["SYS_SCH_GUILD_0010","SYS_SCH_GUILD_0030","MapInStorySelect"] {
            assert!(!world::catalog().events[event].reset_counter);
            l.skipped_event(event).unwrap();
        }
        l.dismiss_tutorial(59).unwrap();
        assert_eq!(l.rng,e.after);assert_eq!(l.state.encounter_counter,700);
    }
}
