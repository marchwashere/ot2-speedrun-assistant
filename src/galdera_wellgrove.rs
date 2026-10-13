//! Winterbloom -> Sai daytime -> Southern/Western Sai -> Eastern Wellgrove.
use crate::{Result,CrtRng,world::{self,Loader},snows_escape::Entry,galdera_coast::{self,SouthernSai}};
use serde::Serialize;

#[derive(Debug,Serialize)]
pub struct Trip {
    pub sai:Entry,pub day:Entry,pub southern_sai:Entry,
    pub western_sai:SouthernSai,pub eastern_wellgrove:Entry,
    pub additional_time_switches:u32,
    pub western_escape_boost_cycles:u32,pub champion:crate::galdera_champion::Trip,
}
fn point(l:&mut Loader<'_>,before:CrtRng)->Entry {
    Entry {arrival:before,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
pub(crate) fn finish(entry:&Entry)->Result<Trip> {
    let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
    l.visit_map("Twn_Dst_2_1_A")?;l.skipped_event("MapInStorySelect")?;
    let sai=point(&mut l,entry.after);
    l.manual_time_switch(2,"Sai: switch to day immediately after fast travel")?;
    let mut day=point(&mut l,sai.after);
    // User confirms this crossing has no encounter. Distance is unmeasured;
    // load each map normally without inventing counter decrements or battles.
    let mut additional_time_switches=0;
    let (southern_sai,mut western_sai)=loop {
        let mut probe=Loader::new(world::catalog(),day.world_after.clone(),day.after)?;
        probe.visit_map("Fld_Dst_2_3")?;let southern=point(&mut probe,day.after);
        probe.visit_map("Fld_Dst_2_2")?;let western=point(&mut probe,southern.after);
        if let Some(fight)=galdera_coast::sai_encounter(western,"ENT_FLD_DST_2_2_LND")? {break (southern,fight);}
        // The latest available control is Sai's time switch. Replay a complete
        // night/day pair and re-evaluate native loads; never rewind prior fights.
        l.manual_time_switch(4,"Sai: additional night switch to avoid Western Sai ambush")?;
        l.manual_time_switch(2,"Sai: return to day before Southern Sai departure")?;
        additional_time_switches+=2;
        let next=point(&mut l,day.after);
        day.after=next.after;day.world_after=next.world_after;
        day.trace.extend(next.trace);day.decisions.extend(next.decisions);
    };
    let mut western_escape_boost_cycles=0;
    let mut cursor=western_sai.menu.clone();
    let (eastern_wellgrove,champion)=loop {
        let escape=cursor.clone().attempt(&[],western_sai.encounter.encounter_level.unwrap())?;
        let w=&western_sai.arrival.world_after;
        let ret=crate::highroad::restore_field(w,&escape,&w.map,"Western Sai Escape before Champion Seeker")?;
        let field_return=Entry {arrival:ret.arrival,after:ret.after,world_after:ret.world_after,trace:ret.trace,decisions:vec![]};
        let mut l=Loader::new(world::catalog(),field_return.world_after.clone(),field_return.after)?;
        l.visit_map("Fld_Fst_2_2")?;let eastern=point(&mut l,field_return.after);
        if let Some(champion)=crate::galdera_champion::finish(&eastern)? {
            western_sai.escape=escape;western_sai.field_return=field_return;
            break (eastern,champion);
        }
        cursor.boost_cancel_cycle();western_escape_boost_cycles+=1;
    };
    Ok(Trip {sai,day,southern_sai,western_sai,eastern_wellgrove,additional_time_switches,western_escape_boost_cycles,champion})
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]
    fn nearest_sai_switches_preserve_prior_state_and_avoid_ambush() {
        let f:serde_json::Value=serde_json::from_str(include_str!("../data/source/gravell-complete-route-fixture.json")).unwrap();
        let mut state:world::WorldState=serde_json::from_value(f["before"]["world_after"].clone()).unwrap();
        state.award_victory(ot2_battle_engine::progression::Reward {exp:300_000,jp:0,money:0}).unwrap();
        state.time=4;state.galdera_route=true;
        let mut direct=false;let mut adjusted=false;let mut western_adjusted=false;
        for seed in 0u32..128 {
            let rng=CrtRng::new(seed.wrapping_mul(1775700753));
            let e=Entry {arrival:rng,after:rng,world_after:state.clone(),trace:vec![],decisions:vec![]};
            let t=finish(&e).unwrap();
            direct|=t.additional_time_switches==0;adjusted|=t.additional_time_switches>0;
            western_adjusted|=t.western_escape_boost_cycles>0;
            assert_eq!(t.additional_time_switches%2,0);
            assert_eq!(t.sai.arrival,rng);
            assert_ne!(t.western_sai.encounter.situation.as_deref(),Some("Surprise"));
            assert_eq!(t.western_sai.escape.chance_percent,100);assert!(t.western_sai.escape.success);
            assert_eq!(t.eastern_wellgrove.world_after.map,"Fld_Fst_2_2");
            assert_eq!(t.eastern_wellgrove.world_after.time,2);
            let mut replay=Loader::new(world::catalog(),t.sai.world_after.clone(),t.sai.after).unwrap();
            replay.manual_time_switch(2,"test immediate day").unwrap();
            for _ in 0..t.additional_time_switches/2 {
                replay.manual_time_switch(4,"test night").unwrap();replay.manual_time_switch(2,"test day").unwrap();
            }
            assert_eq!(replay.rng,t.day.after);
            assert_eq!(t.eastern_wellgrove.world_after.route_items,state.route_items);
            let inputs:Vec<_>=(0..t.western_escape_boost_cycles).flat_map(|_|[1,0]).collect();
            let escape=t.western_sai.menu.clone().attempt(&inputs,t.western_sai.encounter.encounter_level.unwrap()).unwrap();
            assert_eq!(escape.after,t.western_sai.escape.after);
            assert_eq!(t.champion.steal.after.draws,t.champion.approach.after.draws+2);
            assert_ne!(t.champion.encounter.encounter.situation.as_deref(),Some("Surprise"));
            assert_eq!(t.champion.encounter.escape.chance_percent,100);
            assert_eq!(t.champion.wellgrove.world_after.map,"Twn_Fst_2_1_A");
            assert_eq!(t.champion.wellgrove.world_after.route_items.get("ITM_EQP_DGR_120"),Some(&1));
            assert!(t.champion.exit.world_after.encounter_counter>0);
            if direct&&adjusted&&western_adjusted {return;}
        }
        panic!("Exercise direct departure, Sai time-cycle adjustment and Western Escape boosts");
    }
}
