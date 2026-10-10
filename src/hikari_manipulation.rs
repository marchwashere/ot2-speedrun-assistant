//! Joint Hikari second-menu / Canalbrine water first-menu manipulation.
//! Project each immutable Hikari continuation once, then visit total input-cost
//! layers. Neither an earlier route plan nor a rejected candidate is mutated.
use crate::{Result,SimError,canalbrine,escape_search::FieldSearch,hikari,
    manipulation::{BoostInputs,describe_inputs},scholar,snows_escape::Entry};
use ot2_battle_engine::escape::Situation;
use serde::Serialize;

#[derive(Debug,Serialize)]
pub struct Search {
    pub inputs:BoostInputs,
    pub tested_hikari_candidates:u64,
    pub tested_escape_attempts:u64,
    pub total_input_cost:u32,
    pub status:&'static str,
}
pub struct Continuation {
    pub hikari:Box<hikari::Prediction>,
    pub scholar:Box<scholar::Prediction>,
    pub canalbrine:Box<canalbrine::Prediction>,
    pub search:Search,
}
struct Candidate {
    hikari:Box<hikari::Prediction>,
    scholar:Box<scholar::Prediction>,
    canalbrine:Box<canalbrine::Prediction>,
    boarding:Entry,
    encounter:crate::encounter::Prediction,
    battle_entry:crate::CrtRng,
    escape:FieldSearch,
}
fn path(repeats:u32)->Vec<u8> {(0..repeats).flat_map(|_|[1,0]).collect()}

/// Commit each closest control independently, without cross-menu allocations.
pub fn nearest_coasts(town:&Entry)->Result<Continuation> {
    nearest_coasts_bounded(town,None)?.ok_or_else(||SimError("Canalbrine boost search exhausted".into()))
}
pub fn nearest_coasts_bounded(town:&Entry,limit:Option<u32>)->Result<Option<Continuation>> {
    let mut h=0u32;
    let mut c=loop {
        if limit.is_some_and(|l|2*h>l) {return Ok(None);}
        if let Some(c)=project(town,h)? {break c;}
        h=h.checked_add(1).ok_or_else(||SimError("Hikari boost search exhausted".into()))?;
    };
    let mut tested=0;
    let mut fallback_tried=false;
    let mut best:Option<(u32,crate::highroad::Escape,crate::highroad::FieldReturn,crate::conning::Prediction)>=None;
    for cost in (0..u32::MAX-1).step_by(2) {
        // A later northern target cannot improve even before coast inputs.
        if best.as_ref().is_some_and(|(total,_,_,_)|2*h+cost>*total) {break;}
        if limit.is_some_and(|l|2*h+cost>l) {break;}
        tested+=1;
        let Some(escape)=c.escape.candidates(cost,false,None)?.into_iter().next() else {continue;};
        let field_return=crate::highroad::restore_field(&c.boarding.world_after,escape.attempt.as_ref().unwrap(),canalbrine::BRIDGE,"Canalbrine Bridge water Escape")?;
        let north=crate::conning::arrival(&field_return.world_after,field_return.after,crate::conning::NORTH)?;
        if !matches!(north.encounter.situation.as_deref(),Some("Normal"|"Preemptive")) {continue;}
        let has_cait=north.encounter.enemies.iter().any(|s|s.enemy.id=="ENE_NML_OTR_140");
        // One propagated western fallback. Continue the closest upstream menu
        // specifically for northern Cait, rather than freezing its formation.
        if fallback_tried&&!has_cait {continue;}
        fallback_tried=true;
        let cap=best.as_ref().map(|b|b.0).into_iter().chain(limit).min().map(|l|l-2*h-cost);
        let Some(conning)=crate::conning::nearest_bounded(north,cap)? else {continue;};
        let total=2*h+cost+conning.input_cost;
        if best.as_ref().is_none_or(|(old,_,_,p)|total<*old||(total==*old&&has_cait&&p.northern.victory.is_none())) {
            best=Some((total,escape,field_return,conning));
        }
    }
    if let Some((total,escape,field_return,conning))=best {
        c.canalbrine.water=Some(Box::new(canalbrine::Water {boarding:c.boarding,encounter:c.encounter,battle_entry:c.battle_entry,escape,field_return}));
        c.canalbrine.conning=Some(Box::new(conning));
        c.canalbrine.verification="Strict nearest-menu propagation for the selected Oresrush switch allocation. Safe openings, Cultured Cait clear and first Escapes preserved. Awaiting retail validation.";
        return Ok(Some(Continuation {hikari:c.hikari,scholar:c.scholar,canalbrine:c.canalbrine,
            search:Search {inputs:describe_inputs(h,0,path(h))?,tested_hikari_candidates:u64::from(h)+1,
                tested_escape_attempts:tested,total_input_cost:total,status:"Compare northern Cait via Canalbrine boosts against the propagated western fallback; choose lower complete boost cost, preferring Northern on ties"}}));
    }
    Ok(None)
}

/// Cached prefixes for the extended coast search. Query each total cost layer
/// in ascending order; rejected states never mutate the carried route.
pub struct CoastSearch {candidates:Vec<Option<Candidate>>,tested:u64}
impl CoastSearch {
    pub fn new()->Self {Self {candidates:vec![],tested:0}}
    pub fn at_cost(&mut self,town:&Entry,cost:u32,cache:&mut crate::conning::Cache)->Result<Option<Continuation>> {
        if cost%2!=0 {return Ok(None);}
        for h in 0..=cost/2 {
            if self.candidates.len()==h as usize {self.candidates.push(project(town,h)?);}
            let Some(c)=self.candidates[h as usize].as_mut() else {continue;};
            for water_cost in (0..=cost-2*h).step_by(2) {
                self.tested+=1;
                let Some(escape)=c.escape.candidates(water_cost,false,None)?.into_iter().next() else {continue;};
                let field_return=crate::highroad::restore_field(&c.boarding.world_after,escape.attempt.as_ref().unwrap(),canalbrine::BRIDGE,"Canalbrine Bridge water Escape")?;
                let Some(conning)=cache.at_cost(&field_return.world_after,field_return.after,cost-2*h-water_cost)? else {continue;};
                let mut c=self.candidates[h as usize].take().unwrap();
                c.canalbrine.water=Some(Box::new(canalbrine::Water {boarding:c.boarding,encounter:c.encounter,battle_entry:c.battle_entry,escape,field_return}));
                c.canalbrine.conning=Some(Box::new(conning));
                c.canalbrine.verification="Complete source-derived coast continuation; minimum total boost inputs within this prefix, one Cultured Cait clear and safe first Escapes. Awaiting retail validation.";
                return Ok(Some(Continuation {hikari:c.hikari,scholar:c.scholar,canalbrine:c.canalbrine,
                    search:Search {inputs:describe_inputs(h,0,path(h))?,tested_hikari_candidates:self.candidates.len() as u64,
                        tested_escape_attempts:self.tested,total_input_cost:cost,status:"Joint Hikari / Canalbrine / Conning coast search: Cultured Cait, no surprise, successful first Escapes"}}));
            }
        }
        Ok(None)
    }
}

fn project(town:&Entry,repeats:u32)->Result<Option<Candidate>> {
    let hikari=Box::new(hikari::predict_with_boost_inputs(town,&path(repeats))?);
    let scholar=Box::new(scholar::predict(&hikari.recruitment)?);
    let canalbrine=Box::new(canalbrine::predict(&scholar.after)?);
    let (boarding,encounter,battle_entry)=canalbrine::water_arrival(&canalbrine.bridge_entry)?;
    let situation=match encounter.situation.as_deref() {
        Some("Normal")=>Situation::Normal,
        Some("Preemptive")=>Situation::Preemptive,
        Some("Surprise")=>return Ok(None),
        other=>return Err(SimError(format!("Missing Canalbrine water opening: {other:?}"))),
    };
    let w=&boarding.world_after;
    let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
    let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
    let enemies:Vec<_>=encounter.enemies.iter().map(|slot|slot.enemy.id.as_str()).collect();
    let danger=encounter.encounter_level.ok_or_else(||SimError("Missing Canalbrine water danger level".into()))?;
    let escape=FieldSearch::with_players(battle_entry,situation,&players,&levels,&enemies,danger,&w.enemy_knowledge)?;
    Ok(Some(Candidate {hikari,scholar,canalbrine,boarding,encounter,battle_entry,escape}))
}

/// Both menus start with one available BP, so a cancel cycle is the only legal
/// RNG-changing path ending at zero BP: two inputs and one CRT draw. Search the
/// sum of both menus' cycles; ties prefer fewer Hikari cycles. No retry cap.
pub fn predict(town:&Entry)->Result<Continuation> {
    let mut candidates:Vec<Option<Candidate>>=vec![];
    let mut tested_escape_attempts=0u64;
    for total in 0..=u32::MAX/2 {
        for h in 0..=total {
            if candidates.len()==h as usize {candidates.push(project(town,h)?);}
            let Some(candidate)=candidates[h as usize].as_mut() else {continue;};
            let cost=2*(total-h);
            tested_escape_attempts+=1;
            let Some(mut escape)=candidate.escape.candidates(cost,false,None)?.into_iter().next() else {continue;};
            escape.tested_candidates=tested_escape_attempts;
            let attempt=escape.attempt.as_ref().expect("successful Escape candidate");
            let field_return=crate::highroad::restore_field(&candidate.boarding.world_after,attempt,
                canalbrine::BRIDGE,"Canalbrine Bridge water Escape")?;
            let mut chosen=candidates[h as usize].take().unwrap();
            chosen.canalbrine.water=Some(Box::new(canalbrine::Water {
                boarding:chosen.boarding,encounter:chosen.encounter,
                battle_entry:chosen.battle_entry,escape,field_return,
            }));
            chosen.canalbrine.verification="Source-derived shared map loads and daytime Canalbrine Bridge water encounter with the carried four-character party, actual levels, gear, Scholar jobs, active Evasive Maneuvers and enemy knowledge. First canoe tutorial and boarding add no CRT draws or counter reset. Hikari's second command and the first water Escape menu are searched jointly by total input cost; every selected opening is Normal or Preemptive and the first Escape succeeds. Unmeasured water movement is not inferred. Awaiting retail validation.";
            return Ok(Continuation {hikari:chosen.hikari,scholar:chosen.scholar,canalbrine:chosen.canalbrine,
                search:Search {inputs:describe_inputs(h,0,path(h))?,
                    tested_hikari_candidates:candidates.len() as u64,tested_escape_attempts,
                    total_input_cost:2*total,
                    status:"Minimum total boost inputs at Hikari's second menu and the first water Escape menu; no surprise and first Escape succeeds"}});
        }
    }
    Err(SimError("Canalbrine input cost exceeds the representable u32 range".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn town()->Entry {
        let mut w=crate::world::throne_after_second_state().unwrap();
        w.map=crate::ryu::TOWN.into();w.time=2;w.party=vec![7,6,8,4];
        w.party_experience=vec![1530,1467,0,0];w.job_licenses.insert(6,1);
        w.active_scenarios.clear();w.story_types.clear();w.prologues.clear();
        w.story_types.insert(7);w.active_scenarios.insert(701);
        w.main_progress_points=4;w.completed_scenarios.extend([700,600,601,800,400]);
        for flag in [36,35,37,33,20509,8019,8050,30510] {w.set_flag(flag,true);}
        w.visited_maps.extend([crate::ryu::TOWN.into(),crate::ryu::NORTHERN.into()]);
        w.resident_levels=crate::world::catalog().maps[&w.map].streams.iter().chain(w.shared_levels.iter()).cloned().collect();
        w.sync_progression().unwrap();w.character_progression.get_mut(&7).unwrap().jp_earned=517;
        let osvald=w.character_progression.get_mut(&6).unwrap();
        osvald.jp_earned=492;osvald.jp_spent=130;
        osvald.jobs.unlocked_support.insert(crate::scholar::EVASIVE.into());
        osvald.jobs.equipped_support.insert(crate::scholar::EVASIVE.into());
        w.refresh_support_skills();w.encounter_counter=749;
        let mut rng=crate::CrtRng::new(0);rng.advance(7000);
        Entry {arrival:rng,after:rng,world_after:w,trace:vec![],decisions:vec![]}
    }
    #[test]
    fn northern_cait_is_searched_past_first_safe_opening() {
        let mut town=town();let mut improved=false;
        for _ in 0..32 {
            let result=nearest_coasts(&town).unwrap();
            let limit=result.search.total_input_cost;
            let bounded=nearest_coasts_bounded(&town,Some(limit)).unwrap().unwrap();
            assert_eq!(bounded.canalbrine.conning.as_ref().unwrap().town.after,result.canalbrine.conning.as_ref().unwrap().town.after);
            assert_eq!(bounded.search.inputs.levels,result.search.inputs.levels);
            if limit>0 {assert!(nearest_coasts_bounded(&town,Some(limit-1)).unwrap().is_none());}
            let h=result.search.inputs.extra_repeats;
            let mut c=project(&town,h).unwrap().unwrap();
            let chosen=result.canalbrine.conning.as_ref().unwrap();
            let mut first_safe=None;
            // Independent replay of every Canalbrine candidate that can beat
            // the winner, including equal-cost northern preference.
            for cost in (0..=result.search.total_input_cost-2*h).step_by(2) {
                let Some(escape)=c.escape.candidates(cost,false,None).unwrap().into_iter().next() else {continue;};
                let returned=crate::highroad::restore_field(&c.boarding.world_after,escape.attempt.as_ref().unwrap(),canalbrine::BRIDGE,"test").unwrap();
                let north=crate::conning::arrival(&returned.world_after,returned.after,crate::conning::NORTH).unwrap();
                if !matches!(north.encounter.situation.as_deref(),Some("Normal"|"Preemptive")) {continue;}
                let cat=north.encounter.enemies.iter().any(|s|s.enemy.id=="ENE_NML_OTR_140");
                first_safe.get_or_insert((cost,cat));
                if !cat {continue;}
                let plan=crate::conning::nearest(north).unwrap();
                let total=2*h+cost+plan.input_cost;
                assert!(total>=result.search.total_input_cost);
                if total==result.search.total_input_cost {assert!(chosen.northern.victory.is_some());}
            }
            if chosen.northern.victory.is_some()&&first_safe.is_some_and(|(_,cat)|!cat) {improved=true;break;}
            town.after.advance(1);town.arrival=town.after;
        }
        assert!(improved,"Must exercise deliberate northern targeting, not only natural Cait");
    }
    #[test]
    fn joint_cost_is_minimal_and_complete_winner_replays_without_mutating_upstream() {
        let mut town=town();
        // Exercise a nontrivial Hikari allocation, not a vacuous cost-zero plan.
        let mut selected=None;
        for _ in 0..64 {
            let p=predict(&town).unwrap();
            if p.search.inputs.extra_repeats>0 {selected=Some(p);break;}
            town.after.advance(1);town.arrival=town.after;
        }
        let p=selected.expect("Fixture range must include a surprise requiring Hikari adjustment");
        let immutable=crate::state_key::WorldKey::new(&town.world_after);
        let water=p.canalbrine.water.as_ref().unwrap();
        assert!(matches!(water.encounter.situation.as_deref(),Some("Normal"|"Preemptive")));
        assert!(water.escape.attempt.as_ref().unwrap().success);
        let h=p.search.inputs.extra_repeats;
        assert_eq!(p.search.total_input_cost,2*h+water.escape.weighted_input_cost);
        // Exhaust every cheaper allocation using fresh, independently replayed
        // predictions rather than the cost-layer cache or mutable boost prefix.
        for total in 0..p.search.total_input_cost/2 {
            for before in 0..=total {
                if let Some(c)=project(&town,before).unwrap() {
                    let attempt=c.escape.menu.clone().attempt(&path(total-before),
                        c.encounter.encounter_level.unwrap()).unwrap();
                    assert!(!attempt.success,"A cheaper complete plan exists");
                }
            }
        }
        let c=project(&town,h).unwrap().unwrap();
        assert_eq!(serde_json::to_value(&c.hikari).unwrap(),serde_json::to_value(&p.hikari).unwrap());
        assert_eq!(c.battle_entry,water.battle_entry);
        let attempt=c.escape.menu.clone().attempt(&water.escape.inputs.as_ref().unwrap().levels,
            water.encounter.encounter_level.unwrap()).unwrap();
        assert_eq!(serde_json::to_value(&attempt).unwrap(),serde_json::to_value(&water.escape.attempt).unwrap());
        assert_eq!(water.boarding.arrival,water.boarding.after,"Canoe/tutorial has no CRT");
        assert_eq!(water.field_return.after.draws,attempt.after.draws+2,"Real source B/A reset");
        assert!(!water.field_return.world_after.first_encounter);
        assert_eq!(water.field_return.world_after.party,[7,1,8,4]);
        assert_eq!(water.field_return.world_after.bench,[6]);
        assert_eq!(serde_json::to_value(&water.field_return.world_after.character_progression).unwrap(),
            serde_json::to_value(&water.boarding.world_after.character_progression).unwrap(),"Escape awards no EXP/JP");
        assert_eq!(crate::state_key::WorldKey::new(&town.world_after),immutable);
    }
}
