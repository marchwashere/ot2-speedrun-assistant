//! Pirro R3 observed checkpoint: no search of the preceding route.
use crate::{CrtRng,Result,SimError,ensure,pirro,world::WorldState};
use ot2_battle_engine::{Battle,BattleConfig,ItemCatalog,ActorId,Outcome,pirro::PirroRoutine};
use std::collections::{HashSet,VecDeque};

fn round_three(seed:u32)->Result<Battle> {
    let mut b=Battle::new_pirro(BattleConfig::throne_pirro(seed)?,ItemCatalog::builtin()?,false)?;
    let mut routine=PirroRoutine::default();
    for _ in 0..8 {
        if b.state().round==3 && b.current_actor()==Some(ActorId(0)){return Ok(b);}
        routine.step(&mut b)?;
    }
    Err(SimError("Standard R1/R2 did not reach the R3 menu".into()))
}

/// Reconstruct rather than overwrite an unrelated battle. All accepted starts
/// must replay into the observed state, turn-order observation and voice transcript.
/// The bounded prefix supports the fixed two-turn policy only; no match is an
/// explicit error, never a manufactured checkpoint.
pub fn checkpoints(initial:u32,current:u32,voices:&str)->Result<Vec<Battle>> {
    ensure(!voices.is_empty() && voices.bytes().all(|b|matches!(b,b'N'|b'S')),"Missing voice transcript")?;
    let mut matches=Vec::new();
    for distance in 1..512u64 {
        let mut seed=CrtRng::new(initial);seed.advance((1u64<<32)-distance);
        let Ok(mut b)=round_three(seed.state) else {continue};
        b.advance_boost_input(0,1)?;b.advance_boost_input(1,2)?;
        if b.state().rng.state&0x7fff_ffff!=initial&0x7fff_ffff {continue;}
        let mut valid=true;
        for (i,voice) in voices.bytes().enumerate() {
            if i!=0 {b.advance_boost_input(3,2)?;}
            let before=b.state().rng;
            let mut check=before;check.advance(3);
            let normal=check.state&0x4000_0000!=0;
            b.advance_boost_input(2,3)?;
            if normal!=(voice==b'N') || b.state().rng.draws-before.draws!=if normal {4}else{3} {valid=false;break;}
        }
        if !valid || b.state().rng.state&0x7fff_ffff!=current&0x7fff_ffff {continue;}
        b.rebase_pirro_identification()?;
        if !matches.iter().any(|m:&Battle|m.state()==b.state()) {matches.push(b);}
    }
    ensure(!matches.is_empty(),"No matching Pirro R3 checkpoint. Check voice entries and the fixed R1/R2 routine; do not use a guessed battle state.")?;
    Ok(matches)
}

pub fn checkpoint(initial:u32,current:u32,voices:&str,pirro_first:Option<bool>,hp:Option<u32>)->Result<Battle> {
    let mut states=checkpoints(initial,current,voices)?;
    if let Some(first)=pirro_first {states.retain(|b|(b.state().turn_order().first()==Some(&ActorId(1)))==first);}
    if let Some(hp)=hp {states.retain(|b|b.state().actors[0].resources.hp==hp);}
    ensure(!states.is_empty(),"No battle state matches the entered round-3 turn order and HP. Check the observations, order and HP.")?;
    ensure(states.len()==1,"RNG identified. Select who acted first in round 3 (Throné or Pirro) and enter current HP to resolve the battle state.")?;
    Ok(states.remove(0))
}

pub fn plan(menu:&Battle)->Result<pirro::Prediction> {
    let mut world:WorldState=serde_json::from_str(include_str!("../data/source/pirro-identification-world.json"))?;
    world.set_flag(8017,false);
    let config=BattleConfig::throne_pirro(menu.state().rng.state)?;
    let run=|path:&[u8]|pirro::predict_from_checkpoint(menu.state().rng,&world,
        menu.state().actors[0].resources.clone(),config.inventory.clone(),path,&[1,2,3],Some(menu));
    let accepts=|p:&pirro::Prediction|p.state.outcome==Outcome::Victory && p.routine_error.is_none()
        && p.routine.heal_before_round_eight==Some(false);
    let baseline=run(&[])?;
    if accepts(&baseline){return Ok(baseline);}
    let mut queue=VecDeque::from([(menu.clone(),3u8,Vec::<u8>::new())]);
    let mut seen=HashSet::new();
    while let Some((b,bp,path))=queue.pop_front() {
        for next in [bp.checked_sub(1),bp.checked_add(1).filter(|&n|n<=3)].into_iter().flatten() {
            let mut b=b.clone();b.advance_boost_input(bp,next)?;
            if !seen.insert((b.state().rng.state,b.state().opening_crt.as_ref().unwrap().used_voices.clone(),next)){continue;}
            let mut path=path.clone();path.push(next);
            if next==3 {let p=run(&path)?;if accepts(&p){return Ok(p);}}
            queue.push_back((b,next,path));
        }
    }
    Err(SimError("No winning no-grape continuation exists in the supported R3 boost state space".into()))
}


/// Continue the existing route from the observed checkpoint, preserving its
/// nearest-control search policy and all downstream state propagation.
pub fn simulate(voices:&str,pirro_first:bool,hp:u32)->Result<serde_json::Value> {
    simulate_route(voices,pirro_first,hp,false)
}
pub fn simulate_route(voices:&str,pirro_first:bool,hp:u32,galdera_route:bool)->Result<serde_json::Value> {
    ensure((1..=261).contains(&hp),"Enter Throné’s current R3 HP (1–261)")?;
    let identified=crate::voice_identification::identify_observed(voices,Some((pirro_first,hp)))?;
    let candidate=&identified["candidate"];
    let initial=candidate["initial_state"].as_u64().ok_or_else(||SimError("RNG is not uniquely identified yet".into()))? as u32;
    let seed=candidate["current_state"].as_u64().unwrap() as u32;
    let checkpoint=checkpoint(initial,seed,voices,Some(pirro_first),Some(hp))?;
    let mut boss=plan(&checkpoint)?;
    let settings=crate::highroad::Settings {approach_max_boost_repeats:None,break_max_boost_repeats:None,
        flats_max_boost_repeats:None,flats_approach_max_boost_repeats:None,flats_break_max_boost_repeats:None,
        ..Default::default()};
    let (ending,highroad)=crate::highroad::from_identified_battle(&mut boss,settings)?
        .ok_or_else(||SimError("Pirro did not reach the Highroad".into()))?;
    let field=highroad.field_return.as_ref().ok_or_else(||SimError("Highroad Escape unavailable".into()))?;
    let flats=crate::flats::after_highroad(field)?;
    let mut harbor=crate::harbor::after_flats(&flats)?.ok_or_else(||SimError("Harbor not reached".into()))?;
    harbor.world_after.galdera_route=galdera_route;
    let escape=crate::flats_escape::until_osvald_cait(harbor.after,&harbor.world_after,highroad.level,
        Default::default(),Default::default())?;
    let mut result=serde_json::json!({
        "route":if galdera_route {"galdera"} else {"any_hikari"},"identified_entry":identified,"r3_observation":{"pirro_first":pirro_first,"hp":hp},"initial_seed":seed,"actors":boss.actors,"action_names":boss.action_names,
        "initial_state":boss.initial_state,"state":boss.state,"steps":boss.steps,
        "crt_setup":{"route":if galdera_route {"galdera"} else {"any_hikari"},"initial_seed":seed,"battle_seed":seed,"advances":0,"battle_entry_advances":0,
            "recreate_shared_level":false,"transition_checkpoints":[]},
        "post_battle_field":{"pirro_battle":boss,"pirro_aftermath":ending,"highroad":highroad,
            "flats":flats,"harbor_return":harbor,"flats_escape":escape}
    });
    crate::conning::extend_selected_routes(&mut result)?;
    Ok(result)
}

#[cfg(test)] mod tests {
    use super::*;
    #[test]
    #[ignore = "Full live-transcript Galdera route replay"]
    fn galdera_skips_hikari_and_replays_full_route() {
        let result=simulate_route("SNNNNSSSSSNSNSSSSNNNSNSSNNSNS",true,154,true).unwrap();
        fn find_ryu(v:&serde_json::Value)->Option<&serde_json::Value> {
            if v["town_entry"].is_object() && v["canalbrine"]["water"].is_object() {return Some(v);}
            v.as_object()?.values().find_map(find_ryu)
        }
        let r=find_ryu(&result).expect("Ryu continuation");
        assert!(r["hikari"].is_null());
        assert_eq!(r["canalbrine"]["water"]["field_return"]["world_after"]["party"],serde_json::json!([7,6,8,4]));
        assert_eq!(r["canalbrine"]["water"]["field_return"]["world_after"]["bench"],serde_json::json!([]));
        assert!(r["throne_scholar"].is_null());
        let recruited:WorldState=serde_json::from_value(r["town_entry"]["world_after"].clone()).unwrap();
        assert!(!recruited.completed_scenarios.contains(&100));
        assert!(!recruited.flag(8050));
        assert!(!recruited.active_scenarios.contains(&701));
        assert!(!recruited.confirmed_scenarios.contains(&701));
        let inventor=&r["canalbrine"]["conning"]["pomegranate"]["inventor"];
        let mut leg=&inventor["crackridge_return"];
        while leg["southern"].is_object(){leg=&leg["southern"];}
        let g=&leg["gravell"];
        assert_eq!(g["coast"]["eastern_sai"]["world_after"]["map"],"Fld_Dst_2_4");
        assert_eq!(g["town"]["world_after"]["time"],2);
        assert!(g["coast"]["movement"]["estimated_counter_at_exit"].as_u64().unwrap()>0);
        assert_eq!(g["coast"]["southern_sai"]["escape"]["chance_percent"],100);
        assert_eq!(g["coast"]["southern_sai"]["escape"]["success"],true);
        assert_eq!(g["coast"]["eastern_sai_encounter"]["escape"]["success"],true);
        assert_eq!(g["coast"]["sai"]["world_after"]["map"],"Twn_Dst_2_1_A");
        assert_eq!(g["coast"]["oresrush_supplies"]["pickaxe_required_level"],30);
        assert_eq!(g["coast"]["oresrush_supplies"]["pickaxe"]["world_after"]["time"],4);
        assert_eq!(g["coast"]["oresrush_supplies"]["augmentor"]["entreat"]["world_after"]["route_items"]["ITM_EQP_ACS_250"],1);
        let wg=&g["coast"]["oresrush_supplies"]["wellgrove"];
        assert_eq!(wg["day"]["world_after"]["time"],2);
        assert_eq!(wg["southern_sai"]["world_after"]["map"],"Fld_Dst_2_3");
        assert_eq!(wg["western_sai"]["arrival"]["world_after"]["map"],"Fld_Dst_2_2");
        assert_eq!(wg["western_sai"]["escape"]["chance_percent"],100);
        assert_eq!(wg["western_sai"]["escape"]["success"],true);
        assert_ne!(wg["western_sai"]["encounter"]["situation"],"Surprise");
        assert_eq!(wg["eastern_wellgrove"]["world_after"]["map"],"Fld_Fst_2_2");
        let champion=&wg["champion"];
        assert_eq!(champion["steal_chance"],100);
        assert_eq!(champion["steal"]["world_after"]["route_items"]["ITM_EQP_DGR_120"],1);
        assert_ne!(champion["encounter"]["encounter"]["situation"],"Surprise");
        assert_eq!(champion["encounter"]["escape"]["chance_percent"],100);
        assert_eq!(champion["encounter"]["escape"]["success"],true);
        assert_eq!(champion["wellgrove"]["world_after"]["map"],"Twn_Fst_2_1_A");
        let supplies=&champion["supplies"];
        assert_eq!(supplies["dancer_license"]["world_after"]["job_licenses"]["5"],1);
        for item in ["ITM_CSM_0670","ITM_MRL_COM_0010","ITM_CSM_0230"] {assert_eq!(supplies["dancer_license"]["world_after"]["route_items"][item],1);}
        assert_eq!(g["coast"]["oresrush_supplies"]["pickaxe"]["world_after"]["hired_merchant"]["talent"],"SPACT_MERCHANT_014");
        assert_ne!(g["coast"]["southern_sai"]["encounter"]["situation"],"Surprise");
        assert_eq!(g["fight"]["state"]["outcome"],"victory");
        assert_eq!(g["fight"]["evaluation"]["result"],"accepted");
        assert_eq!(g["field_return"]["world_after"]["party"],serde_json::json!([7,6,8,4]));
        let bench=g["field_return"]["world_after"]["bench"].as_array().unwrap();
        assert!(!bench.contains(&serde_json::json!(1))&&bench.contains(&serde_json::json!(3)));
        println!("{}",serde_json::json!({"bench":bench,"exp100_evaluation":g["fight"]["evaluation"],"new_delsta_switches":inventor["new_delsta_adjustment"]["switches"]}));
        if let Ok(path)=std::env::var("OT2_GALDERA_REPLAY_OUTPUT") {std::fs::write(path,serde_json::to_string_pretty(&result).unwrap()).unwrap();}
    }
    #[test] fn round_three_order_distinguishes_sampled_prefixes_without_hp() {
        for seed in (0u32..128).map(|n|n.wrapping_mul(1775700753)) {
            let mut b=round_three(seed).unwrap();
            let first=b.state().turn_order().first()==Some(&ActorId(1));
            b.advance_boost_input(0,1).unwrap();b.advance_boost_input(1,2).unwrap();
            let initial=b.state().rng.state;let mut c=b.state().rng;c.advance(3);
            let voice=if c.state&0x4000_0000!=0{"N"}else{"S"};b.advance_boost_input(2,3).unwrap();
            let restored=checkpoint(initial,b.state().rng.state,voice,Some(first),None).unwrap();
            b.rebase_pirro_identification().unwrap();assert_eq!(b.state(),restored.state());
        }
    }
    #[test] fn checkpoint_preserves_battle_and_starts_draws_at_zero() {
        let mut original=round_three(5678).unwrap();
        let pirro_first=original.state().turn_order().first()==Some(&ActorId(1));
        original.advance_boost_input(0,1).unwrap();original.advance_boost_input(1,2).unwrap();
        let initial=original.state().rng.state;let mut voices=String::new();
        for i in 0..40 {
            if i!=0{original.advance_boost_input(3,2).unwrap();}
            let mut check=original.state().rng;check.advance(3);
            voices.push(if check.state&0x4000_0000!=0{'N'}else{'S'});
            original.advance_boost_input(2,3).unwrap();
        }
        let restored=checkpoint(initial,original.state().rng.state,&voices,Some(pirro_first),Some(original.state().actors[0].resources.hp)).unwrap();
        original.rebase_pirro_identification().unwrap();
        assert_eq!(original.state(),restored.state());
        assert_eq!(restored.state().rng.draws,0);
        let mut a=original;let mut b=restored;
        assert_eq!(a.pirro_identified_darkest_night().unwrap(),b.pirro_identified_darkest_night().unwrap());
    }
    #[test] fn inconsistent_hp_does_not_overwrite_reconstructed_resources() {
        let mut b=round_three(5678).unwrap();
        let first=b.state().turn_order().first()==Some(&ActorId(1));
        b.advance_boost_input(0,1).unwrap();b.advance_boost_input(1,2).unwrap();
        let initial=b.state().rng.state;let mut c=b.state().rng;c.advance(3);
        let voice=if c.state&0x4000_0000!=0{"N"}else{"S"};b.advance_boost_input(2,3).unwrap();
        assert!(checkpoint(initial,b.state().rng.state,voice,Some(first),Some(1)).is_err());
    }
    #[test] fn ambiguous_prefix_is_rejected_without_order() {
        let mut b=round_three(0).unwrap();b.advance_boost_input(0,1).unwrap();b.advance_boost_input(1,2).unwrap();
        let initial=b.state().rng.state;let mut check=b.state().rng;check.advance(3);
        let voice=if check.state&0x4000_0000!=0{"N"}else{"S"};b.advance_boost_input(2,3).unwrap();
        assert!(checkpoint(initial,b.state().rng.state,voice,None,None).unwrap_err().to_string().contains("who acted first"));
    }
}
