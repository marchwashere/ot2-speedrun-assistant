//! Route stops at the first successful rare kill; later rare targets are alternatives.
use crate::{CrtRng,Result,encounter,ensure,world::{self,Loader,WorldState,Trace,Decision}};
use ot2_battle_engine::{escape::Situation,field_escape::FieldRareVictory};
use serde::Serialize;

pub fn target_met(p:&encounter::Prediction)->bool {
    let cape_cold = matches!(p.map.as_str(), "Fld_Snw_1_2" | "Fld_Snw_1_3");
    matches!(p.situation.as_deref(),Some("Normal"|"Preemptive"))
        && p.enemies.iter().any(|slot| encounter::catalog().rare_labels.iter().any(|(family,ids)| {
            ids.iter().enumerate().any(|(tier,id)| id==&slot.enemy.id
                && !(cape_cold && family.starts_with("OCTOPUS_") && tier==0))
        }))
}

#[derive(Debug,Serialize)]
pub struct Victory {
    pub map:String,
    pub inputs:crate::manipulation::BoostInputs,
    pub battle:FieldRareVictory,
    pub after:CrtRng,
    pub world_after:WorldState,
    pub trace:Vec<Trace>,
    pub decisions:Vec<Decision>,
    pub learned_skills:u8,
    pub evasive_maneuvers:bool,
    pub party_levels_after:Vec<u32>,
}

pub fn kill(entry:CrtRng,state:&WorldState,level:u32,p:&encounter::Prediction)->Result<Option<Victory>> {
    kill_with_repeats(entry,state,level,p,0)
}

pub fn kill_with_repeats(entry:CrtRng,state:&WorldState,level:u32,p:&encounter::Prediction,repeats:u32)->Result<Option<Victory>> {
    if !target_met(p) {return Ok(None);}
    let situation=if p.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal};
    let ids:Vec<_>=p.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
    let mut state=state.clone();state.sync_progression()?;
    let levels:Vec<_>=state.party.iter().map(|id|state.character_progression[id].level).collect();
    ensure(levels.first()==Some(&level),"Rare battle level disagrees with carried experience")?;
    let players:Vec<_>=state.party.iter().map(|id|state.character_progression[id].character.clone()).collect();
    let inputs=crate::manipulation::optimal_inputs(repeats,1,0)?;
    let Some(battle)=ot2_battle_engine::field_escape::carried_shadow_soulstone_m(entry,situation,&players,&levels,&ids,state.time==4,&state.enemy_knowledge,&inputs.levels)? else {return Ok(None)};
    ensure(battle.base_jp>=130,"Rare fight cannot fund two learned skills")?;
    state.enemy_knowledge=battle.enemy_knowledge.clone();
    state.first_encounter=false;
    let mut l=Loader::new(world::catalog(),state,battle.after)?;
    l.restore_field_after_battle("Rare victory: field returned")?;
    // The one-action clear gets the source overkill bonus whenever a hit is
    // at least twice the victim's remaining HP (the same rule as earlier fights).
    let mut damage=std::collections::BTreeMap::new();
    let mut overkill=false;
    let mut broken=false;
    for event in &battle.report.events {
        use ot2_battle_engine::{Event,Resource};
        match *event {
            Event::Broken {actor,..} if actor.0>=2=>{broken=true;},
            Event::Damage {target,amount,..}=>{damage.insert(target,amount);},
            Event::ResourceChanged {actor,resource:Resource::Hp,before,after:0} if actor.0>=2 && before>0=>{
                overkill|=damage.get(&actor).is_some_and(|&n|u64::from(n)>=2*u64::from(before));
            },
            _=>{}
        }
    }
    let gained=battle.base_exp+(u32::from(overkill)+u32::from(broken))*(battle.base_exp/10).max(1);
    for (&id,resources) in l.state.party.iter().zip(&battle.party_resources_after) {
        l.state.character_progression.get_mut(&id).unwrap().character.resources=Some(resources.clone());
    }
    // CalcBonusReward: one-turn JP +10%, no-damage money +10%.
    l.state.award_victory(ot2_battle_engine::progression::Reward {exp:gained,jp:battle.base_jp+(battle.base_jp/10).max(1),money:battle.base_money+(battle.base_money/10).max(1)})?;
    if l.state.galdera_route {
        let osvald=l.state.character_progression.get_mut(&6).unwrap();
        ensure(osvald.jp_earned-osvald.jp_spent>=130,"Insufficient carried JP for Advanced Magic and Elemental Barrage")?;
        let learned=osvald.jobs.learned_abilities.entry(6).or_default();
        learned.extend(["ABI_SET_SCH_010","ABI_SET_SCH_020"].into_iter().map(str::to_owned));
        ensure(!learned.contains("ABI_SET_SCH_070") && !learned.contains("ABI_SET_SCH_060"),"Rare setup skills already purchased")?;
        learned.extend(["ABI_SET_SCH_070","ABI_SET_SCH_060"].into_iter().map(str::to_owned));
        osvald.jp_spent+=130;
        osvald.jobs.unlocked_support.insert("ABI_SCH_SUP_01".into());
    } else {
        let osvald=l.state.character_progression.get_mut(&6).unwrap();
        ensure(osvald.jp_earned-osvald.jp_spent>=130,"Insufficient carried JP for two Osvald skills")?;
        osvald.jp_spent+=130;
        osvald.jobs.unlocked_support.insert("ABI_SCH_SUP_01".into());
    }
    if !l.state.flag(world::catalog().tutorials[&54].opened_flag) {l.dismiss_tutorial(54)?;}
    l.state.set_flag(102,true);
    if !l.state.flag(world::catalog().tutorials[&57].opened_flag) {l.dismiss_tutorial(57)?;}
    l.state.set_flag(102,false);
    if !l.state.flag(world::catalog().tutorials[&56].opened_flag) {l.dismiss_tutorial(56)?;}
    l.state.character_progression.get_mut(&6).unwrap().jobs.equipped_support.insert("ABI_SCH_SUP_01".into());
    let party_levels_after=l.state.party.iter().map(|id|l.state.character_progression[id].level).collect();
    l.state.osvald_extra_skills=2;l.state.refresh_support_skills();
    l.boundary(if l.state.galdera_route {"Osvald learns Advanced Magic (30 JP), Elemental Barrage (100 JP), then equips Evasive Maneuvers; Throné learns nothing"} else {"Osvald learns two skills and equips Evasive Maneuvers: future movement ability multiplier 50%"});
    Ok(Some(Victory {inputs,map:l.state.map.clone(),battle,after:l.rng,world_after:l.state,trace:l.trace,decisions:l.decisions,learned_skills:2,evasive_maneuvers:true,party_levels_after}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encounter::{Boundary,PredictionOptions};
    #[test]
    fn accepts_actual_cait_and_equips_evasive_only_after_victory() {
        let mut w=world::throne_after_second_state().unwrap();
        w.map="Fld_Snw_1_2".into();w.time=2;w.party=vec![7,6];w.first_encounter=true;w.encounter_counter=700;
        w.active_scenarios.clear();w.story_types.clear();w.prologues.clear();
        w.party_experience=vec![ot2_battle_engine::field_escape::experience_for_level(5).unwrap(),0];
        w.resident_levels=world::catalog().maps[&w.map].streams.iter().cloned().collect();
        let mut covered=false;
        for seed in 0..1024 {
            let mut rng=CrtRng::new(seed);
            let p=encounter::catalog().predict(&mut rng,&PredictionOptions {map:w.map.clone(),zone:"ENT_FLD_SNW_1_2_LND".into(),period:"DayTime".into(),progress:1,first_encounter:true,boundary:Boundary::CurrentCounter,current_counter:Some(700),party_levels:vec![5,1],dash:true,..Default::default()}).unwrap();
            if !target_met(&p) {continue;}
            let v=kill(rng,&w,5,&p).unwrap().unwrap();
            assert_eq!(p.rare_family.as_deref(),Some("Cait"));
            let adjusted=kill_with_repeats(rng,&w,5,&p,3).unwrap().unwrap();
            assert_eq!(adjusted.inputs.levels,vec![1,0,1,0,1,0]);
            assert_eq!(adjusted.inputs.input_count,6);
            assert_eq!(adjusted.battle.at_command_menu,v.battle.at_command_menu);
            assert_eq!(adjusted.battle.base_jp,v.battle.base_jp);
            assert_ne!(adjusted.after,v.after);
            let replay=kill_with_repeats(rng,&w,5,&p,3).unwrap().unwrap();
            assert_eq!(serde_json::to_value(&adjusted).unwrap(),serde_json::to_value(replay).unwrap());
            covered=true;
            assert!(v.evasive_maneuvers && v.world_after.evasive_maneuvers);
            assert_eq!(v.world_after.osvald_extra_skills,2);
            assert!(v.world_after.flag(1353)&&v.world_after.flag(1355)&&!v.world_after.flag(102));
            let osvald=&v.world_after.character_progression[&6];
            assert_eq!(osvald.jp_spent,130);
            assert_eq!(osvald.jp_earned,v.battle.base_jp+(v.battle.base_jp/10).max(1));
            assert_eq!(osvald.level,ot2_battle_engine::progression::growth(6,osvald.experience).unwrap().level);
            assert_eq!(osvald.character.stats.max_hp,ot2_battle_engine::field_escape::route_player(6,osvald.experience).unwrap().stats.max_hp);
            assert!(!w.evasive_maneuvers,"Rejected/accepted projections cannot mutate the incoming route");
            assert_eq!(v.world_after.encounter_counter>0,true);
            assert_eq!(v.battle.after.draws+2,v.after.draws,"Shared field return performs the real B/A reset");
            let mut surprised=p;surprised.situation=Some("Surprise".into());assert!(!target_met(&surprised));
            break;
        }
        assert!(covered);
    }

    #[test]
    fn regular_puff_excluded_only_from_cape_cold_targets() {
        let mut rng=CrtRng::new(0);
        let mut p=encounter::catalog().predict(&mut rng,&PredictionOptions {
            map:"Fld_Snw_1_2".into(),zone:"ENT_FLD_SNW_1_2_LND".into(),
            boundary:Boundary::CurrentCounter,current_counter:Some(700),party_levels:vec![5,1],
            ..Default::default()
        }).unwrap();
        let c=encounter::catalog();
        for map in ["Fld_Snw_1_2","Fld_Snw_1_3","Fld_Snw_2_2"] {
            p.map=map.into();p.situation=Some("Normal".into());p.enemies.truncate(1);
            for (family,tier,accepted) in [("OCTOPUS_SNOW",0,map=="Fld_Snw_2_2"),("OCTOPUS_SNOW",1,true),("CAT_SNOW",0,true),("CAT_SNOW",1,true)] {
                p.enemies[0].enemy=c.enemies[&c.rare_labels[family][tier]].clone();
                assert_eq!(target_met(&p),accepted,"{map} {family} tier {tier}");
            }
            p.situation=Some("Surprise".into());assert!(!target_met(&p));
        }
    }
}
