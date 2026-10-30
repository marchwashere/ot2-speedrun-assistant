//! Southern Cropdale -> Southern Oresrush -> Mother chapter opening -> Partitio.
use crate::{Result,ensure,encounter::{self,PredictionOptions},snows_escape::Entry,
    winter_voyage::{CropdaleReach,SOUTHERN_CROPDALE},world::{self,Loader}};
use serde::Serialize;

pub const SOUTHERN:&str="Fld_Wld_1_3";
pub const TOWN:&str="Twn_Wld_1_1_C";
pub const RUN_UPDATES:u32=75;
pub const RECRUIT_EVENT:&str="MS_SHO_00_0100";
// Measured 127 walking /119 running, plus three updates of movement leeway.
pub const RYU_WALK_UPDATES:u32=130;
pub const RYU_RUN_UPDATES:u32=122;

/// Use the fresh return counter, after recruiting Partitio and loading the Wilds.
pub fn ryu_reach(state:&world::WorldState)->Result<Option<CropdaleReach>> {
    ensure(state.map==SOUTHERN,"Northern Ryu approach must start on Southern Oresrush Wilds")?;
    crate::winter_voyage::measured_reach(state,"ENT_FLD_WLD_1_3_LND",RYU_WALK_UPDATES,RYU_RUN_UPDATES)
}

#[derive(Debug,Clone,Serialize)]
pub struct RunningReach {
    pub initial_counter:u32,
    pub running_updates:u32,
    pub running_cost:u32,
    pub run_percent:f64,
    pub estimated_counter_at_exit:u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn free_switch_budgets_are_route_specific_and_pruning_is_a_lower_bound() {
        assert_eq!(free_switches_before_partitio(false),3);
        assert_eq!(free_switches_before_partitio(true),2);
        assert_eq!(switch_cost(false,3,1),0);
        assert_eq!(switch_cost(true,3,1),1);
        assert_eq!(switch_cost(false,4,2),1);
        assert_eq!(switch_cost(true,4,2),2);
        for galdera in [false,true] {
            assert_eq!(switch_cost(galdera,2,2),0);
            assert_eq!(switch_cost(galdera,2,3),1);
            for total in 0u32..20 {
                let lower=total.saturating_sub(free_switches_before_partitio(galdera)+FREE_SWITCHES_AFTER_PARTITIO);
                for before in 0..=total {
                    assert!(u64::from(lower)<=switch_cost(galdera,before,total-before));
                }
            }
        }
    }
    #[test]
    fn ryu_limits_use_return_counter_and_padded_measured_densities() {
        let mut state=world::throne_after_second_state().unwrap();
        state.map=SOUTHERN.into();state.time=2;state.evasive_maneuvers=true;
        for (counter,percent,left) in [(521,0.0,1),(627,50.0,1),(732,99.5,1),(733,100.0,1),(758,100.0,26)] {
            state.encounter_counter=counter;
            let before=crate::state_key::WorldKey::new(&state);
            let r=ryu_reach(&state).unwrap().unwrap();
            assert_eq!((r.walking_updates,r.running_updates,r.walking_cost,r.running_cost),(130,122,520,732));
            assert!((r.run_percent-percent).abs()<0.000001);
            assert_eq!(r.estimated_counter_at_exit,left);
            assert_eq!(crate::state_key::WorldKey::new(&state),before);
        }
        state.encounter_counter=520;
        assert!(ryu_reach(&state).unwrap().is_none());
        state.encounter_counter=758;state.evasive_maneuvers=false;
        assert!(ryu_reach(&state).unwrap().is_none());
        state.evasive_maneuvers=true;state.time=4;
        assert!(ryu_reach(&state).unwrap().is_none());
        state.map=TOWN.into();
        assert!(ryu_reach(&state).is_err());
    }
    fn start()->Entry {
        let mut w=world::throne_after_second_state().unwrap();
        w.map=SOUTHERN_CROPDALE.into();w.time=2;w.party=vec![7,6,8];
        w.party_experience=vec![60,0,0];w.evasive_maneuvers=true;
        w.story_types.clear();w.active_scenarios.clear();w.prologues.clear();
        w.set_flag(8019,true);w.set_flag(6525,true);w.set_flag(7062,true);
        w.main_progress_points=3;w.completed_scenarios.extend([700,600,601,800]);
        w.resident_levels=world::catalog().maps[SOUTHERN_CROPDALE].streams.iter()
            .chain(w.shared_levels.iter()).cloned().collect();
        w.encounter_counter=668;w.sync_progression().unwrap();
        Entry {arrival:crate::CrtRng::new(7),after:crate::CrtRng::new(7),world_after:w,trace:vec![],decisions:vec![]}
    }
    #[test]
    fn galdera_night_departures_switch_to_day_after_the_wilds_load() {
        let mut entry=start();entry.world_after.galdera_route=true;
        let reach=crate::winter_voyage::oresrush_reach(&entry.world_after).unwrap().unwrap();
        let mut cache=PrefixCache::new(&entry,&reach).unwrap();
        for (before,after) in [(1,0),(0,1),(2,1),(0,3)] {
            let p=cache.predict(before,after).unwrap();
            assert_eq!(p.southern_return.world_after.time,4);
            let day=p.southern_day_switch.as_ref().unwrap();
            assert_eq!(day.world_after.time,2);assert_eq!(day.arrival,p.southern_return.after);
            assert_eq!(day.world_after.encounter_counter,p.southern_return.world_after.encounter_counter);
            let mut replay=Loader::new(world::catalog(),p.southern_return.world_after.clone(),p.southern_return.after).unwrap();
            replay.manual_time_switch(2,"test required field day switch").unwrap();
            assert_eq!(replay.rng,day.after);
            if let Some(ryu)=&p.ryu {assert_eq!(ryu.northern_entry.arrival,day.after);}
            assert_eq!(serde_json::to_value(&p).unwrap(),serde_json::to_value(predict_with_switches(&entry,&reach,before,after).unwrap()).unwrap());
        }
        assert!(cache.predict(1,1).unwrap().southern_day_switch.is_none());
    }
    #[test]
    fn galdera_declines_mother_without_advancing_rng() {
        let mut entry=start();entry.world_after.galdera_route=true;
        let reach=crate::winter_voyage::oresrush_reach(&entry.world_after).unwrap().unwrap();
        let p=predict(&entry,&reach).unwrap();
        assert_eq!(p.mother_route.after,p.town_entry.after);
        for stage in [&p.mother_route,&p.recruitment,&p.southern_return] {
            assert!(!stage.world_after.flag(8050));
            assert!(!stage.world_after.active_scenarios.contains(&701));
            assert!(!stage.world_after.confirmed_scenarios.contains(&701));
        }
    }
    #[test]
    fn switches_bracket_partitio_after_mother_start_and_finish_daytime() {
        let entry=start();let reach=crate::winter_voyage::oresrush_reach(&entry.world_after).unwrap().unwrap();
        let mut cache=PrefixCache::new(&entry,&reach).unwrap();
        for (before,after) in [(0,0),(2,2),(1,1),(3,1),(2,4)] {
            let p=predict_with_switches(&entry,&reach,before,after).unwrap();
            let cached=cache.predict(before,after).unwrap();
            assert_eq!(serde_json::to_value(&cached).unwrap(),serde_json::to_value(&p).unwrap());
            let mut rng=p.mother_route.after;
            for e in &p.switches_before_partitio {
                assert_eq!(e.arrival,rng);assert!(e.world_after.active_scenarios.contains(&701));
                assert_eq!(e.world_after.party,[7,6,8]);rng=e.after;
            }
            assert_eq!(p.recruitment.arrival,rng);rng=p.recruitment.after;
            for e in &p.switches_after_partitio {assert_eq!(e.arrival,rng);assert_eq!(e.world_after.party,[7,6,8,4]);rng=e.after;}
            assert_eq!(p.southern_return.arrival,rng);assert_eq!(p.southern_return.world_after.time,2);
        }
        assert_eq!(switch_cost(false,2,2),0);assert_eq!(switch_cost(false,0,4)*12,24);
        assert_eq!(switch_cost(false,3,1)*12,0);assert_eq!(switch_cost(false,4,2)*12,12);assert_eq!(switch_cost(false,2,4)*12,24);
        assert!(predict_with_switches(&entry,&reach,1,2).is_err());
    }
    #[test]
    fn loads_story_acceptance_recruitment_and_return_keep_real_world_state() {
        let entry=start();
        let before=crate::state_key::WorldKey::new(&entry.world_after);
        let reach=crate::winter_voyage::oresrush_reach(&entry.world_after).unwrap().unwrap();
        let p=predict(&entry,&reach).unwrap();
        assert_eq!(crate::state_key::WorldKey::new(&entry.world_after),before);
        assert_eq!((p.town_reach.running_updates,p.town_reach.running_cost,p.town_reach.run_percent),(75,450,100.0));
        assert!(p.town_reach.estimated_counter_at_exit>=50);
        assert_eq!(p.southern_entry.world_after.map,SOUTHERN);
        assert_eq!(p.town_entry.world_after.map,TOWN);
        let accepted=&p.mother_route.world_after;
        assert!(accepted.flag(8050)&&accepted.flag(30510));
        assert!(accepted.confirmed_scenarios.contains(&701)&&accepted.active_scenarios.contains(&701));
        assert_eq!(accepted.main_progress_points,3);
        let joined=&p.recruitment.world_after;
        assert_eq!(joined.party,vec![7,6,8,4]);
        assert_eq!(joined.party_experience,vec![60,0,0,0]);
        assert!(joined.completed_scenarios.contains(&400));
        assert_eq!(joined.main_progress_points,4);
        for flag in [6000,6032,25069,25070,34014,6019,25561] {assert!(joined.flag(flag),"Skipped Partitio chapter flag {flag}");}
        assert_eq!(joined.character_progression[&4].level,1);
        assert_eq!(joined.character_progression[&4].jp_earned,0);
        assert_eq!(p.southern_return.world_after.map,SOUTHERN);
        assert!(p.southern_return.world_after.first_encounter);
        assert!((500..=899).contains(&p.southern_return.world_after.encounter_counter));
        assert!(p.southern_return.world_after.active_scenarios.contains(&701));
        assert_eq!(serde_json::to_value(&p.ryu_reach).unwrap(),serde_json::to_value(ryu_reach(&p.southern_return.world_after).unwrap()).unwrap());
        let mut rng=entry.after;
        for stage in [&p.southern_entry,&p.town_entry,&p.mother_route,&p.recruitment,&p.southern_return] {
            assert_eq!(stage.arrival,rng);
            for span in &stage.trace {assert_eq!(span.before,rng);rng=span.after;}
            assert_eq!(stage.after,rng);
        }
    }
}
#[derive(Debug,Serialize)]
pub struct Prediction {
    #[serde(skip_serializing_if="Option::is_none")]
    pub southern_day_switch:Option<Entry>,
    pub southern_entry:Entry,
    pub town_reach:RunningReach,
    pub town_entry:Entry,
    pub mother_route:Entry,
    pub switches_before_partitio:Vec<Entry>,
    pub switches_after_partitio:Vec<Entry>,
    pub recruitment:Entry,
    pub southern_return:Entry,
    pub ryu_reach:Option<CropdaleReach>,
    pub ryu:Option<Box<crate::ryu::Prediction>>,
    pub verification:&'static str,
}
fn checkpoint(l:&mut Loader<'_>,arrival:crate::CrtRng,reason:&str)->Entry {
    l.boundary(reason);
    Entry {arrival,after:l.rng,world_after:l.state.clone(),
        trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
pub fn predict(entry:&Entry,reach:&CropdaleReach)->Result<Prediction> {
    predict_with_switches(entry,reach,0,0)
}
pub const fn free_switches_before_partitio(galdera_route:bool)->u32 {if galdera_route {2}else{3}}
pub const FREE_SWITCHES_AFTER_PARTITIO:u32=2;
pub fn switch_cost(galdera_route:bool,before:u32,after:u32)->u64 {u64::from(before.saturating_sub(free_switches_before_partitio(galdera_route)))+u64::from(after.saturating_sub(FREE_SWITCHES_AFTER_PARTITIO))}
fn switches(l:&mut Loader<'_>,count:u32,label:&str)->Result<Vec<Entry>> {
    let mut out=Vec::new();
    for _ in 0..count {let arrival=l.rng;l.manual_time_switch(if l.state.time==2 {4}else{2},label)?;out.push(checkpoint(l,arrival,label));}
    Ok(out)
}
pub fn predict_with_switches(entry:&Entry,reach:&CropdaleReach,before:u32,after:u32)->Result<Prediction> {
    PrefixCache::new(entry,reach)?.predict(before,after)
}
pub struct PrefixCache {
    southern_entry:Entry,town_reach:RunningReach,town_entry:Entry,mother_route:Entry,
    recruited:std::collections::BTreeMap<u32,(Vec<Entry>,Entry)>,
}
impl PrefixCache {
pub fn new(entry:&Entry,reach:&CropdaleReach)->Result<Self> {
    ensure(entry.world_after.map==SOUTHERN_CROPDALE,"Southern Oresrush approach must start on Southern Cropdale Trail")?;
    let mut state=entry.world_after.clone();
    state.encounter_counter=reach.estimated_counter_at_exit;
    state.sync_progression()?;
    ensure(state.party==[7,6,8] && state.time==2,"Oresrush extension requires the daytime Throné/Osvald/Agnea party")?;
    let mut l=Loader::new(world::catalog(),state,entry.after)?;
    l.visit_map(SOUTHERN)?;
    let southern_entry=checkpoint(&mut l,entry.after,"Southern Oresrush Wilds: field control after loading");
    let decrement=encounter::catalog().counter_decrement(&PredictionOptions {
        zone:"ENT_FLD_WLD_1_3_LND".into(),period:"DayTime".into(),dash:true,
        ability_percent:if l.state.evasive_maneuvers {50}else{100},..Default::default()
    })?;
    let running_cost=RUN_UPDATES*decrement;
    ensure(l.state.encounter_counter>running_cost,"Southern Oresrush -> Oresrush would generate an encounter")?;
    let town_reach=RunningReach {initial_counter:l.state.encounter_counter,
        running_updates:RUN_UPDATES,running_cost,run_percent:100.0,
        estimated_counter_at_exit:l.state.encounter_counter-running_cost};
    l.state.encounter_counter=town_reach.estimated_counter_at_exit;
    l.visit_map(TOWN)?;
    let town_entry=checkpoint(&mut l,southern_entry.after,"Oresrush: loaded before arrival story selection");
    let mother_route=if l.state.galdera_route {
        l.skipped_event_with_choices("MapInStorySelect",&world::EventChoices::new())?;
        checkpoint(&mut l,town_entry.after,"Throné Chapter 2 Mother's route left inactive")
    } else {
        l.skipped_event_with_choices("MapInStorySelect",&world::EventChoices::from([
            (("MapInStorySelect".into(),1),"MS_TOU_01".into()),
        ]))?;
        ensure(l.state.active_scenarios.contains(&701) && l.state.flag(8050),"Mother's chapter opening did not finish")?;
        checkpoint(&mut l,town_entry.after,"Throné Chapter 2 Mother's route accepted; opening cutscene skipped")
    };
    Ok(Self {southern_entry,town_reach,town_entry,mother_route,recruited:Default::default()})
}
pub fn predict(&mut self,before:u32,after:u32)->Result<Prediction> {
    ensure(self.mother_route.world_after.galdera_route || before%2==after%2,"Oresrush time switches must finish by day")?;
    if !self.recruited.contains_key(&before) {
        let mut l=Loader::new(world::catalog(),self.mother_route.world_after.clone(),self.mother_route.after)?;
    let switches_before_partitio=switches(&mut l,before,"Oresrush: time switch before Partitio")?;
    let recruitment_arrival=l.rng;
    l.skipped_event_with_choices(RECRUIT_EVENT,&world::EventChoices::from([
        ((RECRUIT_EVENT.into(),58),"eDIALOG_YES".into()),
        ((RECRUIT_EVENT.into(),72),"eDIALOG_NO".into()),
    ]))?;
    ensure(l.state.party==[7,6,8,4] && l.state.completed_scenarios.contains(&400),"Partitio did not join with Chapter 1 skipped")?;
    l.state.party_experience.push(0);
    l.state.sync_progression()?;
    let recruitment=checkpoint(&mut l,recruitment_arrival,"Partitio recruited: Chapter 1 skipped; tutorials dismissed");
        self.recruited.insert(before,(switches_before_partitio,recruitment));
    }
    let (switches_before_partitio,recruitment)=self.recruited[&before].clone();
    let mut l=Loader::new(world::catalog(),recruitment.world_after.clone(),recruitment.after)?;
    let switches_after_partitio=switches(&mut l,after,"Oresrush: time switch after Partitio")?;
    let southern_entry=self.southern_entry.clone();let town_reach=self.town_reach.clone();
    let town_entry=self.town_entry.clone();let mother_route=self.mother_route.clone();
    let departure=l.rng;
    l.visit_map(SOUTHERN)?;
    let southern_return=checkpoint(&mut l,departure,"Southern Oresrush Wilds: returned from Oresrush with Partitio");
    let southern_day_switch=if l.state.galdera_route && l.state.time==4 {
        l.manual_time_switch(2,"Southern Oresrush Wilds: switch to day immediately after loading")?;
        Some(checkpoint(&mut l,southern_return.after,"Southern Oresrush Wilds: daytime restored before movement"))
    } else {None};
    let field=southern_day_switch.as_ref().unwrap_or(&southern_return);
    let ryu_reach=ryu_reach(&field.world_after)?;
    let ryu=ryu_reach.as_ref().map(|reach|crate::ryu::predict(field,reach).map(Box::new)).transpose()?;
    let galdera_route=field.world_after.galdera_route;
    Ok(Prediction {southern_day_switch,southern_entry,town_reach,town_entry,mother_route,switches_before_partitio,switches_after_partitio,recruitment,southern_return,ryu_reach,ryu,
        verification:if galdera_route {
            "Source-derived shared map loading and Partitio skip-Chapter-1 recruitment; Mother Chapter 2 remains inactive. Southern Wilds -> Oresrush uses the measured 75 running updates. Declining the arrival story selection consumes no RNG."
        } else {
            "Source-derived shared map loading, Mother Chapter 2 acceptance and Partitio skip-Chapter-1 recruitment. Southern Wilds -> Oresrush uses the measured 75 running updates; no walking measurement assumed. Mother arrival acceptance matches the seven-draw retail capture of 2026-10-09; remaining route callbacks are source-derived."
        }})
}

}

#[cfg(test)]
mod retail_tests {
    use super::*;
    #[test]
    fn retail_mother_chapter_acceptance_20261009() {
        let fixture:serde_json::Value=serde_json::from_str(include_str!("../data/source/live-oresrush-mother-20261009.json")).unwrap();
        let state=serde_json::from_value(fixture["world_before"].clone()).unwrap();
        let rng=crate::CrtRng {state:fixture["initial_state"].as_u64().unwrap() as u32,draws:19286550};
        let mut loader=Loader::new(world::catalog(),state,rng).unwrap();
        let mut declined=Loader::new(world::catalog(),loader.state.clone(),rng).unwrap();
        declined.skipped_event_with_choices("MapInStorySelect",&world::EventChoices::new()).unwrap();
        assert_eq!(declined.rng,rng,"Declining must not inherit chapter-acceptance resets");
        loader.skipped_event_with_choices("MapInStorySelect",&world::EventChoices::from([
            (("MapInStorySelect".into(),1),"MS_TOU_01".into()),
        ])).unwrap();
        assert_eq!(loader.rng.draws,19286557);
        assert_eq!(loader.rng.state,fixture["final_state"].as_u64().unwrap() as u32);
        assert!(loader.state.active_scenarios.contains(&701) && loader.state.flag(8050));
        let draws:Vec<_>=loader.trace.iter().filter(|t|t.after.draws>t.before.draws).collect();
        let records=fixture["records"].as_array().unwrap();
        assert_eq!(draws.len(),records.len());
        for (actual,expected) in draws.iter().zip(records) {
            assert_eq!(actual.before.draws,expected["position"].as_u64().unwrap());
            assert_eq!(actual.before.state,expected["before"].as_u64().unwrap() as u32);
            if let Some(npc)=expected["npc"].as_str() {assert_eq!(actual.subject,npc);}
        }
    }
}
