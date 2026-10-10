//! Ryu's scripted recruit battle, field return, chapter skip and full-party swap.
//! Battles belong to the standalone core; field/event callbacks use shared Loader.
use crate::{CrtRng,Result,ensure,snows_escape::Entry,world::{self,Loader,CharacterProgression}};
use ot2_battle_engine::{Outcome,hikari_recruitment,progression::{self,Reward}};
use serde::Serialize;

pub const INTRO:&str="MS_KEN_00_0100";
pub const VICTORY:&str="MS_KEN_00_0300";

#[derive(Debug,Serialize)]
pub struct Prediction {
    pub introduction:Entry,
    pub request:world::ScriptedBattle,
    pub battle:hikari_recruitment::Prediction,
    pub reward:Reward,
    pub field_return:Entry,
    pub recruitment:Entry,
    pub verification:&'static str,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entry()->Entry {
        let mut w=world::throne_after_second_state().unwrap();
        w.map=crate::ryu::TOWN.into();w.time=2;w.party=vec![7,6,8,4];w.bench.clear();
        w.party_experience=vec![1530,1467,0,0];w.evasive_maneuvers=true;w.osvald_extra_skills=2;
        w.story_types.clear();w.active_scenarios.clear();w.prologues.clear();
        w.story_types.insert(7);w.active_scenarios.insert(701);w.confirmed_scenarios.insert(701);
        w.main_progress_points=4;w.completed_scenarios.extend([700,600,601,800,400]);
        for flag in [36,35,37,33] {w.set_flag(flag,true);}
        w.resident_levels=world::catalog().maps[&w.map].streams.iter().chain(w.shared_levels.iter()).cloned().collect();
        w.encounter_counter=749;w.sync_progression().unwrap();
        w.character_progression.get_mut(&6).unwrap().jp_earned=487;
        w.character_progression.get_mut(&6).unwrap().jp_spent=130;
        let rng=CrtRng {state:0x39C23898,draws:1991};
        Entry {arrival:rng,after:rng,world_after:w,trace:vec![],decisions:vec![]}
    }
    #[test]
    fn scripted_boundary_does_not_run_declined_intro_cleanup() {
        let entry=entry();let mut l=Loader::new(world::catalog(),entry.world_after,entry.after).unwrap();
        let b=l.skipped_event_until_battle(INTRO,&world::EventChoices::from([
            ((INTRO.into(),47),"eDIALOG_YES".into()),
        ])).unwrap();
        assert_eq!((b.command_index,b.volume.as_str(),b.victory_event.as_str()),(66,"ENM_EVE_WAR_C00_010",VICTORY));
        assert_eq!(l.rng,entry.after,"Scripted encounter lookup/fixed skipped intro has no CRT lottery");
        assert!(l.state.flag(37501));
        assert_eq!(l.state.party,[7,6,8,4]);
        assert!(l.trace.iter().all(|s|s.kind==world::DrawKind::Boundary));
    }
    #[test]
    fn victory_rewards_only_participants_then_skips_and_swaps_with_bench_ledger_intact() {
        let entry=entry();let old=crate::state_key::WorldKey::new(&entry.world_after);
        let p=predict(&entry).unwrap();
        assert_eq!(crate::state_key::WorldKey::new(&entry.world_after),old);
        assert_eq!(p.battle.outcome,Outcome::Victory);assert_eq!(p.battle.menus.len(),2);
        let mut cursor=entry.after;
        for s in &p.introduction.trace {assert_eq!(s.before,cursor);cursor=s.after;}
        for s in &p.battle.trace {assert_eq!(s.before,cursor);cursor=s.after;}
        assert_eq!(cursor,p.battle.after);
        for stage in [&p.field_return,&p.recruitment] {
            assert_eq!(stage.arrival,cursor);
            for s in &stage.trace {assert_eq!(s.before,cursor);cursor=s.after;}
            assert_eq!(cursor,stage.after);
        }
        let final_state=&p.recruitment.world_after;
        assert_eq!(final_state.party,[7,1,8,4]);assert_eq!(final_state.bench,[6]);
        assert!(!final_state.evasive_maneuvers);assert_eq!(final_state.osvald_extra_skills,2);
        assert_eq!(final_state.main_progress_points,5);
        assert!(final_state.completed_scenarios.contains(&100));
        assert!(final_state.active_scenarios.contains(&701));
        let hikari=&final_state.character_progression[&1];
        assert_eq!((hikari.level,hikari.experience,hikari.jp_earned),(1,0,0));
        for &id in &[7,6,8,4] {
            let before=&entry.world_after.character_progression[&id];
            let after=&final_state.character_progression[&id];
            assert_eq!(after.experience,before.experience+p.reward.exp);
            assert_eq!(after.jp_earned,before.jp_earned+p.reward.jp);
            assert_eq!(after.jp_spent,before.jp_spent);
            let after_combat=after.character.combat.as_ref().unwrap();
            let before_combat=before.character.combat.as_ref().unwrap();
            assert_eq!(after_combat.main_weapon,before_combat.main_weapon);
            assert_eq!(serde_json::to_value(&after_combat.weapons).unwrap(),serde_json::to_value(&before_combat.weapons).unwrap());
        }
        assert_eq!(p.field_return.after.draws,p.battle.after.draws+2);
        let resets:Vec<_>=p.recruitment.trace.iter().filter(|s|s.kind==world::DrawKind::Counter).collect();
        assert_eq!(resets.len(),2,"Recruitment's FinishPre is a separate B/A reset");
        assert!(resets.iter().all(|s|s.reason.starts_with("MS_KEN_00_0300 FinishPre")));
        assert_eq!(final_state.party_experience,final_state.party.iter().map(|id|final_state.character_progression[id].experience).collect::<Vec<_>>());
    }
}
fn checkpoint(l:&mut Loader<'_>,arrival:CrtRng,reason:&str)->Entry {
    l.boundary(reason);
    Entry {arrival,after:l.rng,world_after:l.state.clone(),
        trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
pub fn predict(entry:&Entry)->Result<Prediction> {
    predict_with_boost_inputs(entry,&[])
}
/// Cancel-only inputs at the second player's command menu; the action remains
/// its unboosted default weapon. The core validates actual available BP.
pub fn predict_with_boost_inputs(entry:&Entry,inputs:&[u8])->Result<Prediction> {
    ensure(entry.world_after.map==crate::ryu::TOWN && entry.world_after.time==2,
        "Hikari recruitment requires daytime Ryu field control")?;
    ensure(entry.world_after.party==[7,6,8,4] && entry.world_after.bench.is_empty(),
        "Hikari recruitment requires the carried four-character party before the swap")?;
    ensure([36,35,37,33].iter().all(|&flag|entry.world_after.flag(flag)),
        "Hikari's carried participants require their completed/skipped Chapter 1 latent unlocks")?;
    let mut w=entry.world_after.clone();w.sync_progression()?;
    let mut l=Loader::new(world::catalog(),w,entry.after)?;
    let request=l.skipped_event_until_battle(INTRO,&world::EventChoices::from([
        ((INTRO.into(),47),"eDIALOG_YES".into()),
    ]))?;
    ensure(request.volume=="ENM_EVE_WAR_C00_010" && request.victory_event==VICTORY,
        "Hikari script requested a different battle or continuation")?;
    let introduction=checkpoint(&mut l,entry.after,"Hikari recruit battle: before initialization");
    let players:Vec<_>=l.state.party.iter().map(|id|l.state.character_progression[id].character.clone()).collect();
    let battle=hikari_recruitment::predict_with_boost_inputs(l.rng,&players,&l.state.enemy_knowledge,&[],inputs)?;
    ensure(battle.outcome==Outcome::Victory,"Hikari's supplied two-command routine did not win")?;
    ensure(battle.party_resources_after.len()==l.state.party.len(),"Hikari battle returned an incomplete party")?;
    for (&id,resources) in l.state.party.iter().zip(&battle.party_resources_after) {
        l.state.character_progression.get_mut(&id).unwrap().character.resources=Some(resources.clone());
    }
    l.state.enemy_knowledge=battle.enemy_knowledge.clone();
    let reward=battle.rewards;
    // Only the four participants receive rewards. Hikari is not yet a member.
    l.state.award_victory(reward)?;
    l.rng=battle.after;
    l.restore_field_after_battle("Hikari recruit victory: field returned before recruitment event")?;
    let field_return=checkpoint(&mut l,battle.after,"Hikari recruit battle: field restored and rewards applied");
    let character=ot2_battle_engine::field_escape::route_player(1,0)?;
    let growth=progression::growth(1,0)?.clone();
    l.state.character_progression.insert(1,CharacterProgression {
        experience:0,jp_earned:0,jp_spent:0,level:growth.level,growth,character,jobs:Default::default(),
    });
    l.skipped_event_with_choices(VICTORY,&world::EventChoices::from([
        ((VICTORY.into(),127),"eDIALOG_YES".into()),
        ((VICTORY.into(),138),"eDIALOG_NO".into()),
        ((VICTORY.into(),148),"6".into()),
    ]))?;
    ensure(l.state.party==[7,1,8,4] && l.state.bench==[6]
        && l.state.completed_scenarios.contains(&100) && !l.state.evasive_maneuvers,
        "Hikari chapter skip or Osvald replacement did not finish")?;
    let recruitment=checkpoint(&mut l,field_return.after,"Hikari recruited: Chapter 1 skipped; Osvald replaced; tutorials dismissed");
    Ok(Prediction {introduction,request,battle,reward,field_return,recruitment,
        verification:"Source-derived Hikari scripted battle with carried original equipment, resources, enemy knowledge and EXP/JP. The supplied two-command routine uses a regular Ice Soulstone followed by the next actor's unboosted default weapon. Victory field-return reset precedes the recruitment event's separate FinishPre reset. Chapter 1 is skipped; Hikari joins at level 1 and replaces Osvald, whose ledger stays on the bench. Evasive Maneuvers becomes inactive. Awaiting retail validation."})
}
