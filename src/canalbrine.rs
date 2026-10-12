//! Ryu -> Northern Ryu Sands -> Canalbrine Bridge, without an encounter.
//! Movement is supplied by the route assumption; every map load uses Loader.
use crate::{Result,ensure,encounter::{self,Boundary,PredictionOptions},snows_escape::Entry,world::{self,Loader}};
use serde::Serialize;

pub const BRIDGE:&str="Fld_Sea_1_2";
pub const WATER_ZONE:&str="ENT_FLD_SEA_1_2_WTR";
#[derive(Debug,Serialize)]
pub struct Prediction {
    pub northern_return:Entry,
    pub bridge_entry:Entry,
    pub water:Option<Box<Water>>,
    pub conning:Option<Box<crate::conning::Prediction>>,
    pub verification:&'static str,
}
#[derive(Debug,Serialize)]
pub struct Water {
    pub boarding:Entry,
    pub encounter:encounter::Prediction,
    pub battle_entry:crate::CrtRng,
    pub escape:crate::highroad::Escape,
    pub field_return:crate::highroad::FieldReturn,
}
/// Boarding changes the active encounter volume but makes no CRT call and does
/// not reset the counter. The first canoe tutorial updates persistent flags.
pub fn water_arrival(bridge:&Entry)->Result<(Entry,encounter::Prediction,crate::CrtRng)> {
    ensure(bridge.world_after.map==BRIDGE && bridge.world_after.time==2,"Water encounter requires daytime Canalbrine Bridge")?;
    let mut l=Loader::new(world::catalog(),bridge.world_after.clone(),bridge.after)?;
    if !l.state.flag(world::catalog().tutorials[&6].opened_flag) {l.dismiss_tutorial(6)?;}
    let boarding=checkpoint(&mut l,bridge.after,"Canalbrine Bridge: boarded canoe; water volume active; first canoe tutorial dismissed if unseen");
    let w=&boarding.world_after;let mut rng=boarding.after;
    let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions {
        map:BRIDGE.into(),zone:WATER_ZONE.into(),period:"DayTime".into(),progress:w.main_progress_points,
        first_encounter:w.first_encounter,boundary:Boundary::CurrentCounter,current_counter:Some(w.encounter_counter),
        party_levels:w.party.iter().map(|id|w.character_progression[id].level).collect(),
        ability_percent:if w.evasive_maneuvers {50}else{100},dash:true,..Default::default()
    })?;
    Ok((boarding,encounter,rng))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entry()->Entry {
        let mut w=world::throne_after_second_state().unwrap();
        w.map=crate::ryu::TOWN.into();w.time=2;w.party=vec![7,6,8,4];
        w.party_experience=vec![1530,1467,0,0];w.job_licenses.insert(6,1);
        w.active_scenarios.clear();w.story_types.clear();w.prologues.clear();
        w.story_types.insert(7);w.active_scenarios.insert(701);
        w.main_progress_points=4;w.completed_scenarios.extend([700,600,601,800,400]);
        for flag in [36,35,37,33,20509,8019,8050,30510] {w.set_flag(flag,true);}
        w.visited_maps.extend([crate::ryu::TOWN.into(),crate::ryu::NORTHERN.into()]);
        w.resident_levels=world::catalog().maps[&w.map].streams.iter().chain(w.shared_levels.iter()).cloned().collect();
        w.sync_progression().unwrap();w.character_progression.get_mut(&7).unwrap().jp_earned=517;
        let osvald=w.character_progression.get_mut(&6).unwrap();
        osvald.jp_earned=492;osvald.jp_spent=130;
        osvald.jobs.unlocked_support.insert(crate::scholar::EVASIVE.into());
        osvald.jobs.equipped_support.insert(crate::scholar::EVASIVE.into());
        w.refresh_support_skills();w.encounter_counter=749;
        let mut rng=crate::CrtRng::new(0);rng.advance(7000);
        let town=Entry {arrival:rng,after:rng,world_after:w,trace:vec![],decisions:vec![]};
        let hikari=crate::hikari::predict(&town).unwrap();
        crate::scholar::predict(&hikari.recruitment).unwrap().after
    }
    #[test]
    fn ordinary_loads_preserve_all_party_ledgers_and_trace_continuity() {
        let e=entry();let original=crate::state_key::WorldKey::new(&e.world_after);
        let p=predict(&e).unwrap();
        assert_eq!(crate::state_key::WorldKey::new(&e.world_after),original);
        let mut rng=e.after;
        for (stage,map,streams,npcs) in [(&p.northern_return,crate::ryu::NORTHERN,6,2),(&p.bridge_entry,BRIDGE,7,7)] {
            assert_eq!(stage.arrival,rng);
            for s in &stage.trace {assert_eq!(s.before,rng);rng=s.after;}
            assert_eq!(stage.after,rng);assert_eq!(stage.world_after.map,map);
            assert_eq!(stage.trace.iter().filter(|s|s.kind==world::DrawKind::StreamedLevel).count(),streams);
            assert_eq!(stage.trace.iter().filter(|s|s.kind==world::DrawKind::NpcFrame).count(),npcs);
            assert_eq!(stage.trace.iter().filter(|s|s.kind==world::DrawKind::Counter).count(),4);
            assert!(stage.trace.iter().all(|s|!s.reason.contains("Create streamed level Btl_SinExpose")));
            assert!(stage.world_after.first_encounter && stage.world_after.evasive_maneuvers);
            assert!((500..=899).contains(&stage.world_after.encounter_counter));
            assert_eq!(stage.world_after.party,[7,1,8,4]);assert_eq!(stage.world_after.bench,[6]);
            assert_eq!(stage.world_after.main_progress_points,5);
            assert!(stage.world_after.active_scenarios.contains(&701));
            assert_eq!(serde_json::to_value(&stage.world_after.character_progression).unwrap(),serde_json::to_value(&e.world_after.character_progression).unwrap());
            assert_eq!(stage.world_after.enemy_knowledge,e.world_after.enemy_knowledge);
            assert_eq!(stage.world_after.job_licenses,e.world_after.job_licenses);
        }
        assert!(p.bridge_entry.world_after.visited_maps.contains(BRIDGE));
    }
    #[test]
    fn unmeasured_counter_consumption_cannot_change_load_rng_but_residency_can() {
        let e=entry();let expected=predict(&e).unwrap();
        let mut varied=Entry {arrival:e.arrival,after:e.after,world_after:e.world_after.clone(),trace:vec![],decisions:vec![]};
        varied.world_after.encounter_counter=1;
        varied.world_after.first_encounter=false;
        let p=predict(&varied).unwrap();
        for (a,b) in [(&p.northern_return,&expected.northern_return),(&p.bridge_entry,&expected.bridge_entry)] {
            assert_eq!(a.after,b.after);
            assert_eq!(crate::state_key::WorldKey::new(&a.world_after),crate::state_key::WorldKey::new(&b.world_after));
        }
        varied.world_after.resident_levels.remove("Btl_SinExpose");
        let recreated=predict(&varied).unwrap();
        assert_eq!(recreated.northern_return.after.draws,expected.northern_return.after.draws+1);
        assert_eq!(recreated.bridge_entry.after.draws,expected.bridge_entry.after.draws+1);
        assert_eq!(recreated.northern_return.trace.iter().filter(|s|s.reason=="Create streamed level Btl_SinExpose").count(),1);
        assert_ne!(recreated.bridge_entry.after.state,expected.bridge_entry.after.state);
    }
    #[test]
    fn canoe_uses_real_water_tier_and_tutorial_without_a_reset_or_fixed_first_pack() {
        let scholar=entry();let p=predict(&scholar).unwrap();
        let before=crate::state_key::WorldKey::new(&p.bridge_entry.world_after);
        let (boarding,encounter,battle_entry)=water_arrival(&p.bridge_entry).unwrap();
        assert_eq!(boarding.after,p.bridge_entry.after);
        assert_eq!(boarding.world_after.encounter_counter,p.bridge_entry.world_after.encounter_counter);
        assert!(boarding.trace.iter().all(|s|s.kind==world::DrawKind::Boundary));
        let tutorial=&world::catalog().tutorials[&6];
        assert!(boarding.world_after.flag(1005));
        assert!(boarding.world_after.flag(tutorial.opened_flag));
        assert_eq!(boarding.world_after.party,[7,1,8,4]);
        assert_eq!(boarding.world_after.bench,[6]);
        assert_eq!(encounter.encounter_level,Some(11));
        assert_eq!(encounter.encounter_table.as_deref(),Some("ENC_FLD_SEA_1_3_WTR_DAY_030"));
        assert!(!encounter.first_override);
        assert_eq!(battle_entry.draws,boarding.after.draws+encounter.draws);
        assert_eq!(battle_entry.state,encounter.final_state);
        assert_eq!(crate::state_key::WorldKey::new(&p.bridge_entry.world_after),before);
    }
}
fn checkpoint(l:&mut Loader<'_>,arrival:crate::CrtRng,reason:&str)->Entry {
    l.boundary(reason);
    Entry {arrival,after:l.rng,world_after:l.state.clone(),
        trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
pub fn predict(entry:&Entry)->Result<Prediction> {
    ensure(entry.world_after.map==crate::ryu::TOWN && entry.world_after.time==2
        && ((entry.world_after.galdera_route && entry.world_after.party==[7,6,8,4] && entry.world_after.bench==[1])
            || (!entry.world_after.galdera_route && entry.world_after.party==[7,1,8,4] && entry.world_after.bench==[6])),
        "Canalbrine Bridge continuation requires daytime Ryu with the selected route party")?;
    ensure(entry.world_after.evasive_maneuvers &&
        (entry.world_after.galdera_route || entry.world_after.character_progression[&7].jobs.secondary_job==Some(crate::scholar::JOB)),
        "Canalbrine Bridge continuation requires the selected route's Evasive Maneuvers setup")?;
    let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
    // User specifies these legs are encounterless. Movement therefore adds no
    // CRT calls. No distance, safe running share or pre-exit counter is inferred.
    // Both loads reset the counter from source, independently of its old value.
    l.visit_map(crate::ryu::NORTHERN)?;
    let northern_return=checkpoint(&mut l,entry.after,"Northern Ryu Sands: returned from Ryu; field control after loading");
    l.visit_map(BRIDGE)?;
    let bridge_entry=checkpoint(&mut l,northern_return.after,"Canalbrine Bridge: field control after loading");
    Ok(Prediction {northern_return,bridge_entry,water:None,conning:None,
        verification:"Source-derived shared map loads with the carried daytime party, progression, original gear, learned skills, active Evasive Maneuvers, NPC history and resident levels. No encounters on Ryu -> Northern Ryu Sands -> Canalbrine Bridge, as specified by the user. Movement distance and counter consumption before transitions are not inferred; the real outgoing/incoming source resets are executed. New endpoints await retail validation."})
}
