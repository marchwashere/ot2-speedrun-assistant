//! Frozen EXP100 routine -> Gravell tag -> safe coast crossing to Eastern Sai.
use crate::{Result,SimError,CrtRng,ensure,world::{self,Loader},snows_escape::Entry,
    inventor::{GravellArrival,settle_gravell},encounter::{self,PredictionOptions}};
use ot2_battle_engine::{gravell_routine::{self,Action,Fight,Rewards},fight_search::BoostFrontier,escape::Situation};
use serde::Serialize;

#[derive(Debug,Serialize)]
pub struct Movement {
    pub initial_counter:u32,pub walking_updates:u32,pub running_updates:u32,
    pub walk_decrement:u32,pub run_decrement:u32,pub run_percent:f64,
    pub switch_to_walking_counter:u32,pub counter_cost:u32,pub estimated_counter_at_exit:u32,
}
#[derive(Debug,Serialize)]
pub struct Continuation {
    pub day_before_gravell:Entry,pub conning:Entry,
    #[serde(skip_serializing_if="Option::is_none")] pub conning_night:Option<Entry>,
    pub coast_entry:Entry,
    #[serde(skip_serializing_if="Option::is_none")] pub coast_day:Option<Entry>,
    pub movement:Movement,pub coast_exit:Entry,pub eastern_sai:Entry,
}
fn point(l:&mut Loader<'_>,before:CrtRng)->Entry {
    Entry {arrival:before,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
// Measured endpoints 113 walking /107 running, with three extra updates each.
// Enumerate whole running updates; round the remaining walking updates UP.
fn movement(w:&world::WorldState)->Result<Option<Movement>> {
    ensure(w.map==crate::conning::WEST&&w.time==2,"Coast movement requires the daytime Western Coast")?;
    let mut o=PredictionOptions {zone:"ENT_FLD_SEA_2_3_LND".into(),period:"DayTime".into(),
        ability_percent:if w.evasive_maneuvers {50}else{100},dash:false,..Default::default()};
    let walk=encounter::catalog().counter_decrement(&o)?;o.dash=true;
    let run=encounter::catalog().counter_decrement(&o)?;
    for fast in (0u32..=110).rev() {
        let slow=(116*(110-fast)).div_ceil(110);
        let cost=fast*run+slow*walk;
        if cost<w.encounter_counter {return Ok(Some(Movement {initial_counter:w.encounter_counter,
            walking_updates:slow,running_updates:fast,walk_decrement:walk,run_decrement:run,
            run_percent:(fast*1000/110) as f64/10.0,switch_to_walking_counter:w.encounter_counter-fast*run,
            counter_cost:cost,estimated_counter_at_exit:w.encounter_counter-cost}));}
    }
    Ok(None)
}
fn travel(field_return:&Entry,night_pair:bool)->Result<Option<(Entry,Continuation)>> {
    let mut l=Loader::new(world::catalog(),field_return.world_after.clone(),field_return.after)?;
    l.manual_time_switch(2,"Western Gravell: switch to day before tagging Gravell")?;
    let day_before_gravell=point(&mut l,field_return.after);
    // User-confirmed encounterless leg: no invented movement/counter consumption.
    l.visit_map("Twn_Wld_3_1_A")?;let town=point(&mut l,day_before_gravell.after);
    l.visit_map(crate::conning::TOWN)?;let conning=point(&mut l,town.after);
    let conning_night=if night_pair {
        l.manual_time_switch(4,"Conning Creek: switch to night immediately after fast travel")?;
        Some(point(&mut l,conning.after))
    }else{None};
    let before=l.rng;l.visit_map(crate::conning::WEST)?;let coast_entry=point(&mut l,before);
    let coast_day=if night_pair {
        l.manual_time_switch(2,"Western Conning Creek Coast: switch to day immediately after entry")?;
        Some(point(&mut l,coast_entry.after))
    }else{None};
    let Some(movement)=movement(&l.state)? else {return Ok(None);};
    let before=l.rng;l.state.encounter_counter=movement.estimated_counter_at_exit;
    l.boundary(format!("Western Conning Creek Coast -> Eastern Sai Sands: run {:.1}%, then walk; {} running updates x {}, {} walking updates x {}; measured 107/113 +3 leeway, counter {} -> {}",movement.run_percent,movement.running_updates,movement.run_decrement,movement.walking_updates,movement.walk_decrement,movement.initial_counter,movement.estimated_counter_at_exit));
    let coast_exit=point(&mut l,before);
    l.visit_map("Fld_Dst_2_4")?;let eastern_sai=point(&mut l,coast_exit.after);
    Ok(Some((town,Continuation {day_before_gravell,conning,conning_night,coast_entry,coast_day,movement,coast_exit,eastern_sai})))
}
pub(crate) fn extend(g:&mut GravellArrival)->Result<()> {
    let w=&g.night.world_after;
    let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
    let ids:Vec<_>=g.encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
    let situation=if g.encounter.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal};
    let battle=ot2_battle_engine::field_escape::carried_step_ahead(g.battle_entry,situation,&players,&ids,true,&w.enemy_knowledge)?;
    let menu=gravell_routine::finisher_menu(battle.clone(),&g.fight)?;
    let can_adjust=menu.state().actors[menu.current_actor().unwrap().0].resources.bp>0;
    let mut frontier=BoostFrontier::new(&menu)?;
    type Selected=(Fight,Rewards,Entry,Entry,Continuation);
    // Fewest additional boost presses first. At equal cost maximize safe running;
    // the optional landing/entry switch pair has zero user-specified time cost.
    for cost in (0..usize::MAX).step_by(2) {
        let mut best:Option<Selected>=None;
        while let Some(candidate)=frontier.next(&Action::ThunderSoulstone.command(),cost)? {
            let fight=gravell_routine::replay_finisher(battle.clone(),&g.fight,&candidate.inputs)?;
            let (fight,rewards,field_return)=settle_gravell(&g.night,g.battle_entry,fight)?;
            for pair in [false,true] {
                let Some((town,continuation))=travel(&field_return,pair)? else {continue;};
                if best.as_ref().is_none_or(|b|continuation.movement.run_percent>b.4.movement.run_percent) {
                    best=Some((fight.clone(),rewards.clone(),field_return.clone(),town,continuation));
                }
            }
        }
        if let Some((fight,rewards,field_return,town,coast))=best {
            g.fight=fight;g.rewards=rewards;g.field_return=field_return;g.town=town;g.coast=Some(coast);
            return Ok(());
        }
        if !can_adjust {break;}
    }
    Err(SimError("No safe Western Conning Creek Coast continuation".into()))
}

#[cfg(test)] mod tests {
    use super::*;
    #[test]
    fn exact_mixed_counter_budget_never_rounds_walking_down() {
        let f:serde_json::Value=serde_json::from_str(include_str!("../data/source/gravell-complete-route-fixture.json")).unwrap();
        let mut w:world::WorldState=serde_json::from_value(f["before"]["world_after"].clone()).unwrap();
        w.map=crate::conning::WEST.into();w.time=2;w.evasive_maneuvers=true;
        for counter in 0..=1800 {
            w.encounter_counter=counter;
            let m=movement(&w).unwrap();
            let Some(m)=m else {continue;};
            assert_eq!(m.walking_updates,(116*(110-m.running_updates)).div_ceil(110));
            assert_eq!(m.counter_cost,m.walking_updates*m.walk_decrement+m.running_updates*m.run_decrement);
            assert!(m.counter_cost<counter);
            if m.running_updates<110 {
                let faster=m.running_updates+1;
                let slow=(116*(110-faster)).div_ceil(110);
                assert!(faster*m.run_decrement+slow*m.walk_decrement>=counter);
            }
        }
    }
}
