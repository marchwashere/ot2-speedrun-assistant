//! Winterbloom tag -> New Delsta Harbor -> Crackridge -> Eastern Cropdale Trail.
use crate::{CrtRng,Result,ensure,crackridge,encounter::{self,Boundary,PredictionOptions},
    highroad::{self,Escape,FieldReturn},escape_search::FieldSearch,
    snows_escape::Entry,world::{self,Loader}};
use ot2_battle_engine::escape::Situation;
use serde::Serialize;

pub const RUN_UPDATES:u32=172;
pub const WALK_UPDATES:u32=182;
pub const DESTINATION:&str="Fld_Fst_1_3";
pub const CROPDALE_WALK_UPDATES:u32=147;
pub const CROPDALE_RUN_UPDATES:u32=137;
// Measured 119 walking /109 running, plus three updates of movement leeway.
pub const ORESRUSH_WALK_UPDATES:u32=122;
pub const ORESRUSH_RUN_UPDATES:u32=112;
#[derive(Debug,serde::Deserialize,Serialize)]
pub struct CropdaleReach {
    pub initial_counter:u32,
    pub walking_updates:u32,
    pub running_updates:u32,
    pub walking_cost:u32,
    pub running_cost:u32,
    pub run_percent:f64,
    pub estimated_counter_at_exit:u32,
}
pub fn cropdale_reach(state:&crate::world::WorldState)->Result<Option<CropdaleReach>> {
    measured_reach(state,"ENT_FLD_FST_1_3_LND",CROPDALE_WALK_UPDATES,CROPDALE_RUN_UPDATES)
}
/// Measured Southern Cropdale Trail entry -> Southern Oresrush Wilds leg.
/// Mixed distance is interpolation, with one point reserved and share rounded
/// down to 0.1%; it is not a frame-by-frame simulation of speed-switch overshoot.
pub fn oresrush_reach(state:&crate::world::WorldState)->Result<Option<CropdaleReach>> {
    ensure(state.map==SOUTHERN_CROPDALE,"Oresrush approach must start on Southern Cropdale Trail")?;
    measured_reach(state,"ENT_FLD_FST_1_4_LND",ORESRUSH_WALK_UPDATES,ORESRUSH_RUN_UPDATES)
}
pub(crate) fn measured_reach(state:&crate::world::WorldState,zone:&str,walking_updates:u32,running_updates:u32)->Result<Option<CropdaleReach>> {
    let mut options=PredictionOptions {
        zone:zone.into(),
        period:if state.time==2 {"DayTime"}else{"Night"}.into(),
        ability_percent:if state.evasive_maneuvers {50}else{100},dash:false,..Default::default()
    };
    let walking_cost=walking_updates*encounter::catalog().counter_decrement(&options)?;
    options.dash=true;
    let running_cost=running_updates*encounter::catalog().counter_decrement(&options)?;
    let counter=state.encounter_counter;
    if counter<=walking_cost {return Ok(None);}
    let fraction=if counter>running_cost {1.0}else{
        ((counter-1-walking_cost) as f64/(running_cost-walking_cost) as f64*1000.0).floor()/1000.0
    };
    let consumed=(walking_cost as f64*(1.0-fraction)+running_cost as f64*fraction).ceil() as u32;
    Ok(Some(CropdaleReach {initial_counter:counter,walking_updates,
        running_updates,walking_cost,running_cost,run_percent:fraction*100.0,
        estimated_counter_at_exit:counter-consumed}))
}
#[derive(Debug,Serialize)]
pub struct Arrival {pub harbor:Entry,pub crackridge:Entry,pub encounter:encounter::Prediction,pub battle_entry:CrtRng}
#[derive(Debug,Serialize)]
pub struct Prediction {pub arrival:Arrival,pub escape:Escape,pub field_return:FieldReturn,
    pub remaining_run_updates:u32,pub remaining_walk_updates:u32,pub run_percent:f64,pub destination:Entry,
    pub cropdale_reach:CropdaleReach,pub cropdale:Box<Cropdale>,pub cropdale_exit:Box<CropdaleExit>}


/// Town loading and the first invitation, accepting Agnea but skipping Chapter 1.
#[derive(Debug,Serialize)]
pub struct Cropdale {
    pub entry:Entry,pub recruitment:Entry,
    /// Actual manual-switch callbacks: first before recruitment, all others after.
    #[serde(skip_serializing_if="Vec::is_empty")]
    pub time_switches:Vec<Entry>,
    pub verification:&'static str,
}
impl Cropdale {
    pub fn departure(&self)->&Entry {self.time_switches.last().unwrap_or(&self.recruitment)}
    pub fn time_switch_input_cost(&self)->u32 {
        (self.time_switches.len() as u32).saturating_sub(2)*7
    }
}
pub fn enter_cropdale(destination:&Entry,reach:&CropdaleReach)->Result<Cropdale> {
    enter_cropdale_with_switches(destination,reach,0)
}
pub fn enter_cropdale_with_switches(destination:&Entry,reach:&CropdaleReach,switches:u32)->Result<Cropdale> {
    ensure(switches%2==0,"Cropdale time switches must end in daytime")?;
    ensure(destination.world_after.map==DESTINATION,"Cropdale approach must start on Eastern Cropdale Trail")?;
    let mut state=destination.world_after.clone();
    state.encounter_counter=reach.estimated_counter_at_exit;
    let mut l=Loader::new(world::catalog(),state,destination.after)?;
    l.visit_map("Twn_Fst_1_1_A")?;
    l.boundary("Cropdale: field control after loading");
    let entry=Entry {arrival:destination.after,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)};
    ensure(l.state.party==[7,6],"Agnea invitation requires the carried Throné/Osvald party")?;
    ensure(l.state.time==2,"Cropdale switch search must start in daytime")?;
    let mut time_switches=Vec::new();
    if switches>0 {
        l.manual_time_switch(4,"Cropdale: switch to night before recruiting Agnea")?;
        time_switches.push(Entry {arrival:entry.after,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)});
    }
    let recruitment_start=l.rng;
    let event="MS_ODO_00_0200";
    l.skipped_event_with_choices(event,&world::EventChoices::from([
        ((event.into(),125),"eDIALOG_YES".into()),
        ((event.into(),134),"eDIALOG_NO".into()),
    ]))?;
    ensure(l.state.party==[7,6,8] && l.state.flag(6525) && !l.state.flag(2901),"Agnea did not join with Chapter 1 skipped")?;
    l.state.party_experience.push(0);
    l.state.sync_progression()?;
    l.boundary("Agnea recruited: Chapter 1 skipped; tutorials dismissed");
    let recruitment=Entry {arrival:recruitment_start,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)};
    for _ in 1..switches {
        let arrival=l.rng;
        l.manual_time_switch(if l.state.time==2 {4}else{2},"Cropdale: time switch after recruiting Agnea")?;
        time_switches.push(Entry {arrival,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)});
    }
    ensure(l.state.time==2,"Cropdale departure must be daytime")?;
    Ok(Cropdale {entry,recruitment,time_switches,verification:"Source-derived town load, optional manual time switches and Agnea skip-Chapter-1 event; awaiting retail validation"})
}

pub const SOUTHERN_CROPDALE:&str="Fld_Fst_1_4";
#[derive(Debug,Serialize)]
pub struct CropdaleExit {
    pub trail:Entry,pub encounter:encounter::Prediction,pub battle_entry:CrtRng,
    pub escape:Escape,pub field_return:FieldReturn,pub southern:Entry,
    pub oresrush_reach:Option<CropdaleReach>,
    pub oresrush:Option<Box<crate::oresrush::Prediction>>,
    pub verification:&'static str,
}
fn return_to_trail(cropdale:&Cropdale)->Result<(Entry,encounter::Prediction,CrtRng)> {
    let departure=cropdale.departure();
    let mut w=departure.world_after.clone();w.sync_progression()?;
    ensure(w.party==[7,6,8] && w.time==2,"Cropdale exit requires the daytime Throné/Osvald/Agnea party")?;
    let mut l=Loader::new(world::catalog(),w,departure.after)?;
    l.visit_map(DESTINATION)?;
    let trail=Entry {arrival:departure.after,after:l.rng,world_after:l.state,trace:l.trace,decisions:l.decisions};
    let w=&trail.world_after;let mut rng=trail.after;
    let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions {
        map:DESTINATION.into(),zone:"ENT_FLD_FST_1_3_LND".into(),period:"DayTime".into(),progress:w.main_progress_points,
        first_encounter:w.first_encounter,boundary:Boundary::CurrentCounter,current_counter:Some(w.encounter_counter),
        party_levels:w.party.iter().map(|id|w.character_progression[id].level).collect(),
        ability_percent:if w.evasive_maneuvers {50}else{100},dash:true,..Default::default()
    })?;
    Ok((trail,encounter,rng))
}
fn escape_to_southern(trail:Entry,encounter:encounter::Prediction,battle_entry:CrtRng)->Result<CropdaleExit> {
    escape_to_southern_range(trail,encounter,battle_entry,0,u32::MAX-1,true)?.ok_or_else(||crate::SimError("Eastern Escape exhausted".into()))
}
fn escape_to_southern_range(trail:Entry,encounter:encounter::Prediction,battle_entry:CrtRng,min:u32,max:u32,finish:bool)->Result<Option<CropdaleExit>> {
    ensure(matches!(encounter.situation.as_deref(),Some("Normal"|"Preemptive")),"Eastern Cropdale return is surprised; adjust Crackridge escape")?;
    let w=&trail.world_after;
    let ids:Vec<_>=encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
    let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
    let situation=if encounter.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal};
    let mut search=FieldSearch::with_history(battle_entry,situation,&levels,&ids,encounter.encounter_level.unwrap(),false,&w.enemy_knowledge)?;
    for cost in min..=max {
        for escape in search.candidates(cost,false,None)? {
            let field_return=highroad::restore_field(w,escape.attempt.as_ref().unwrap(),DESTINATION,"Eastern Cropdale return Escape")?;
            // User-confirmed exactly one encounter on this leg. Distance and
            // residual counter are deliberately not inferred. The outgoing and
            // incoming transition callbacks roll real fresh counters regardless.
            let mut l=Loader::new(world::catalog(),field_return.world_after.clone(),field_return.after)?;
            l.boundary("Eastern Cropdale -> Southern Cropdale: one encounter assumed; unmeasured movement has no CRT draws");
            l.visit_map(SOUTHERN_CROPDALE)?;
            let southern=Entry {arrival:field_return.after,after:l.rng,world_after:l.state,trace:l.trace,decisions:l.decisions};
            let Some(oresrush_reach)=oresrush_reach(&southern.world_after)? else {continue};
            let mut oresrush=Box::new(crate::oresrush::predict(&southern,&oresrush_reach)?);
            // This is the last controllable menu. Replay the town, Mother opening,
            // recruitment and return load before testing its newly rolled counter;
            // movement never consumes synthetic CRT draws.
            if !oresrush.ryu.as_ref().is_some_and(|ryu|ryu.town_reach.is_some()) {continue;}
            if finish {crate::ryu::recruit(oresrush.ryu.as_mut().unwrap())?;}
            return Ok(Some(CropdaleExit {trail,encounter,battle_entry,escape,field_return,southern,
                oresrush_reach:Some(oresrush_reach),oresrush:Some(oresrush),
                verification:"Source-derived three-character first-menu Escape and shared subsequent loads/events. Search secures walking-safe Southern Cropdale -> Southern Oresrush, Southern Oresrush return -> Northern Ryu Sands and Northern Ryu Sands -> Ryu legs. User assumes exactly one encounter on the unmeasured Eastern return leg. Appends Hikari's supplied recruit battle and skip-Chapter-1 Osvald replacement after selecting the movement candidate. Awaiting retail validation"}));
        }
    }
    Ok(None)
}

/// Freeze Cropdale and its escape, then compare Oresrush switch allocations.
pub fn search_oresrush(destination:&Entry,reach:&CropdaleReach)->Result<(Box<Cropdale>,Box<CropdaleExit>)> {
    let crop=Box::new(enter_cropdale_with_switches(destination,reach,0)?);
    let (trail,encounter,rng)=return_to_trail(&crop)?;
    let mut exit=Box::new(escape_to_southern_range(trail,encounter,rng,0,u32::MAX-1,false)?.ok_or_else(||crate::SimError("Eastern Escape exhausted".into()))?);
    let (before,after,expected_cost,expected_rng)=world::without_diagnostics(|| {
        let mut best:Option<(u32,u32,u64,CrtRng,bool)>=None;
        let mut prefixes=crate::oresrush::PrefixCache::new(&exit.southern,exit.oresrush_reach.as_ref().unwrap())?;
        let mut budget=32u32;
        loop {
        for total in (0..u32::MAX-1).step_by(2) {
            let bound=best.as_ref().map_or(u64::from(budget),|b|b.2);
            if u64::from(total.saturating_sub(crate::oresrush::FREE_SWITCHES_BEFORE_PARTITIO+crate::oresrush::FREE_SWITCHES_AFTER_PARTITIO))*7>bound {break;}
            for before in 0..=total {
                let after=total-before;
                let time_cost=crate::oresrush::switch_cost(before,after)*7;
                let bound=best.as_ref().map_or(u64::from(budget),|b|b.2);
                if time_cost>bound {continue;}
                let Some(o)=finish_oresrush(prefixes.predict(before,after)?,Some((bound-time_cost) as u32))? else {continue;};
                let r=o.ryu.as_ref().unwrap();
                let cost=time_cost+u64::from(r.hikari_search.as_ref().unwrap().total_input_cost);
                let coast=r.canalbrine.as_ref().unwrap().conning.as_ref().unwrap();
                let northern=coast.northern.victory.is_some();
                if best.as_ref().is_none_or(|b|cost<b.2||(cost==b.2&&northern&&!b.4)) {
                    best=Some((before,after,cost,coast.town.after,northern));
                }
            }
        }
        if best.is_some() {break;}
        budget=budget.checked_mul(2).ok_or_else(||crate::SimError("Oresrush cost overflow".into()))?;
        }
        best.map(|b|(b.0,b.1,b.2,b.3)).ok_or_else(||crate::SimError("Oresrush switch search exhausted".into()))
    })?;
    let o=propagate_oresrush(&exit,before,after)?.expect("Selected Oresrush plan replays");
    let r=o.ryu.as_ref().unwrap();
    ensure(r.canalbrine.as_ref().unwrap().conning.as_ref().unwrap().town.after==expected_rng,"Oresrush replay changed RNG")?;
    ensure(crate::oresrush::switch_cost(before,after)*7+u64::from(r.hikari_search.as_ref().unwrap().total_input_cost)==expected_cost,"Oresrush replay changed cost")?;
    exit.oresrush=Some(Box::new(o));
    Ok((crop,exit))
}
fn propagate_oresrush(exit:&CropdaleExit,before:u32,after:u32)->Result<Option<crate::oresrush::Prediction>> {
    let o=crate::oresrush::predict_with_switches(&exit.southern,exit.oresrush_reach.as_ref().unwrap(),before,after)?;
    finish_oresrush(o,None)
}
fn finish_oresrush(mut o:crate::oresrush::Prediction,limit:Option<u32>)->Result<Option<crate::oresrush::Prediction>> {
    let Some(r)=o.ryu.as_mut() else {return Ok(None)};
    let Some(town)=r.town_entry.as_ref() else {return Ok(None)};
    let Some(selected)=crate::hikari_manipulation::nearest_coasts_bounded(town,limit)? else {return Ok(None)};
    r.hikari=Some(selected.hikari);r.throne_scholar=Some(selected.scholar);
    r.canalbrine=Some(selected.canalbrine);r.hikari_search=Some(selected.search);
    Ok(Some(o))
}
pub fn arrival(entry:&Entry)->Result<Arrival> {
    let mut w=entry.world_after.clone();w.sync_progression()?;
    ensure(w.visited_maps.contains(crackridge::NEW_DELSTA_HARBOR),"New Delsta Harbor must be tagged")?;
    let mut l=Loader::new(world::catalog(),w,entry.after)?;
    l.visit_map(crackridge::NEW_DELSTA_HARBOR)?;
    // Active route arrives at night following the fixed Winterbloom switch.
    if l.state.time!=2 {l.manual_time_switch(2,"New Delsta Harbor: switch to day after landing, before boat")?;}
    let harbor=Entry {arrival:entry.after,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)};
    l.skipped_event(crackridge::VOYAGE_EVENT)?;
    let crackridge=Entry {arrival:harbor.after,after:l.rng,world_after:l.state.clone(),trace:l.trace,decisions:l.decisions};
    let mut rng=crackridge.after;
    let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions {
        map:crackridge::CRACKRIDGE_HARBOR.into(),zone:"ENT_FLD_WLD_1_2_LND".into(),period:"DayTime".into(),
        progress:crackridge.world_after.main_progress_points,first_encounter:crackridge.world_after.first_encounter,boundary:Boundary::CurrentCounter,
        current_counter:Some(crackridge.world_after.encounter_counter),
        party_levels:crackridge.world_after.party.iter().map(|id|crackridge.world_after.character_progression[id].level).collect(),
        ability_percent:if crackridge.world_after.evasive_maneuvers {50}else{100},dash:true,..Default::default()
    })?;
    Ok(Arrival {harbor,crackridge,encounter,battle_entry:rng})
}
pub fn safe(entry:&Entry)->Result<bool> {
    let a=arrival(entry)?;
    if !matches!(a.encounter.situation.as_deref(),Some("Normal"|"Preemptive")) || a.encounter.enemies.len()!=1 {return Ok(false);}
    let w=&a.crackridge.world_after;
    let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
    let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
    let ids:Vec<_>=a.encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
    let situation=if a.encounter.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal};
    for bp in 0..=1 {
        for weapon in [ot2_battle_engine::combat::Weapon::Dagger,ot2_battle_engine::combat::Weapon::Sword] {
            let inputs=if bp==0 {vec![]}else{vec![1]};
            if ot2_battle_engine::field_escape::opening_break_menu_with_players(a.battle_entry,situation,&levels,&ids,weapon,&inputs,false,&w.enemy_knowledge,Some(&players))?.is_some() {return Ok(true);}
        }
    }
    Ok(false)
}
pub fn predict(entry:&Entry)->Result<Prediction> {
    let _profile=crate::profile::span("voyage complete");
    let arrival=arrival(entry)?;
    ensure(matches!(arrival.encounter.situation.as_deref(),Some("Normal"|"Preemptive")),"Crackridge approach is surprised; adjust the preceding Western fight")?;
    let w=&arrival.crackridge.world_after;
    let ids:Vec<_>=arrival.encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
    let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
    let situation=if arrival.encounter.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal};
    ensure(safe(entry)?,"Crackridge requires a safe Throné-breakable opening")?;
    let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
    let mut search=FieldSearch::with_players(arrival.battle_entry,situation,&players,&levels,&ids,arrival.encounter.encounter_level.unwrap(),&w.enemy_knowledge)?;
    search.require_break();
    let trigger=arrival.encounter.counter_updates.unwrap();
    ensure(trigger<=RUN_UPDATES,"Crackridge encounter lies past the measured route")?;
    let run=RUN_UPDATES-trigger;
    let walk=((run as u64*WALK_UPDATES as u64).div_ceil(RUN_UPDATES as u64)) as u32;
    let run_dec=arrival.encounter.decrement.unwrap();
    let walk_dec=encounter::catalog().counter_decrement(&PredictionOptions {
        zone:"ENT_FLD_WLD_1_2_LND".into(),period:"DayTime".into(),dash:false,
        ability_percent:if w.evasive_maneuvers {50}else{100},..Default::default()
    })?;
    for cost in 0..u32::MAX {
        for escape in search.candidates(cost,true,None)? {
            let returned=highroad::restore_field(w,escape.attempt.as_ref().unwrap(),crackridge::CRACKRIDGE_HARBOR,"Crackridge Escape")?;
            let counter=returned.world_after.encounter_counter;
            let walk_cost=walk*walk_dec;let run_cost=run*run_dec;
            if counter<=walk_cost {continue;}
            let fraction=if counter>run_cost {1.0}else{((counter-1-walk_cost) as f64/(run_cost-walk_cost) as f64*1000.0).floor()/1000.0};
            let consumed=(walk_cost as f64*(1.0-fraction)+run_cost as f64*fraction).ceil() as u32;
            let mut state=returned.world_after.clone();state.encounter_counter-=consumed;
            let mut l=Loader::new(world::catalog(),state,returned.after)?;l.visit_map(DESTINATION)?;
            let destination=Entry {arrival:returned.after,after:l.rng,world_after:l.state,trace:l.trace,decisions:l.decisions};
            let Some(cropdale_reach)=cropdale_reach(&destination.world_after)? else {continue};
            let cropdale=Box::new(enter_cropdale(&destination,&cropdale_reach)?);
            let (trail,encounter,battle_entry)=return_to_trail(&cropdale)?;
            if !matches!(encounter.situation.as_deref(),Some("Normal"|"Preemptive")) {continue;}
            let cropdale_exit=Box::new(escape_to_southern(trail,encounter,battle_entry)?);
            return Ok(Prediction {arrival,escape,field_return:returned,remaining_run_updates:run,remaining_walk_updates:walk,run_percent:fraction*100.0,destination,cropdale_reach,cropdale,cropdale_exit});
        }
    }
    Err(crate::SimError("Crackridge Escape search exhausted".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retail_southern_cropdale_load_20261009() {
        let fixture:serde_json::Value=serde_json::from_str(include_str!("../data/source/live-southern-cropdale-20261009.json")).unwrap();
        let state=serde_json::from_value(fixture["world_before"].clone()).unwrap();
        let rng=CrtRng {state:fixture["initial_state"].as_u64().unwrap() as u32,draws:19286486};
        let mut loader=Loader::new(world::catalog(),state,rng).unwrap();
        loader.visit_map(SOUTHERN_CROPDALE).unwrap();
        assert_eq!(world::catalog().maps[SOUTHERN_CROPDALE].id,132);
        assert_eq!(loader.rng.draws,19286498);
        assert_eq!(loader.rng.state,fixture["final_state"].as_u64().unwrap() as u32);
        let draws:Vec<_>=loader.trace.iter().filter(|t|t.after.draws>t.before.draws).collect();
        let records=fixture["records"].as_array().unwrap();
        assert_eq!(draws.len(),records.len());
        for (actual,expected) in draws.iter().zip(records) {
            assert_eq!(actual.before.draws,expected["position"].as_u64().unwrap());
            assert_eq!(actual.before.state,expected["before"].as_u64().unwrap() as u32);
            if let Some(npc)=expected["npc"].as_str() {assert_eq!(actual.subject,npc);}
        }
        assert!(oresrush_reach(&loader.state).is_ok());
    }
    #[test]
    fn cropdale_switches_straddle_recruitment_and_exit_by_day() {
        let mut w=world::throne_after_second_state().unwrap();
        w.map=DESTINATION.into();w.time=2;w.party=vec![7,6];
        w.party_experience=vec![1530,1467];
        w.story_types.clear();w.active_scenarios.clear();w.prologues.clear();
        w.completed_scenarios.extend([700,600,601]);
        w.set_flag(8019,true);w.set_flag(7062,true);w.set_flag(6525,false);
        w.main_progress_points=2;w.encounter_counter=1000;w.sync_progression().unwrap();
        w.evasive_maneuvers=true;
        w.resident_levels=world::catalog().maps[DESTINATION].streams.iter()
            .chain(w.shared_levels.iter()).cloned().collect();
        let rng=CrtRng::new(17);
        let entry=Entry {arrival:rng,after:rng,world_after:w,trace:vec![],decisions:vec![]};
        let reach=cropdale_reach(&entry.world_after).unwrap().unwrap();
        let unchanged=crate::state_key::WorldKey::new(&entry.world_after);
        for switches in [0,2,4,6] {
            let p=enter_cropdale_with_switches(&entry,&reach,switches).unwrap();
            assert_eq!(p.departure().world_after.time,2);
            assert_eq!(p.departure().world_after.party,[7,6,8]);
            assert_eq!(p.time_switch_input_cost(),switches.saturating_sub(2)*7);
            if switches>0 {
                assert_eq!(p.time_switches[0].world_after.party,[7,6]);
                assert_eq!(p.time_switches[0].world_after.time,4);
                assert_eq!(p.recruitment.arrival,p.time_switches[0].after);
                assert_eq!(p.recruitment.world_after.time,4);
                assert_eq!(p.time_switches[1].arrival,p.recruitment.after);
                for s in &p.time_switches[1..] {assert_eq!(s.world_after.party,[7,6,8]);}
                for pair in p.time_switches[1..].windows(2) {assert_eq!(pair[0].after,pair[1].arrival);}
            } else {assert_eq!(p.departure().after,p.recruitment.after);}
            let (trail,_,_)=return_to_trail(&p).unwrap();
            assert_eq!(trail.arrival,p.departure().after);
            assert_eq!(trail.world_after.time,2);
        }
        assert!(enter_cropdale_with_switches(&entry,&reach,1).is_err());
        assert_eq!(crate::state_key::WorldKey::new(&entry.world_after),unchanged);
    }
    #[test]
    fn successful_escape_is_retried_when_the_later_ryu_counter_is_unsafe() {
        let mut world=world::throne_after_second_state().unwrap();
        world.map=DESTINATION.into();world.time=2;world.party=vec![7,6,8];
        world.party_experience=vec![60,0,0];world.evasive_maneuvers=true;
        world.story_types.clear();world.active_scenarios.clear();world.prologues.clear();
        world.set_flag(8019,true);world.set_flag(6525,true);world.set_flag(7062,true);
        for flag in [36,35,37] {world.set_flag(flag,true);} // completed/skipped Ch1 latent releases
        world.main_progress_points=3;world.completed_scenarios.extend([700,600,601,800]);
        world.resident_levels=world::catalog().maps[DESTINATION].streams.iter()
            .chain(world.shared_levels.iter()).cloned().collect();
        world.encounter_counter=700;world.sync_progression().unwrap();
        // This fixture starts after the rare kill and guild visit; carry the
        // earlier reward/license needed by the final Scholar continuation.
        world.character_progression.get_mut(&7).unwrap().jp_earned=130;
        world.job_licenses.insert(6,1);
        let unchanged=crate::state_key::WorldKey::new(&world);
        // Find a real unsafe continuation under current source map/story callbacks.
        // A hardcoded seed stopped being unsafe after those callbacks changed.
        for seed in 0..4096 {
        let start=CrtRng::new(seed);let mut battle_entry=start;
        let encounter=encounter::catalog().predict(&mut battle_entry,&PredictionOptions {
            map:DESTINATION.into(),zone:"ENT_FLD_FST_1_3_LND".into(),period:"DayTime".into(),
            progress:3,first_encounter:true,boundary:Boundary::CurrentCounter,
            current_counter:Some(700),party_levels:world.party.iter().map(|id|world.character_progression[id].level).collect(),
            dash:true,ability_percent:50,..Default::default()
        }).unwrap();
        if !matches!(encounter.situation.as_deref(),Some("Normal"|"Preemptive")) {continue;}
        let situation=if encounter.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal};
        let ids:Vec<_>=encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
        let levels:Vec<_>=world.party.iter().map(|id|world.character_progression[id].level).collect();
        let mut search=FieldSearch::with_history(battle_entry,situation,&levels,&ids,encounter.encounter_level.unwrap(),false,&world.enemy_knowledge).unwrap();
        let Some(first)=search.candidates(0,false,None).unwrap().into_iter().next() else {continue};
        let returned=highroad::restore_field(&world,first.attempt.as_ref().unwrap(),DESTINATION,"Eastern Cropdale return Escape").unwrap();
        let mut loader=Loader::new(world::catalog(),returned.world_after.clone(),returned.after).unwrap();
        loader.visit_map(SOUTHERN_CROPDALE).unwrap();
        let southern=Entry {arrival:returned.after,after:loader.rng,world_after:loader.state,trace:loader.trace,decisions:loader.decisions};
        let Some(reach)=oresrush_reach(&southern.world_after).unwrap() else {continue};
        let rejected=crate::oresrush::predict(&southern,&reach).unwrap();
        if rejected.ryu_reach.is_some() {continue;}
        let trail=Entry {arrival:start,after:start,world_after:world.clone(),trace:vec![],decisions:vec![]};
        let selected=escape_to_southern(trail,encounter,battle_entry).unwrap();
        assert!(selected.escape.inputs.as_ref().is_some_and(|i|i.input_count>0),"The unadjusted Escape cannot meet the return-counter goal");
        let r=selected.oresrush.as_ref().unwrap().ryu_reach.as_ref().unwrap();
        assert!(r.initial_counter>r.walking_cost && r.estimated_counter_at_exit>0);
        assert_eq!(r.initial_counter,selected.oresrush.as_ref().unwrap().southern_return.world_after.encounter_counter);
        let ryu=selected.oresrush.as_ref().unwrap().ryu.as_ref().unwrap();
        assert!(ryu.town_reach.as_ref().unwrap().estimated_counter_at_exit>0);
        assert_eq!(ryu.town_entry.as_ref().unwrap().world_after.map,crate::ryu::TOWN);
        assert_eq!(crate::state_key::WorldKey::new(&world),unchanged);
        assert_eq!(selected.field_return.arrival,selected.escape.attempt.as_ref().unwrap().after);
        let fresh=search.candidates(selected.escape.weighted_input_cost,false,None).unwrap().into_iter()
            .find(|candidate|candidate.inputs==selected.escape.inputs).unwrap();
        assert_eq!(serde_json::to_value(&fresh).unwrap(),serde_json::to_value(&selected.escape).unwrap());
        return;
        }
        panic!("No successful Escape with unsafe Ryu continuation found");
    }
    #[test]
    fn oresrush_limits_use_carried_counter_and_support_without_advancing_rng() {
        let mut state=world::throne_after_second_state().unwrap();
        state.map=SOUTHERN_CROPDALE.into();state.time=2;state.evasive_maneuvers=true;
        for (counter,percent,left) in [(489,0.0,1),(500,5.9,1),(581,50.0,1),(672,99.4,1),(673,100.0,1),(668,97.2,1)] {
            state.encounter_counter=counter;
            let before=crate::state_key::WorldKey::new(&state);
            let r=oresrush_reach(&state).unwrap().unwrap();
            assert_eq!((r.walking_updates,r.running_updates,r.walking_cost,r.running_cost),(122,112,488,672));
            assert!((r.run_percent-percent).abs()<0.000001);
            assert_eq!(r.estimated_counter_at_exit,left);
            assert_eq!(crate::state_key::WorldKey::new(&state),before);
        }
        state.encounter_counter=488;
        assert!(oresrush_reach(&state).unwrap().is_none());
        state.encounter_counter=668;state.evasive_maneuvers=false;
        assert!(oresrush_reach(&state).unwrap().is_none());
    }
    #[test]
    fn retail_cropdale_return_uses_recruited_prologue_tier() {
        let mut rng = CrtRng::new(0);
        rng.advance(6817);
        let p = encounter::catalog().predict(&mut rng, &PredictionOptions {
            map:DESTINATION.into(),zone:"ENT_FLD_FST_1_3_LND".into(),period:"DayTime".into(),
            progress:3,first_encounter:true,boundary:Boundary::CurrentCounter,
            current_counter:Some(700),party_levels:vec![15,15,1],dash:true,ability_percent:50,
            ..Default::default()
        }).unwrap();
        assert_eq!(p.encounter_table.as_deref(),Some("ENC_FLD_FST_1_3_LND_DAY_020"));
        let names:Vec<_> = p.enemies.iter().map(|e|e.enemy.name.as_str()).collect();
        assert_eq!(names, vec!["Woodland Birdian I","Woodland Birdian I","Woodland Birdian II"]);
    }

    #[test]
    fn padded_cropdale_limits_use_both_measured_densities_and_strict_thresholds() {
        let mut state=world::throne_after_second_state().unwrap();
        state.time=2;state.evasive_maneuvers=true;
        state.encounter_counter=588;
        assert!(cropdale_reach(&state).unwrap().is_none());
        state.encounter_counter=589;
        let p=cropdale_reach(&state).unwrap().unwrap();
        assert_eq!((p.walking_cost,p.running_cost,p.run_percent,p.estimated_counter_at_exit),(588,822,0.0,1));
        state.encounter_counter=706;
        let p=cropdale_reach(&state).unwrap().unwrap();
        assert_eq!(p.run_percent,50.0);assert_eq!(p.estimated_counter_at_exit,1);
        state.encounter_counter=822;
        assert!(cropdale_reach(&state).unwrap().unwrap().run_percent<100.0);
        state.encounter_counter=823;
        assert_eq!(cropdale_reach(&state).unwrap().unwrap().run_percent,100.0);
        state.evasive_maneuvers=false;
        assert!(cropdale_reach(&state).unwrap().is_none());
    }
}
