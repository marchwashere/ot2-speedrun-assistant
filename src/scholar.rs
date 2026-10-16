//! Throné's post-recruitment Scholar setup. Every menu callback is CRT-neutral.
use crate::{Result,ensure,snows_escape::Entry,world::{self,Loader}};
use serde::{Deserialize,Serialize};
use std::sync::OnceLock;

pub const JOB:u32=6;
pub const EVASIVE:&str="ABI_SCH_SUP_01";
pub const ELEMENTAL_AUGMENTATION:&str="ABI_SCH_SUP_02";
#[derive(Deserialize)]
struct Data {job_id:u32,default_abilities:Vec<String>,selected_abilities:Vec<String>,
    jp_costs:Vec<u32>,abilities:Vec<Ability>,supports:Vec<Support>}
#[derive(Deserialize)]
struct Ability {id:String,name:String}
#[derive(Deserialize)]
struct Support {id:String,learned_ability_count:usize,effect:String,value:u32}
fn data()->&'static Data {
    static DATA:OnceLock<Data>=OnceLock::new();
    DATA.get_or_init(||serde_json::from_str(include_str!("../data/scholar-assignment.json")).expect("imported Scholar setup"))
}
/// Flat support stats are carried alongside equipment by the growth/job helpers.
/// Add the imported bonus only on the first equip, never on support refresh.
pub(crate) fn equip_elemental_augmentation(w:&mut world::WorldState,id:u32)->Result<()> {
    let support=data().supports.iter().find(|s|s.id==ELEMENTAL_AUGMENTATION).unwrap();
    ensure(support.effect=="ePARAMETER_UP_MATK","Unexpected Elemental Augmentation effect")?;
    let p=w.character_progression.get_mut(&id).ok_or_else(||crate::SimError("Missing support recipient".into()))?;
    ensure(p.jobs.unlocked_support.contains(ELEMENTAL_AUGMENTATION),"Elemental Augmentation not unlocked")?;
    if p.jobs.equipped_support.contains(ELEMENTAL_AUGMENTATION) {return Ok(());}
    ensure(p.jobs.equipped_support.len()<4,"No free support-ability slot")?;
    let c=p.character.combat.as_mut().ok_or_else(||crate::SimError("Missing carried combat stats".into()))?;
    c.magic+=support.value;
    p.jobs.equipped_support.insert(ELEMENTAL_AUGMENTATION.into());
    Ok(())
}
#[derive(Debug,Serialize)]
pub struct Purchase {pub ability:String,pub name:String,pub jp:u32}
#[derive(Debug,Serialize)]
pub struct Prediction {
    pub assignment:Entry,
    pub learning:Entry,
    pub after:Entry,
    pub purchases:Vec<Purchase>,
    pub jp_before:u32,
    pub jp_after:u32,
    pub equipment_change:ot2_battle_engine::jobs::EquipmentChange,
    pub verification:&'static str,
}
fn checkpoint(l:&mut Loader<'_>,arrival:crate::CrtRng,reason:&str)->Entry {
    l.boundary(reason);
    Entry {arrival,after:l.rng,world_after:l.state.clone(),
        trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
fn tutorial(l:&mut Loader<'_>,id:u32)->Result<()> {
    let t=&world::catalog().tutorials[&id];
    if l.state.flag(t.opened_flag) {l.boundary(format!("Tutorial {id} already seen; no popup"));Ok(())}
    else {l.dismiss_tutorial(id)}
}
pub fn predict(entry:&Entry)->Result<Prediction> {
    ensure(entry.world_after.map==crate::ryu::TOWN && entry.world_after.party==[7,1,8,4]
        && entry.world_after.bench==[6],"Throné's Scholar setup follows Hikari's Osvald replacement in Ryu")?;
    let d=data();ensure(d.job_id==JOB,"Unexpected imported Scholar job")?;
    let mut w=entry.world_after.clone();w.sync_progression()?;
    let owned=usize::from(w.job_licenses.get(&JOB).copied().unwrap_or(0));
    // Native SubJobWidget counts second jobs of main AND sub members. Osvald's
    // first job is Scholar, so his primary class consumes none of these licenses.
    let used=w.party.iter().chain(&w.bench).filter(|id|w.character_progression[id].jobs.secondary_job==Some(JOB)).count();
    ensure(owned>used,"No unassigned Scholar License available")?;
    ensure(w.character_progression[&7].jobs.secondary_job.is_none(),"This route expects Throné to have no subjob yet")?;
    let mut l=Loader::new(world::catalog(),w,entry.after)?;
    l.boundary("Main menu opened for Throné's Scholar setup");
    tutorial(&mut l,58)?;
    let equipment_change;
    let jp_before;
    {
        let throne=l.state.character_progression.get_mut(&7).unwrap();
        jp_before=throne.jp_earned.checked_sub(throne.jp_spent).ok_or_else(||crate::SimError("Invalid Throné JP ledger".into()))?;
        equipment_change=ot2_battle_engine::jobs::change_secondary_job(&mut throne.character,7,throne.level,None,Some(JOB))?;
        throne.jobs.secondary_job=Some(JOB);
        // Default command bits were initialized by the new-save constructor;
        // assigning the subjob makes those saved skills available, without JP.
        throne.jobs.learned_abilities.entry(JOB).or_default().extend(d.default_abilities.iter().cloned());
        for ability in &d.default_abilities {
            if !throne.character.skills.contains(ability) {throne.character.skills.push(ability.clone());}
        }
    }
    let assignment=checkpoint(&mut l,entry.after,"Throné assigned Scholar; default Fireball and Icewind available; resources clamped, not healed");
    let mut purchases=Vec::new();
    for id in &d.selected_abilities {
        let new_support;
        {
            let p=l.state.character_progression.get_mut(&7).unwrap();
            let learned=p.jobs.learned_abilities.get_mut(&JOB).unwrap();
            ensure(!learned.contains(id),format!("Scholar ability {id} is already learned"))?;
            let jp=*d.jp_costs.get(learned.len()).ok_or_else(||crate::SimError("Missing Scholar JP cost".into()))?;
            ensure(p.jp_earned-p.jp_spent>=jp,format!("Insufficient JP for {id}"))?;
            let ability=d.abilities.iter().find(|a|&a.id==id).ok_or_else(||crate::SimError(format!("Missing Scholar ability {id}")))?;
            learned.insert(id.clone());p.jp_spent+=jp;p.character.skills.push(id.clone());
            new_support=d.supports.iter().filter(|s|s.learned_ability_count<=learned.len())
                .fold(false,|changed,s|p.jobs.unlocked_support.insert(s.id.clone())||changed);
            purchases.push(Purchase {ability:id.clone(),name:ability.name.clone(),jp});
        }
        l.boundary(format!("Throné learned {}; spent {} JP",purchases.last().unwrap().name,purchases.last().unwrap().jp));
        tutorial(&mut l,54)?;
        if new_support {l.state.set_flag(102,true);tutorial(&mut l,57)?;}
    }
    let learning=checkpoint(&mut l,assignment.after,"Throné learned Lightning Bolt and Elemental Barrage; Evasive Maneuvers unlocked");
    // Entering support equipment clears the native NEW_MAINMENU_SUPPORT_ABILITY
    // notification before SetSupportAbility rebuilds active-party effects.
    l.state.set_flag(102,false);tutorial(&mut l,56)?;
    let p=l.state.character_progression.get_mut(&7).unwrap();
    ensure(p.jobs.unlocked_support.contains(EVASIVE),"Evasive Maneuvers has not been unlocked")?;
    ensure(p.jobs.equipped_support.len()<4,"No free support-ability slot")?;
    p.jobs.equipped_support.insert(EVASIVE.into());
    let jp_after=p.jp_earned-p.jp_spent;
    l.state.refresh_support_skills();
    ensure(l.state.evasive_maneuvers,"Throné's equipped Evasive Maneuvers was not applied to the party")?;
    // Close -> RefreshPartyCharacterFormation / SetAfterMainMenu does not load
    // a map, respawn NPCs or reset the counter. No field-return callback here.
    let after=checkpoint(&mut l,learning.after,"Throné equipped Evasive Maneuvers; main menu closed; field control restored");
    ensure(after.after==entry.after && after.world_after.encounter_counter==entry.world_after.encounter_counter,
        "Scholar setup changed CRT RNG or the encounter counter")?;
    Ok(Prediction {assignment,learning,after,purchases,jp_before,jp_after,equipment_change,
        verification:"Source-audited Scholar assignment, Lightning Bolt and Elemental Barrage purchases, Evasive unlock/equip and menu closure consume no CRT and do not reset the encounter counter. Two purchases cost 30+100 JP. Scholar stat revisions apply before equipment; new staff uses the source zero-stat default. Current HP/SP are clamped rather than healed. Evasive Maneuvers is active on Throné with Osvald benched."})
}

#[cfg(test)]
mod tests {
    use super::*;
    use ot2_battle_engine::{combat::Weapon,progression::Reward};
    fn entry()->Entry {
        let mut w=world::throne_after_second_state().unwrap();
        w.map=crate::ryu::TOWN.into();w.time=2;w.party=vec![7,6,8,4];
        w.party_experience=vec![1591,1528,61,61];w.sync_progression().unwrap();
        w.party=vec![7,1,8,4];w.bench=vec![6];w.party_experience=vec![1591,0,61,61];
        w.sync_progression().unwrap();w.job_licenses.insert(JOB,1);
        w.character_progression.get_mut(&7).unwrap().jp_earned=578;
        let osvald=w.character_progression.get_mut(&6).unwrap();
        osvald.jp_earned=553;osvald.jp_spent=130;
        osvald.jobs.unlocked_support.insert(EVASIVE.into());
        osvald.jobs.equipped_support.insert(EVASIVE.into());
        w.refresh_support_skills();w.encounter_counter=726;
        // Previous guild and Osvald menus were already dismissed.
        for flag in [100,1053,1353,1055,1355,1057,1357] {w.set_flag(flag,true);}
        let mut rng=crate::CrtRng::new(0);rng.advance(7104);
        Entry {arrival:rng,after:rng,world_after:w,trace:vec![],decisions:vec![]}
    }
    #[test]
    fn purchases_apply_real_stats_and_restore_support_without_rng_reset_or_healing() {
        let mut e=entry();
        let p=e.world_after.character_progression.get_mut(&7).unwrap();
        p.character.resources.as_mut().unwrap().hp=17;
        p.character.resources.as_mut().unwrap().sp=9;
        let before=crate::state_key::WorldKey::new(&e.world_after);
        assert!(!e.world_after.evasive_maneuvers,"Bench support does not affect active party");
        let p=predict(&e).unwrap();
        assert_eq!(crate::state_key::WorldKey::new(&e.world_after),before);
        assert_eq!((p.jp_before,p.jp_after),(578,448));
        assert_eq!(p.purchases.iter().map(|p|(p.name.as_str(),p.jp)).collect::<Vec<_>>(),
            [("Lightning Bolt",30),("Elemental Barrage",100)]);
        for stage in [&p.assignment,&p.learning,&p.after] {
            assert_eq!((stage.arrival,stage.after),(e.after,e.after));
            assert_eq!(stage.world_after.encounter_counter,726);
            assert!(stage.trace.iter().all(|s|s.kind==world::DrawKind::Boundary));
        }
        assert!(!p.assignment.world_after.evasive_maneuvers);
        assert!(!p.learning.world_after.evasive_maneuvers,"Unlocking alone does not equip");
        let w=&p.after.world_after;let t=&w.character_progression[&7];
        assert_eq!(t.jobs.secondary_job,Some(JOB));
        assert_eq!(t.jobs.learned_abilities[&JOB],std::collections::BTreeSet::from([
            "ABI_SET_SCH_010".into(),"ABI_SET_SCH_020".into(),"ABI_SET_SCH_030".into(),"ABI_SET_SCH_060".into()]));
        assert_eq!((t.level,t.experience,t.jp_earned,t.jp_spent),(15,1591,578,130));
        assert_eq!((t.character.stats.max_hp,t.character.stats.max_sp),(803,61));
        let c=t.character.combat.as_ref().unwrap();
        assert_eq!((c.magic,c.magic_defense,c.main_weapon),(116,97,Some(Weapon::Dagger)));
        assert_eq!((t.character.resources.as_ref().unwrap().hp,t.character.resources.as_ref().unwrap().sp),(17,9));
        assert_eq!(p.equipment_change.added.len(),1);
        assert_eq!(p.equipment_change.added[0].source_id,"ITM_EQP_ROD_010");
        assert!(w.evasive_maneuvers && t.jobs.equipped_support.contains(EVASIVE));
        assert_eq!(w.job_licenses[&JOB],1,"Assignment does not consume the owned license");
        assert!(!w.flag(102));assert!(!w.flag(7),"Raw JobData.PermissionFlag is not an EventFlag gate");
        for id in [1,4,6,8] {assert_eq!(serde_json::to_value(&w.character_progression[&id]).unwrap(),
            serde_json::to_value(&e.world_after.character_progression[&id]).unwrap());}
    }
    #[test]
    fn insufficient_jp_and_benched_secondary_license_use_are_rejected_atomically() {
        for case in 0..3 {
            let mut e=entry();
            match case {
                0=>{e.world_after.job_licenses.clear();},
                1=>{e.world_after.character_progression.get_mut(&6).unwrap().jobs.secondary_job=Some(JOB);},
                _=>{e.world_after.character_progression.get_mut(&7).unwrap().jp_earned=129;},
            }
            let before=crate::state_key::WorldKey::new(&e.world_after);
            assert!(predict(&e).is_err());
            assert_eq!(crate::state_key::WorldKey::new(&e.world_after),before);
        }
    }
    #[test]
    fn serialization_sync_and_future_rewards_preserve_job_stats_and_skills() {
        let p=predict(&entry()).unwrap();
        let mut w:world::WorldState=serde_json::from_value(serde_json::to_value(&p.after.world_after).unwrap()).unwrap();
        let before=crate::state_key::WorldKey::new(&w);w.sync_progression().unwrap();
        assert_eq!(crate::state_key::WorldKey::new(&w),before,"Sync must not apply Scholar bonuses twice");
        let threshold=ot2_battle_engine::progression::growth_at_level(7,16).unwrap().total_exp;
        w.award_victory(Reward {exp:threshold-1591,jp:1,money:0}).unwrap();
        let t=&w.character_progression[&7];
        let mut expected=ot2_battle_engine::field_escape::route_player(7,threshold).unwrap();
        ot2_battle_engine::jobs::change_secondary_job(&mut expected,7,16,None,Some(JOB)).unwrap();
        assert_eq!(serde_json::to_value(&t.character.stats).unwrap(),serde_json::to_value(&expected.stats).unwrap());
        assert_eq!(serde_json::to_value(&t.character.combat).unwrap(),serde_json::to_value(&expected.combat).unwrap());
        assert_eq!((t.level,t.jp_spent),(16,130));
        assert_eq!(t.jobs.learned_abilities[&JOB].len(),4);assert!(w.evasive_maneuvers);
        let osvald=&w.character_progression[&6];assert_eq!((osvald.experience,osvald.jp_earned,osvald.jp_spent),(1528,553,130));
    }
}
