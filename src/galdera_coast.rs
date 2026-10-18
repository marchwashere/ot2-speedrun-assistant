//! Frozen EXP100 routine -> Gravell tag -> safe coast crossing to Eastern Sai.
use crate::{Result,SimError,CrtRng,ensure,world::{self,Loader},snows_escape::Entry,
    inventor::{GravellArrival,settle_gravell},encounter::{self,Boundary,PredictionOptions}};
use ot2_battle_engine::{gravell_routine::{self,Action,Fight,Rewards},fight_search::BoostFrontier,escape::Situation};
use ot2_battle_engine::field_escape::{FieldEscapeMenu,FieldEscapeAttempt};
use serde::Serialize;

#[derive(Debug,Clone,Serialize)]
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
    pub movement:Movement,pub coast_exit:Entry,pub eastern_sai:Entry,pub eastern_sai_encounter:SouthernSai,pub southern_sai:SouthernSai,pub sai:Entry,#[serde(skip_serializing_if="Option::is_none")]pub oresrush_supplies:Option<crate::galdera_oresrush::Supplies>,
}
#[derive(Debug,Clone,Serialize)]
pub struct SouthernSai {
    pub arrival:Entry,pub encounter:encounter::Prediction,pub battle_entry:CrtRng,
    pub menu:FieldEscapeMenu,pub escape:FieldEscapeAttempt,pub field_return:Entry,
    pub defends:Vec<ot2_battle_engine::StepReport>,pub defend_rounds:Vec<u32>,pub escape_round:u32,
}
fn southern_sai(eastern:&Entry)->Result<Option<SouthernSai>> {
    let mut l=Loader::new(world::catalog(),eastern.world_after.clone(),eastern.after)?;
    l.visit_map("Fld_Dst_2_3")?;let arrival=point(&mut l,eastern.after);
    sai_encounter(arrival,"ENT_FLD_DST_2_3_LND")
}
pub(crate) fn sai_encounter(arrival:Entry,zone:&str)->Result<Option<SouthernSai>> {sai_encounter_defends(arrival,zone,0)}
pub(crate) fn sai_encounter_defends(arrival:Entry,zone:&str,count:u8)->Result<Option<SouthernSai>> {
    let w=&arrival.world_after;
    let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
    let mut rng=arrival.after;
    let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions {
        map:w.map.clone(),zone:w.encounter_zone(zone)?.into(),period:"DayTime".into(),
        progress:w.main_progress_points,first_encounter:w.first_encounter,
        boundary:Boundary::CurrentCounter,current_counter:Some(w.encounter_counter),
        party_levels:levels.clone(),ability_percent:if w.evasive_maneuvers {50}else{100},dash:true,
        ..Default::default()
    })?;
    let situation=match encounter.situation.as_deref() {
        Some("Normal")=>Situation::Normal,Some("Preemptive")=>Situation::Preemptive,_=>return Ok(None)
    };
    let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
    let ids:Vec<_>=encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
    let step_members:Vec<_>=w.party.iter().enumerate().filter(|(_,id)|w.character_progression[id].jobs.equipped_support.contains(crate::gravell_setup::STEP_AHEAD)).map(|(i,_)|i).collect();
    let menu=if !step_members.is_empty() {FieldEscapeMenu::with_step_ahead_members(rng,situation,&players,&levels,&ids,&w.enemy_knowledge,if step_members.len()==players.len(){None}else{Some(step_members)})?}
        else {FieldEscapeMenu::with_players(rng,situation,&players,&levels,&ids,&w.enemy_knowledge)?};
    let mut menu=menu;let mut defends=vec![];let mut defend_rounds=vec![];
    ensure(count<=7,"At most seven Defends before Escape")?;
    for _ in 0..count {let round=menu.round();let Some((next,report))=menu.safe_step_ahead_defend_through_next_round()?else{return Ok(None);};defend_rounds.push(round);defends.push(report);menu=next;}
    let escape_round=menu.round();
    let escape=menu.clone().attempt(&[],encounter.encounter_level.ok_or_else(||SimError(format!("Missing {} encounter danger level",w.map)))?)?;
    ensure(escape.chance_percent==100&&escape.success,format!("{} Escape is not guaranteed with the carried levels",w.map))?;
    let ret=crate::highroad::restore_field(w,&escape,&w.map,&format!("{} T{escape_round} Escape after {count} Defends",w.map))?;
    let field_return=Entry {arrival:ret.arrival,after:ret.after,world_after:ret.world_after,trace:ret.trace,decisions:vec![]};
    Ok(Some(SouthernSai {arrival,encounter,battle_entry:rng,menu,escape,field_return,defends,defend_rounds,escape_round}))
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
    // Head Cook's Critical Nut (L) is deferred until after the Stormhail book,
    // outside the manipulated route. Continue straight to the coast here.
    let conning_night=if night_pair {
        l.manual_time_switch(4,"Conning Creek: switch to night immediately on arrival")?;
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
    let Some(eastern_sai_encounter)=sai_encounter(eastern_sai.clone(),"ENT_FLD_DST_2_4_LND")? else {return Ok(None);};
    let Some(southern_sai)=southern_sai(&eastern_sai_encounter.field_return)? else {return Ok(None);};
    let mut l=Loader::new(world::catalog(),southern_sai.field_return.world_after.clone(),southern_sai.field_return.after)?;
    // User specifies exactly one encounter in each Sai field. No unmeasured
    // post-escape traversal budget or extra encounter is fabricated.
    l.visit_map("Twn_Dst_2_1_A")?;
    l.skipped_event("MapInStorySelect")?;
    let sai=point(&mut l,southern_sai.field_return.after);
    Ok(Some((town,Continuation {day_before_gravell,conning,conning_night,coast_entry,coast_day,movement,coast_exit,eastern_sai,eastern_sai_encounter,southern_sai,sai,oresrush_supplies:None})))
}
fn has_finisher_control(fight:&Fight)->bool {
    fight.actions.last().is_some_and(|a|a.action==Action::ThunderSoulstone
        &&fight.state.actors[a.actor.0].resources.bp>0)
}
fn unadjusted_travel(field_return:&Entry)->Result<Option<(Entry,Continuation)>> {
    let mut best:Option<(Entry,Continuation)>=None;
    for pair in [false,true] {
        if let Some(candidate)=travel(field_return,pair)? {
            if best.as_ref().is_none_or(|b|candidate.1.movement.run_percent>b.1.movement.run_percent) {best=Some(candidate);}
        }
    }
    Ok(best)
}
/// A BG victory has no later item menu to adjust. A Soulstone user with no BP
/// likewise cannot boost. Check both existing free travel controls while the
/// EXP search can still reject this candidate, rather than failing afterward.
pub(crate) fn accepts_fight(night:&Entry,battle_entry:CrtRng,fight:&Fight)->Result<bool> {
    if has_finisher_control(fight) {return Ok(true);}
    let (_,_,field_return)=settle_gravell(night,battle_entry,fight.clone())?;
    Ok(unadjusted_travel(&field_return)?.is_some())
}
pub(crate) fn extend(g:&mut GravellArrival)->Result<()> {
    extend_to_sai(g)?;
    let coast=g.coast.as_mut().unwrap();
    coast.oresrush_supplies=Some(crate::galdera_oresrush::finish(&coast.sai)?);
    Ok(())
}
fn extend_to_sai(g:&mut GravellArrival)->Result<()> {
    if !has_finisher_control(&g.fight) {
        let (town,coast)=unadjusted_travel(&g.field_return)?.ok_or_else(||SimError("EXP100 victory has no adjustable finisher and no safe coast continuation".into()))?;
        g.town=town;g.coast=Some(coast);
        return Ok(());
    }
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
    Err(SimError("No safe coast / non-ambushed Southern Sai continuation with the available finisher and Conning Creek controls".into()))
}

#[cfg(test)] mod tests {
    use super::*;
    #[test]
    fn bg_victory_3ed34a23_continues_without_a_phantom_soulstone_or_extra_rewards() {
        let f:serde_json::Value=serde_json::from_str(include_str!("../data/source/gravell-bg-victory-3ed34a23.json")).unwrap();
        let rng:CrtRng=serde_json::from_value(f["night"]["after"].clone()).unwrap();
        let night=Entry {arrival:rng,after:rng,world_after:serde_json::from_value(f["night"]["world_after"].clone()).unwrap(),trace:vec![],decisions:vec![]};
        let mut battle_entry=night.after;
        let w=&night.world_after;
        let encounter=encounter::catalog().predict(&mut battle_entry,&PredictionOptions {
            map:w.map.clone(),zone:"ENT_FLD_WLD_3_2_LND".into(),period:"Night".into(),
            progress:w.main_progress_points,first_encounter:w.first_encounter,
            boundary:Boundary::CurrentCounter,current_counter:Some(w.encounter_counter),
            party_levels:w.party.iter().map(|id|w.character_progression[id].level).collect(),
            ability_percent:if w.evasive_maneuvers{50}else{100},dash:true,..Default::default()
        }).unwrap();
        assert_eq!(battle_entry,serde_json::from_value(f["battle_entry"].clone()).unwrap());
        assert_eq!(serde_json::to_value(&encounter.enemies).unwrap(),f["encounter"]["enemies"]);
        assert_eq!(serde_json::to_value(&encounter.checkpoints).unwrap(),f["encounter"]["checkpoints"]);
        let players:Vec<_>=night.world_after.party.iter().map(|id|night.world_after.character_progression[id].character.clone()).collect();
        let ids:Vec<_>=encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
        let situation=if encounter.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal};
        let battle=ot2_battle_engine::field_escape::carried_step_ahead(battle_entry,situation,&players,&ids,true,&night.world_after.enemy_knowledge).unwrap();
        let bg:Vec<u8>=serde_json::from_value(f["bg_inputs"].clone()).unwrap();
        let guards:Vec<u8>=serde_json::from_value(f["guard_inputs"].clone()).unwrap();
        let fight=gravell_routine::replay(battle,&bg,&guards).unwrap();
        assert_eq!(fight.actions.last().unwrap().action,Action::BewilderingGrace);
        assert_eq!(fight.evaluation,gravell_routine::Evaluation::Accepted);
        assert!(!has_finisher_control(&fight));
        assert!(accepts_fight(&night,battle_entry,&fight).unwrap());
        let (fight,rewards,field_return)=settle_gravell(&night,battle_entry,fight).unwrap();
        assert_eq!(field_return.after,serde_json::from_value(f["expected"]["field_return"].clone()).unwrap());
        let before=serde_json::to_value((&fight,&rewards,&field_return)).unwrap();
        let mut g=GravellArrival {license:night.clone(),setup:night.clone(),arrival:night.clone(),town:night.clone(),night,encounter,battle_entry,fight,rewards,field_return,coast:None};
        extend_to_sai(&mut g).unwrap();
        assert_eq!(serde_json::to_value((&g.fight,&g.rewards,&g.field_return)).unwrap(),before);
        let c=g.coast.as_ref().unwrap();
        assert_eq!(c.movement.run_percent,79.0);
        assert!(c.conning_night.is_none());
        assert!(c.movement.estimated_counter_at_exit>0);
        assert!(c.eastern_sai_encounter.escape.success&&c.southern_sai.escape.success);
        assert_ne!(c.eastern_sai_encounter.encounter.situation.as_deref(),Some("Surprise"));
        assert_ne!(c.southern_sai.encounter.situation.as_deref(),Some("Surprise"));
    }
    #[test]
    fn second_conning_visit_goes_straight_to_coast_without_critical_nut() {
        let f:serde_json::Value=serde_json::from_str(include_str!("../data/source/gravell-complete-route-fixture.json")).unwrap();
        let mut w:world::WorldState=serde_json::from_value(f["before"]["world_after"].clone()).unwrap();
        w.award_victory(ot2_battle_engine::progression::Reward {exp:300_000,jp:0,money:0}).unwrap();
        w.map="Fld_Wld_3_2".into();w.time=4;
        let count=w.route_items.get("ITM_CSM_0730").copied().unwrap_or(0);
        for night in [false,true] {
            let mut accepted=false;
            for seed in 0u32..128 {
                let rng=CrtRng::new(seed.wrapping_mul(1775700753));
                let e=Entry {arrival:rng,after:rng,world_after:w.clone(),trace:vec![],decisions:vec![]};
                if let Some((_,c))=travel(&e,night).unwrap() {
                    assert_eq!(c.sai.world_after.route_items.get("ITM_CSM_0730").copied().unwrap_or(0),count);
                    let mut expected=Loader::new(world::catalog(),c.conning.world_after.clone(),c.conning.after).unwrap();
                    if night {
                        let n=c.conning_night.as_ref().unwrap();
                        assert_eq!(n.arrival,c.conning.after);
                        expected.manual_time_switch(4,"test: night on arrival").unwrap();
                        assert_eq!(n.after,expected.rng);
                    }else{assert!(c.conning_night.is_none());}
                    assert_eq!(c.coast_entry.arrival,expected.rng);
                    expected.visit_map(crate::conning::WEST).unwrap();
                    assert_eq!(c.coast_entry.after,expected.rng);
                    assert_eq!(c.coast_entry.world_after.encounter_counter,expected.state.encounter_counter);
                    assert!(serde_json::to_value(&c).unwrap().get("head_cook_steal").is_none());
                    accepted=true;break;
                }
            }
            assert!(accepted,"No accepted coast continuation for night={night}");
        }
    }
    #[test]
    fn southern_sai_rejects_surprise_and_escapes_with_actual_carried_levels() {
        let f:serde_json::Value=serde_json::from_str(include_str!("../data/source/gravell-complete-route-fixture.json")).unwrap();
        let mut state:world::WorldState=serde_json::from_value(f["before"]["world_after"].clone()).unwrap();
        state.award_victory(ot2_battle_engine::progression::Reward {exp:300_000,jp:0,money:0}).unwrap();
        state.map="Fld_Dst_2_4".into();state.time=2;
        let mut rejected=false;let mut accepted=false;
        for seed in 0u32..1024 {
            let rng=CrtRng::new(seed.wrapping_mul(1775700753));
            let e=Entry {arrival:rng,after:rng,world_after:state.clone(),trace:vec![],decisions:vec![]};
            match southern_sai(&e).unwrap() {
                None=>rejected=true,
                Some(s)=>{
                    accepted=true;assert_ne!(s.encounter.situation.as_deref(),Some("Surprise"));
                    assert_eq!(s.escape.chance_percent,100);assert!(s.escape.success);
                    assert_eq!(s.field_return.world_after.map,"Fld_Dst_2_3");
                    assert!(s.field_return.world_after.enemy_knowledge.len()>=state.enemy_knowledge.len());
                }
            }
            if accepted&&rejected {return;}
        }
        panic!("Both normal and rejected ambush candidates must be exercised");
    }
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
