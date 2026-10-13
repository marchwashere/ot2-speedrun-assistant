//! Al delivery -> safe 56-update run -> one Western Clockbank land encounter.
use crate::{Result,CrtRng,ensure,world::{self,Loader},snows_escape::Entry,galdera_coast::{self,SouthernSai},encounter::{self,PredictionOptions}};
use serde::Serialize;
const TO_CLOCKBANK:u32=56;
const TO_FLAMECHURCH:u32=193;
#[derive(Debug,Serialize)]pub struct Trip {
    pub highroad_exit:Entry,pub clockbank_entry:Entry,
    #[serde(skip_serializing_if="Option::is_none")]pub adjustment:Option<Entry>,
    pub additional_time_switches:u32,pub encounter:SouthernSai,pub clockbank_exit:Entry,pub eastern_flamechurch:Entry,
    pub highroad_running_updates:u32,pub clockbank_running_updates:u32,pub running_updates_after_escape:u32,pub run_decrement:u32,
}
fn point(l:&mut Loader<'_>,before:CrtRng)->Entry {
    Entry {arrival:before,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
pub(crate) fn finish(entry:&Entry)->Result<Trip> {
    let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
    ensure(l.state.map=="Fld_Cty_1_3"&&l.state.time==2,"Flamechurch leg requires daytime after Al delivery")?;
    let ability=if l.state.evasive_maneuvers {50}else{100};
    let decrement=|zone:&str|encounter::catalog().counter_decrement(&PredictionOptions {zone:zone.into(),period:"DayTime".into(),ability_percent:ability,dash:true,..Default::default()});
    let highroad_cost=TO_CLOCKBANK*decrement("ENT_FLD_CTY_1_3_LND")?;
    ensure(l.state.encounter_counter>highroad_cost,"Al -> Western Clockbank run would encounter an enemy")?;
    l.state.encounter_counter-=highroad_cost;
    l.boundary(format!("Al -> Western Clockbank Highroad: run 100%; {TO_CLOCKBANK} updates, cost{highroad_cost}, no encounter"));
    let highroad_exit=point(&mut l,entry.after);
    l.visit_map("Fld_Cty_1_4")?;let clockbank_entry=point(&mut l,highroad_exit.after);
    let run_decrement=decrement("ENT_FLD_CTY_1_4_LND")?;
    let mut additional_time_switches=0;
    let mut start=clockbank_entry.clone();let mut adjustment:Option<Entry>=None;
    let encounter=loop {
        if let Some(fight)=galdera_coast::sai_encounter(start.clone(),"ENT_FLD_CTY_1_4_LND")? {break fight;}
        // Latest control, after earlier events are frozen. Actual callbacks
        // preserve the encounter counter while returning to daytime.
        l.manual_time_switch(4,"Western Clockbank: night switch to avoid ambush")?;
        l.manual_time_switch(2,"Western Clockbank: return to day before full run")?;
        additional_time_switches+=2;
        let next=point(&mut l,start.after);
        if let Some(a)=&mut adjustment {a.after=next.after;a.world_after=next.world_after;a.trace.extend(next.trace);a.decisions.extend(next.decisions);}
        else {adjustment=Some(next);}
        start=adjustment.as_ref().unwrap().clone();
    };
    let before_encounter=encounter.encounter.counter_updates.unwrap();
    ensure(before_encounter<=TO_FLAMECHURCH,"Expected Western Clockbank encounter lies beyond exit")?;
    let remaining=TO_FLAMECHURCH-before_encounter;
    let mut l=Loader::new(world::catalog(),encounter.field_return.world_after.clone(),encounter.field_return.after)?;
    ensure(l.state.encounter_counter>remaining*run_decrement,"A second encounter would occur before Eastern Flamechurch")?;
    l.state.encounter_counter-=remaining*run_decrement;
    l.boundary(format!("Western Clockbank -> Eastern Flamechurch Pass: run 100% after Escape; {remaining} remaining updates x{run_decrement}, no second encounter"));
    let clockbank_exit=point(&mut l,encounter.field_return.after);
    l.visit_map("Fld_Mnt_1_2")?;let eastern_flamechurch=point(&mut l,clockbank_exit.after);
    Ok(Trip {highroad_exit,clockbank_entry,adjustment,additional_time_switches,encounter,clockbank_exit,eastern_flamechurch,highroad_running_updates:TO_CLOCKBANK,clockbank_running_updates:TO_FLAMECHURCH,running_updates_after_escape:remaining,run_decrement})
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]
    fn full_run_has_one_safe_encounter_and_adjustment_replays_native_callbacks() {
        let f:serde_json::Value=serde_json::from_str(include_str!("../data/source/gravell-complete-route-fixture.json")).unwrap();
        let mut w:world::WorldState=serde_json::from_value(f["before"]["world_after"].clone()).unwrap();
        w.award_victory(ot2_battle_engine::progression::Reward {exp:300_000,jp:0,money:0}).unwrap();
        w.map="Fld_Cty_1_3".into();w.time=2;w.encounter_counter=500;w.galdera_route=true;w.set_flag(20511,true);
        let mut direct=false;let mut adjusted=false;
        for seed in 0u32..1024 {
            let rng=CrtRng::new(seed.wrapping_mul(1775700753));
            let entry=Entry {arrival:rng,after:rng,world_after:w.clone(),trace:vec![],decisions:vec![]};
            let t=finish(&entry).unwrap();
            direct|=t.additional_time_switches==0;adjusted|=t.additional_time_switches>0;
            assert_eq!(t.highroad_exit.world_after.encounter_counter,164);
            assert_eq!(t.highroad_exit.after,rng,"Movement draws no RNG");
            assert_eq!(t.encounter.encounter.counter_updates.unwrap()+t.running_updates_after_escape,193);
            assert!((84..=150).contains(&t.encounter.encounter.counter_updates.unwrap()));
            assert_ne!(t.encounter.encounter.situation.as_deref(),Some("Surprise"));
            assert_eq!(t.encounter.escape.chance_percent,100);assert!(t.encounter.escape.success);
            assert!(t.clockbank_exit.world_after.encounter_counter>0);
            assert_eq!(t.eastern_flamechurch.world_after.map,"Fld_Mnt_1_2");
            assert_eq!(t.eastern_flamechurch.world_after.time,2);
            assert_eq!(t.eastern_flamechurch.world_after.route_items,w.route_items);
            let mut replay=Loader::new(world::catalog(),t.clockbank_entry.world_after.clone(),t.clockbank_entry.after).unwrap();
            for _ in 0..t.additional_time_switches/2 {replay.manual_time_switch(4,"test night").unwrap();replay.manual_time_switch(2,"test day").unwrap();}
            assert_eq!(replay.rng,t.encounter.arrival.after);
            assert_eq!(replay.state.encounter_counter,t.clockbank_entry.world_after.encounter_counter);
            if direct&&adjusted {return;}
        }
        panic!("Exercise direct departure and actual Clockbank time-pair adjustment");
    }
}
