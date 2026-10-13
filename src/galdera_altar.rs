//! Southern Crackridge first escape -> encounterless altar approach -> Negotiate Schedule.
use crate::{Result,ensure,world::{self,Loader},snows_escape::Entry,galdera_coast::Movement};
use serde::Serialize;
pub const MAP:&str="Dng_Wld_2_Job";
const RUN:u32=286;
const WALK:u32=301;
const LEEWAY:u32=3;
#[derive(Debug,Serialize)]
pub struct Visit {
    pub running_updates_before_encounter:u32,
    pub movement:Movement,
    pub approach_exit:Entry,
    pub arrival:Entry,
    pub learned:Entry,
}
fn point(l:&mut Loader<'_>,before:crate::CrtRng)->Entry {
    Entry {arrival:before,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
pub(crate) fn movement(counter:u32,first_run:u32)->Result<Option<Movement>> {
    ensure(first_run<RUN,"Southern encounter must precede the altar")?;
    // First run to the encounter, then budget the remaining measured distance.
    // Round walking up and reserve three extra updates in either speed mode.
    let full_run=RUN-first_run+LEEWAY;
    let full_walk=(WALK*(RUN-first_run)).div_ceil(RUN)+LEEWAY;
    for fast in (0..=full_run).rev() {
        let slow=(full_walk*(full_run-fast)).div_ceil(full_run);
        let cost=fast*7+slow*5;
        if cost<counter {return Ok(Some(Movement {initial_counter:counter,walking_updates:slow,running_updates:fast,
            walk_decrement:5,run_decrement:7,run_percent:(fast*1000/full_run) as f64/10.0,
            switch_to_walking_counter:counter-fast*7,counter_cost:cost,estimated_counter_at_exit:counter-cost}));}
    }
    Ok(None)
}
pub(crate) fn visit(entry:&Entry,first_run:u32)->Result<Option<Visit>> {
    let w=&entry.world_after;
    ensure(w.galdera_route&&w.map=="Fld_Wld_2_2"&&w.time==2&&w.evasive_maneuvers,"Altar approach requires Galdera daytime/Evasive Southern escape")?;
    let Some(movement)=movement(w.encounter_counter,first_run)? else {return Ok(None);};
    let mut l=Loader::new(world::catalog(),w.clone(),entry.after)?;
    l.state.encounter_counter=movement.estimated_counter_at_exit;
    l.boundary(format!("Southern -> Altar: ran {first_run} updates before encounter; after Escape run {:.1}% then walk; {} run x7 + {} walk x5, 3-update leeway, no second encounter",movement.run_percent,movement.running_updates,movement.walking_updates));
    let approach_exit=point(&mut l,entry.after);
    l.visit_map(MAP)?;let arrival=point(&mut l,approach_exit.after);
    ensure(l.state.party.contains(&4),"Partitio must be present at the altar")?;
    l.skipped_event("SHRINE_SHO_0100")?;
    ensure(l.state.flag(2503)&&l.state.character_progression[&4].jobs.ex_abilities.contains("ABI_SET_MER_090"),"Altar did not grant Negotiate Schedule")?;
    let learned=point(&mut l,arrival.after);
    Ok(Some(Visit {running_updates_before_encounter:first_run,movement,approach_exit,arrival,learned}))
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn altar_grants_ex2_without_regular_skill_jp_support_or_rng_changes() {
        let f:serde_json::Value=serde_json::from_str(include_str!("../data/source/gravell-complete-route-fixture.json")).unwrap();
        let mut w:world::WorldState=serde_json::from_value(f["before"]["world_after"].clone()).unwrap();
        w.galdera_route=true;w.time=2;w.map="Fld_Wld_2_2".into();w.evasive_maneuvers=true;w.encounter_counter=1400;
        let before=w.character_progression[&4].clone();
        let rng=crate::CrtRng::new(77);
        let e=Entry {arrival:rng,after:rng,world_after:w,trace:vec![],decisions:vec![]};
        let v=visit(&e,100).unwrap().unwrap();
        let after=&v.learned.world_after.character_progression[&4];
        assert_eq!(v.arrival.after,v.learned.after);
        assert_eq!(before.jp_spent,after.jp_spent);
        assert_eq!(before.jobs.learned_abilities,after.jobs.learned_abilities);
        assert_eq!(before.jobs.unlocked_support,after.jobs.unlocked_support);
        assert!(after.jobs.ex_abilities.contains("ABI_SET_MER_090"));
        assert!(!after.jobs.ex_abilities.contains("ABI_SET_MER_080"));
        assert!(v.learned.world_after.flag(2503));
        let mut l=Loader::new(world::catalog(),v.learned.world_after.clone(),v.learned.after).unwrap();
        l.visit_map("Fld_Wld_2_2").unwrap();
        assert!(l.state.first_encounter);
        assert!((500..=899).contains(&l.state.encounter_counter));
    }
    #[test]fn remaining_distance_preserves_padding_and_requires_positive_counter() {
        for first in 72..=129 {for counter in 1000..=1798 {
            let m=movement(counter,first).unwrap();
            let walk=(301*(286-first)).div_ceil(286)+3;
            assert_eq!(m.is_some(),walk*5<counter);
            if let Some(m)=m {assert!(m.estimated_counter_at_exit>0);assert_eq!(m.counter_cost,m.running_updates*7+m.walking_updates*5);}
        }}
        assert!(movement(1000,72).unwrap().is_none());
        assert_eq!(movement(1798,72).unwrap().unwrap().run_percent,100.0);
    }
}
