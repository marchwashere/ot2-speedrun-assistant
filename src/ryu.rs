//! Southern Oresrush return -> Northern Ryu Sands -> Ryu and Hikari recruitment.
use crate::{Result,ensure,snows_escape::Entry,winter_voyage::CropdaleReach,
    world::{self,Loader}};
use serde::Serialize;

pub const NORTHERN:&str="Fld_Dst_1_1";
pub const TOWN:&str="Twn_Dst_1_1_A";
// Measured143 walking /137 running updates, plus3 movement-error allowance.
pub const WALK_UPDATES:u32=146;
pub const RUN_UPDATES:u32=140;

#[derive(Debug,Serialize)]
pub struct Prediction {
    pub northern_entry:Entry,
    pub town_reach:Option<CropdaleReach>,
    pub town_entry:Option<Entry>,
    pub hikari:Option<Box<crate::hikari::Prediction>>,
    pub throne_scholar:Option<Box<crate::scholar::Prediction>>,
    pub canalbrine:Option<Box<crate::canalbrine::Prediction>>,
    pub hikari_search:Option<crate::hikari_manipulation::Search>,
    pub verification:&'static str,
}

pub fn town_reach(state:&world::WorldState)->Result<Option<CropdaleReach>> {
    ensure(state.map==NORTHERN,"Ryu approach must start on Northern Ryu Sands")?;
    crate::winter_voyage::measured_reach(state,"ENT_FLD_DST_1_1_LND",WALK_UPDATES,RUN_UPDATES)
}

pub fn predict(southern:&Entry,reach:&CropdaleReach)->Result<Prediction> {
    ensure(southern.world_after.map==crate::oresrush::SOUTHERN,
        "Northern Ryu entry requires the Southern Oresrush return")?;
    ensure(southern.world_after.party==[7,6,8,4] && southern.world_after.time==2,
        "Ryu extension requires the daytime carried four-character party")?;
    ensure(reach.initial_counter==southern.world_after.encounter_counter && reach.estimated_counter_at_exit>0,
        "Southern Oresrush movement budget does not match the carried return counter")?;
    let mut state=southern.world_after.clone();
    state.encounter_counter=reach.estimated_counter_at_exit;
    state.sync_progression()?;
    let mut l=Loader::new(world::catalog(),state,southern.after)?;
    l.visit_map(NORTHERN)?;
    l.boundary("Northern Ryu Sands: field control after loading");
    let northern_entry=Entry {arrival:southern.after,after:l.rng,world_after:l.state.clone(),
        trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)};
    let town_reach=town_reach(&l.state)?;
    let town_entry=if let Some(reach)=&town_reach {
        l.state.encounter_counter=reach.estimated_counter_at_exit;
        l.visit_map(TOWN)?;
        l.boundary(if l.state.galdera_route {"Ryu: field control after loading; Osvald retained"} else {"Ryu: field control after loading; before Hikari recruitment"});
        Some(Entry {arrival:northern_entry.after,after:l.rng,world_after:l.state,
            trace:l.trace,decisions:l.decisions})
    } else {None};
    Ok(Prediction {northern_entry,town_reach,town_entry,hikari:None,throne_scholar:None,canalbrine:None,hikari_search:None,
        verification:"Source-derived shared map loads with carried progression, party, NPC and level state. Northern Ryu Sands -> Ryu uses measured143 walking /137 running updates plus3 leeway. Mixed distance share interpolates the two measured densities, reserves1 counter point and rounds down to0.1%. Stops before Hikari recruitment; new map endpoints await retail validation."})
}

/// Movement search tests the complete town continuation first. Append the
/// Hikari/water joint search only after selecting that valid town arrival,
/// never on rejected movement trials. Earlier inputs and states are unchanged.
pub fn recruit(prediction:&mut Prediction)->Result<()> {
    let town=prediction.town_entry.as_ref().ok_or_else(||crate::SimError("Hikari recruit needs a safe Ryu arrival".into()))?;
    ensure(prediction.canalbrine.is_none(),"Hikari recruitment was already appended")?;
    let selected=crate::hikari_manipulation::predict(town)?;
    prediction.hikari=selected.hikari;
    prediction.throne_scholar=selected.scholar;
    prediction.canalbrine=Some(selected.canalbrine);
    prediction.hikari_search=Some(selected.search);
    prediction.verification="Source-derived shared map loads, Hikari recruitment, Throné's Scholar assignment and Canalbrine Bridge water Escape with carried party, EXP/JP, skills, supports and gear. Northern Ryu Sands -> Ryu uses measured143 walking /137 running updates plus3 leeway, reserving1 counter point and rounding the mixed run share down to0.1%. Lightning Bolt and Elemental Barrage cost130 JP; equipping Evasive restores the party effect without CRT draws or counter reset. Hikari's second menu and the first water Escape menu minimize combined boost inputs while requiring no surprise and first Escape success. Recruit fight and final endpoint await retail validation.";
    if town.world_after.galdera_route {
        prediction.verification="Galdera route keeps Osvald in the party; no Hikari battle, Throné Scholar assignment, skill purchases or support equip. Canalbrine water manipulation uses the carried party and actual progression.";
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn both_map_loads_preserve_the_party_ledger_and_real_rng_continuity() {
        let mut w=world::throne_after_second_state().unwrap();
        w.map=crate::oresrush::SOUTHERN.into();w.time=2;w.party=vec![7,6,8,4];
        w.party_experience=vec![1530,1467,0,0];w.evasive_maneuvers=true;
        w.story_types.clear();w.active_scenarios.clear();w.prologues.clear();
        w.story_types.insert(7);w.active_scenarios.insert(701);
        w.set_flag(8019,true);w.set_flag(8050,true);w.set_flag(30510,true);
        w.main_progress_points=4;w.completed_scenarios.extend([700,600,601,800,400]);
        w.resident_levels=world::catalog().maps[&w.map].streams.iter()
            .chain(w.shared_levels.iter()).cloned().collect();
        w.encounter_counter=758;w.sync_progression().unwrap();
        w.character_progression.get_mut(&6).unwrap().jp_earned=487;
        w.character_progression.get_mut(&6).unwrap().jp_spent=130;
        let immutable=crate::state_key::WorldKey::new(&w);
        let ledger=serde_json::to_value(&w.character_progression).unwrap();
        let reach=crate::oresrush::ryu_reach(&w).unwrap().unwrap();
        let mut safe=false;let mut unsafe_counter=false;
        for seed in [0,107] {
            let rng=crate::CrtRng::new(seed);
            let southern=Entry {arrival:rng,after:rng,world_after:w.clone(),trace:vec![],decisions:vec![]};
            let p=predict(&southern,&reach).unwrap();
            assert_eq!(p.northern_entry.world_after.map,NORTHERN);
            assert_eq!(p.northern_entry.arrival,rng);
            let mut cursor=rng;
            for entry in std::iter::once(&p.northern_entry).chain(p.town_entry.iter()) {
                assert_eq!(entry.arrival,cursor);
                for span in &entry.trace {assert_eq!(span.before,cursor);cursor=span.after;}
                assert_eq!(cursor,entry.after);
                assert_eq!(entry.world_after.party,w.party);
                assert_eq!(entry.world_after.party_experience,w.party_experience);
                assert_eq!(serde_json::to_value(&entry.world_after.character_progression).unwrap(),ledger);
                assert!(entry.world_after.evasive_maneuvers);
                assert_eq!(entry.world_after.main_progress_points,4);
                assert!(!entry.world_after.character_progression.contains_key(&1),"Hikari recruitment is outside this extension");
            }
            assert_eq!(p.town_reach.is_some(),p.town_entry.is_some());
            if let Some(town)=&p.town_entry {
                assert_eq!(town.world_after.map,TOWN);
                assert!(town.world_after.visited_maps.contains(TOWN));
                assert!(p.town_reach.as_ref().unwrap().estimated_counter_at_exit>0);
                safe=true;
            } else {unsafe_counter=true;}
        }
        assert!(safe&&unsafe_counter,"Both safe and rejected fresh Northern counters must be covered");
        assert_eq!(crate::state_key::WorldKey::new(&w),immutable);
    }
    #[test]
    fn padded_ryu_limits_use_actual_support_and_strict_thresholds() {
        let mut w=world::throne_after_second_state().unwrap();
        w.map=NORTHERN.into();w.time=2;w.evasive_maneuvers=true;
        for (counter,percent,left) in [(585,0.0,1),(713,50.0,1),(840,99.6,1),(841,100.0,1)] {
            w.encounter_counter=counter;
            let before=crate::state_key::WorldKey::new(&w);
            let r=town_reach(&w).unwrap().unwrap();
            assert_eq!((r.walking_updates,r.running_updates,r.walking_cost,r.running_cost),(146,140,584,840));
            assert!((r.run_percent-percent).abs()<0.000001);
            assert_eq!(r.estimated_counter_at_exit,left);
            assert_eq!(crate::state_key::WorldKey::new(&w),before);
        }
        w.encounter_counter=584;assert!(town_reach(&w).unwrap().is_none());
        w.encounter_counter=899;w.evasive_maneuvers=false;assert!(town_reach(&w).unwrap().is_none());
        w.evasive_maneuvers=true;w.time=4;assert!(town_reach(&w).unwrap().is_none());
    }
}
