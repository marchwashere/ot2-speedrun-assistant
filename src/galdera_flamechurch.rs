//! Al delivery -> safe 56-update run -> one Western Clockbank land encounter.
use crate::{Result,CrtRng,ensure,world::{self,Loader},snows_escape::Entry,galdera_coast::{self,SouthernSai},encounter::{self,PredictionOptions}};
use serde::Serialize;
const TO_CLOCKBANK:u32=56;
const TO_FLAMECHURCH:u32=193;
const PASS_RUN:u32=133+3;
const PASS_WALK:u32=137+3;
#[derive(Debug,Serialize)]pub struct Trip {
    pub highroad_exit:Entry,pub clockbank_entry:Entry,
    #[serde(skip_serializing_if="Option::is_none")]pub adjustment:Option<Entry>,
    pub additional_time_switches:u32,pub encounter:SouthernSai,pub clockbank_exit:Entry,pub eastern_flamechurch:Entry,
    #[serde(skip_serializing_if="Option::is_none")]pub temenos:Option<crate::galdera_temenos::Trip>,pub escape_boost_cycles:u32,pub movement:galdera_coast::Movement,pub pass_exit:Entry,pub flamechurch:Entry,
    pub highroad_running_updates:u32,pub clockbank_running_updates:u32,pub running_updates_after_escape:u32,pub run_decrement:u32,
}
fn point(l:&mut Loader<'_>,before:CrtRng)->Entry {
    Entry {arrival:before,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
fn movement(w:&world::WorldState)->Result<Option<galdera_coast::Movement>> {
    let mut o=PredictionOptions {zone:"ENT_FLD_MNT_1_2_LND".into(),period:"DayTime".into(),ability_percent:if w.evasive_maneuvers {50}else{100},dash:false,..Default::default()};
    let walk=encounter::catalog().counter_decrement(&o)?;o.dash=true;
    let run=encounter::catalog().counter_decrement(&o)?;
    for fast in (0u32..=PASS_RUN).rev() {
        let slow=(PASS_WALK*(PASS_RUN-fast)).div_ceil(PASS_RUN);
        let cost=fast*run+slow*walk;
        if cost<w.encounter_counter {return Ok(Some(galdera_coast::Movement {initial_counter:w.encounter_counter,walking_updates:slow,running_updates:fast,walk_decrement:walk,run_decrement:run,run_percent:(fast*1000/PASS_RUN) as f64/10.0,switch_to_walking_counter:w.encounter_counter-fast*run,counter_cost:cost,estimated_counter_at_exit:w.encounter_counter-cost}));}
    }
    Ok(None)
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
    let mut encounter=loop {
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
    let mut escape_boost_cycles=0;
    let mut cursor=encounter.menu.clone();
    let (clockbank_exit,eastern_flamechurch,movement,pass_exit,flamechurch)=loop {
        let escape=cursor.clone().attempt(&[],encounter.encounter.encounter_level.unwrap())?;
        ensure(escape.success&&escape.chance_percent==100,"Clockbank Escape must remain guaranteed")?;
        let w=&encounter.arrival.world_after;
        let ret=crate::highroad::restore_field(w,&escape,&w.map,"Clockbank Escape before encounterless Flamechurch passage")?;
        let field_return=Entry {arrival:ret.arrival,after:ret.after,world_after:ret.world_after,trace:ret.trace,decisions:vec![]};
        let mut l=Loader::new(world::catalog(),field_return.world_after.clone(),field_return.after)?;
        ensure(l.state.encounter_counter>remaining*run_decrement,"A second encounter would occur before Eastern Flamechurch")?;
        l.state.encounter_counter-=remaining*run_decrement;
        l.boundary(format!("Western Clockbank -> Eastern Flamechurch Pass: run 100% after Escape; {remaining} remaining updates x{run_decrement}, no second encounter"));
        let clockbank_exit=point(&mut l,field_return.after);
        l.visit_map("Fld_Mnt_1_2")?;let eastern=point(&mut l,clockbank_exit.after);
        if let Some(m)=movement(&l.state)? {
            l.state.encounter_counter-=m.counter_cost;
            l.boundary(format!("Eastern Flamechurch Pass -> Flamechurch: run {:.1}%, then walk; {} running updates x{}, {} walking updates x{}; measured 133/137 endpoints +3 leeway each",m.run_percent,m.running_updates,m.run_decrement,m.walking_updates,m.walk_decrement));
            let pass_exit=point(&mut l,eastern.after);
            l.visit_map("Twn_Mnt_1_1_A")?;l.skipped_event("MapInStorySelect")?;
            let town=point(&mut l,pass_exit.after);
            encounter.escape=escape;encounter.field_return=field_return;
            break (clockbank_exit,eastern,m,pass_exit,town);
        }
        cursor.boost_cancel_cycle();escape_boost_cycles+=1;
    };
    let temenos=if flamechurch.world_after.galdera_route {Some(crate::galdera_temenos::finish(&flamechurch)?)}else{None};
    Ok(Trip {temenos,escape_boost_cycles,movement,pass_exit,flamechurch,highroad_exit,clockbank_entry,adjustment,additional_time_switches,encounter,clockbank_exit,eastern_flamechurch,highroad_running_updates:TO_CLOCKBANK,clockbank_running_updates:TO_FLAMECHURCH,running_updates_after_escape:remaining,run_decrement})
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]
    fn full_run_has_one_safe_encounter_and_adjustment_replays_native_callbacks() {
        let f:serde_json::Value=serde_json::from_str(include_str!("../data/source/gravell-complete-route-fixture.json")).unwrap();
        let mut w:world::WorldState=serde_json::from_value(f["before"]["world_after"].clone()).unwrap();
        w.job_licenses.insert(2,1);w.job_licenses.insert(5,1);w.party=vec![7,6,8,4];w.bench=vec![1,3];w.party_experience=w.party.iter().map(|id|w.character_progression[id].experience).collect();
        w.character_progression.get_mut(&6).unwrap().jobs.equipped_support.insert("ABI_SCH_SUP_01".into());
        w.award_victory(ot2_battle_engine::progression::Reward {exp:300_000,jp:0,money:0}).unwrap();
        w.encounter_counter=560;assert!(movement(&w).unwrap().is_none());
        w.encounter_counter=561;let walk=movement(&w).unwrap().unwrap();assert_eq!((walk.running_updates,walk.walking_updates,walk.estimated_counter_at_exit),(0,140,1));
        w.encounter_counter=816;assert!(movement(&w).unwrap().unwrap().run_percent<100.0);
        w.encounter_counter=817;assert_eq!(movement(&w).unwrap().unwrap().run_percent,100.0);
        w.map="Fld_Cty_1_3".into();w.time=2;w.encounter_counter=500;w.galdera_route=true;w.set_flag(20511,true);
        let mut direct=false;let mut adjusted=false;let mut boost_adjusted=false;
        for seed in 0u32..1024 {
            let rng=CrtRng::new(seed.wrapping_mul(1775700753));
            let entry=Entry {arrival:rng,after:rng,world_after:w.clone(),trace:vec![],decisions:vec![]};
            let t=finish(&entry).unwrap();
            boost_adjusted|=t.escape_boost_cycles>0;
            let inputs:Vec<_>=(0..t.escape_boost_cycles).flat_map(|_|[1,0]).collect();
            let escaped=t.encounter.menu.clone().attempt(&inputs,t.encounter.encounter.encounter_level.unwrap()).unwrap();
            assert_eq!(escaped.after,t.encounter.escape.after);
            assert!(t.pass_exit.world_after.encounter_counter>0);
            assert_eq!(t.pass_exit.after,t.eastern_flamechurch.after);
            assert_eq!(t.movement.counter_cost,t.movement.running_updates*6+t.movement.walking_updates*4);
            assert_eq!(t.flamechurch.world_after.map,"Twn_Mnt_1_1_A");
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
            if direct&&adjusted&&boost_adjusted {return;}
        }
        panic!("Exercise direct departure and actual Clockbank time-pair adjustment");
    }
}
