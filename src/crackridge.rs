//! Cleric shopping -> fast travel -> daytime liner ship -> Crackridge Harbor.
use crate::{CrtRng, Result, black_market, ensure,
    world::{self, Decision, Loader, Trace, WorldState}};
use serde::Serialize;
use crate::encounter::{self, Boundary, PredictionOptions};
use ot2_battle_engine::{escape::Situation, field_escape::{FieldEscapeMenu, FieldEscapeAttempt}};
use crate::{manipulation::{self, BoostInputs}, highroad::{self, FieldReturn}};

pub const NEW_DELSTA_HARBOR: &str = "Fld_Cty_1_1";
pub const CRACKRIDGE_HARBOR: &str = "Fld_Wld_1_2";
pub const VOYAGE_EVENT: &str = "SC_LinerShip_Cty_to_Wld";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DaySwitchLocation { Flats, Harbor }

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub day_switch_location: DaySwitchLocation,
    pub arrival: CrtRng,
    pub after_shopping: CrtRng,
    pub at_new_delsta_harbor: CrtRng,
    pub after_day_switch: CrtRng,
    pub before_boat: CrtRng,
    pub after: CrtRng,
    pub world_after: WorldState,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub verification: &'static str,
}

pub fn after_market(route: &black_market::Route) -> Result<Prediction> {
    ensure(route.clerics_present, "Crackridge voyage requires the clerics shopping stop")?;
    with_world_state(route.after_night, route.world_after.clone())
}

pub fn with_world_state(arrival: CrtRng, state: WorldState) -> Result<Prediction> {
    with_day_switch(arrival, state, DaySwitchLocation::Harbor)
}
pub fn with_day_switch(arrival: CrtRng, state: WorldState, location: DaySwitchLocation) -> Result<Prediction> {
    ensure(state.map == "Fld_Cty_1_2" && state.time == 4,
        "Crackridge voyage starts at the Flats clerics at night")?;
    ensure(state.visited_maps.contains(NEW_DELSTA_HARBOR),
        "New Delsta Harbor must be tagged before fast travel")?;
    let mut loader = Loader::new(world::catalog(), state, arrival)?;
    // Ordinary shop purchases change inventory/money, not CRT. This route
    // assumes sufficient funds and no path actions, extra menus or encounters.
    loader.boundary("Clerics: soulstones purchased, shop dismissed");
    let after_shopping = loader.rng;
    let mut after_day_switch = arrival;
    if location == DaySwitchLocation::Flats {
        loader.set_time(2, "Flats: switch to day after buying soulstones, before fast travel")?;
        loader.refresh_bar()?;
        loader.refresh_npcs("Flats: daytime placements after cleric purchases")?;
        after_day_switch = loader.rng;
    }
    loader.visit_map(NEW_DELSTA_HARBOR)?;
    loader.boundary("Fast travel: New Delsta Harbor arrival");
    let at_new_delsta_harbor = loader.rng;
    if location == DaySwitchLocation::Harbor {
        loader.set_time(2, "New Delsta Harbor: immediate switch to day")?;
        loader.refresh_bar()?;
        loader.refresh_npcs("New Delsta Harbor: daytime placements")?;
        after_day_switch = loader.rng;
    }
    loader.boundary("New Delsta Harbor: boat to Crackridge selected and paid");
    let before_boat = loader.rng;
    // Shop's selected route invokes PlayEventForce. Its EventList MapID189
    // loads Fld_Ocn_1_1; command500 then loads map85, Fld_Wld_1_2. FinishPre
    // resets the destination counter because EventList.EncountReset is true.
    loader.skipped_event(VOYAGE_EVENT)?;
    ensure(loader.state.map == CRACKRIDGE_HARBOR, "Boat event ended on an unexpected map")?;
    loader.boundary("Crackridge Harbor: voyage skipped, field control restored");
    Ok(Prediction {
        day_switch_location: location,
        arrival, after_shopping, at_new_delsta_harbor, after_day_switch, before_boat,
        after: loader.rng, world_after: loader.state, trace: loader.trace,
        decisions: loader.decisions,
        verification: "Source-derived; awaiting retail verification. Day switch is replayed at the selected map after shopping. Shared loading executes Harbor arrival, ocean map189, Crackridge map85 and FinishPre counter reset. Assumes skipped sailing, sufficient funds and no intervening encounters/path actions. Purchases and ship payment consume no CRT; inventory/money are not modeled.",
    })
}

pub fn first_encounter(p: &Prediction, throne_level: u32) -> Result<(CrtRng, encounter::Prediction)> {
    ensure(p.world_after.party == [7,6], "Crackridge route requires Throné and Osvald")?;
    let options = PredictionOptions {
        map: CRACKRIDGE_HARBOR.into(), zone: "ENT_FLD_WLD_1_2_LND".into(),
        period: "DayTime".into(), progress: 1, first_encounter: p.world_after.first_encounter,
        boundary: Boundary::CurrentCounter, current_counter: Some(p.world_after.encounter_counter),
        party_levels: vec![throne_level,1], dash: true, ..Default::default()
    };
    let mut rng = p.after;
    let encounter = encounter::catalog().predict(&mut rng, &options)?;
    Ok((rng, encounter))
}

/// Prefer the established Harbor switch; moving it to Flats has no added inputs.
pub fn safe_voyage(route: &black_market::Route, level: u32) -> Result<Option<Prediction>> {
    for location in [DaySwitchLocation::Harbor, DaySwitchLocation::Flats] {
        let p = with_day_switch(route.after_night, route.world_after.clone(), location)?;
        if first_encounter(&p,level)?.1.situation.as_deref() != Some("Surprise") { return Ok(Some(p)); }
    }
    Ok(None)
}

#[derive(Debug, Serialize)]
pub struct EscapePrediction {
    pub encounter: encounter::Prediction,
    pub before_battle_initialization: CrtRng,
    pub menu: Option<FieldEscapeMenu>,
    pub inputs: Option<BoostInputs>,
    pub attempt: Option<FieldEscapeAttempt>,
    pub field_return: Option<FieldReturn>,
    pub status: &'static str,
    pub verification: &'static str,
}

pub fn escape(p: &Prediction, level:u32) -> Result<EscapePrediction> {
    let (entry,encounter) = first_encounter(p,level)?;
    let situation=match encounter.situation.as_deref() {
        Some("Normal")=>Situation::Normal, Some("Preemptive")=>Situation::Preemptive,
        _=>return Ok(EscapePrediction {encounter,before_battle_initialization:entry,menu:None,inputs:None,attempt:None,field_return:None,status:"Surprise: no clean first-menu Escape plan",verification:"Source-derived; not live-validated"}),
    };
    let enemies:Vec<_>=encounter.enemies.iter().map(|e|e.enemy.id.as_str()).collect();
    let menu=FieldEscapeMenu::new_party(entry,situation,[level,1],&enemies)?;
    // Throné Ch1 contributes one committed progress point. Recruitment with
    // skipped Osvald prologue does not commit another chapter-clear point.
    let danger=encounter::catalog().volumes[encounter.volume.as_ref().unwrap()].encounters.iter()
        .filter(|r|r.table!="None" && r.progress<=1).last().unwrap().danger;
    for repeats in 0..=u32::MAX {
        let path:Vec<_>=(0..repeats).flat_map(|_|[1,0]).collect();
        let attempt=menu.clone().attempt(&path,danger)?;
        if attempt.success {
            let inputs=manipulation::describe_inputs(repeats,0,path)?;
            let field_return=highroad::restore_field(&p.world_after,&attempt,CRACKRIDGE_HARBOR,"Crackridge Harbor Escape")?;
            return Ok(EscapePrediction {encounter,before_battle_initialization:entry,menu:Some(menu),inputs:Some(inputs),attempt:Some(attempt),field_return:Some(field_return),status:"First Escape succeeds",verification:"Source-derived first encounter, healthy original-gear Throné/Osvald and first-menu Escape. Study Foe reveals one weakness per distinct new enemy type. No intervening CRT actions or previous regional weakness discoveries. Retail alignment pending."});
        }
    }
    Err(crate::SimError("No Escape in CRT cycle".into()))
}

pub struct RoutePlan { pub market: black_market::Prediction, pub voyage: Prediction, pub escape: EscapePrediction }
pub fn plan(before:CrtRng, world:WorldState, level:u32, clerics_cap:Option<u32>, approach_cap:Option<u32>) -> Result<RoutePlan> {
    let baseline=black_market::predict(before,world.clone(),clerics_cap)?;
    if let Some(voyage)=safe_voyage(&baseline.route,level)? {
        let escape=escape(&voyage,level)?;
        return Ok(RoutePlan {market:baseline,voyage,escape});
    }
    let max_cost=approach_cap.map(|c|baseline.search.weighted_input_cost.saturating_add(c.saturating_mul(2)));
    let improved=black_market::predict_accepting(before,world,clerics_cap,max_cost,|r|Ok(safe_voyage(r,level)?.is_some()))?;
    let market=improved.unwrap_or(baseline);
    let voyage=match safe_voyage(&market.route,level)? {Some(p)=>p,None=>after_market(&market.route)?};
    let escape=escape(&voyage,level)?;
    Ok(RoutePlan {market,voyage,escape})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn market() -> black_market::Route {
        let fixture: serde_json::Value = serde_json::from_str(include_str!("../data/source/capecold-mug-entry-6154.json")).unwrap();
        let before = serde_json::from_value(fixture["before"].clone()).unwrap();
        let mut state: WorldState = serde_json::from_value(fixture["world"].clone()).unwrap();
        state.visited_maps.insert(NEW_DELSTA_HARBOR.into());
        black_market::predict(before, state, None).unwrap().route
    }
    #[test]
    fn voyage_replays_ocean_destination_and_finish_reset() {
        let market = market();
        let p = after_market(&market).unwrap();
        assert_eq!(p.after_shopping, market.after_night);
        assert_eq!(p.world_after.map, CRACKRIDGE_HARBOR);
        assert_eq!(p.world_after.time, 2);
        assert_eq!(p.world_after.party, market.world_after.party);
        // Time switches reroll the black-market flag. Story progression stays.
        for flag in market.world_after.flags.iter().filter(|f| !(25500..=25503).contains(*f)) {
            assert!(p.world_after.flags.contains(flag));
        }
        assert_eq!(p.world_after.active_scenarios, market.world_after.active_scenarios);
        assert!(p.world_after.first_encounter);
        assert!((500..=899).contains(&p.world_after.encounter_counter));
        assert!(p.trace.iter().any(|t| t.subject == "Fld_Ocn_1_1"));
        let resets: Vec<_> = p.trace.iter().filter(|t| t.reason.contains("SC_LinerShip_Cty_to_Wld FinishPre: counter")).collect();
        assert_eq!(resets.len(), 2);
        assert_eq!(resets.last().unwrap().after, p.after);
        let mut loader = Loader::new(world::catalog(), market.world_after.clone(), market.after_night).unwrap();
        loader.visit_map(NEW_DELSTA_HARBOR).unwrap();
        assert_eq!(loader.rng, p.at_new_delsta_harbor);
        loader.set_time(2, "replay day switch").unwrap();
        loader.refresh_bar().unwrap();
        loader.refresh_npcs("replay day placements").unwrap();
        assert_eq!(loader.rng, p.after_day_switch);
        loader.skipped_event(VOYAGE_EVENT).unwrap();
        assert_eq!(loader.rng, p.after);
        println!("Harbor {} -> day {} -> Crackridge {} ({:#010X})", p.at_new_delsta_harbor.draws, p.after_day_switch.draws, p.after.draws, p.after.state);
    }
    #[test]
    fn fast_travel_requires_a_tagged_harbor() {
        let mut market = market();
        market.world_after.visited_maps.remove(NEW_DELSTA_HARBOR);
        assert!(after_market(&market).is_err());
    }
    #[test]
    fn switch_locations_replay_different_maps_and_escape_uses_both_travelers() {
        let market=market();
        let harbor=after_market(&market).unwrap();
        let flats=with_day_switch(market.after_night,market.world_after.clone(),DaySwitchLocation::Flats).unwrap();
        assert_eq!(flats.world_after.time,2);
        assert_ne!(harbor.after,flats.after);
        assert!(flats.after_day_switch.draws<flats.at_new_delsta_harbor.draws);
        assert!(harbor.after_day_switch.draws>harbor.at_new_delsta_harbor.draws);
        let chosen=safe_voyage(&market,5).unwrap().unwrap();
        let escape=escape(&chosen,5).unwrap();
        assert_ne!(escape.encounter.situation.as_deref(),Some("Surprise"));
        assert_eq!(escape.menu.as_ref().unwrap().party_levels,[5,1]);
        let replay=escape.menu.clone().unwrap().attempt(&escape.inputs.as_ref().unwrap().levels,5).unwrap();
        assert!(replay.success);
        assert_eq!(replay.after,escape.attempt.as_ref().unwrap().after);
        assert_eq!(escape.field_return.as_ref().unwrap().world_after.party,[7,6]);
        assert!(!escape.field_return.as_ref().unwrap().world_after.first_encounter);
    }
    #[test]
    fn both_surprise_falls_back_to_mug_while_preserving_clerics() {
        let f:serde_json::Value=serde_json::from_str(include_str!("../data/source/capecold-mug-entry-6154.json")).unwrap();
        let origin:CrtRng=serde_json::from_value(f["before"].clone()).unwrap();
        let mut world:WorldState=serde_json::from_value(f["world"].clone()).unwrap();
        world.visited_maps.insert(NEW_DELSTA_HARBOR.into());
        // Reproducible upstream fallback case found by sweeping fixture entries.
            let mut before=origin; before.advance(1590);
            let baseline=black_market::predict(before,world.clone(),None).unwrap();
            assert!(safe_voyage(&baseline.route,5).unwrap().is_none());
            let optional=plan(before,world.clone(),5,None,Some(0)).unwrap();
            assert!(optional.market.search.weighted_input_cost<=baseline.search.weighted_input_cost);
            let selected=plan(before,world.clone(),5,None,None).unwrap();
            assert!(selected.market.route.clerics_present);
            assert!(selected.escape.attempt.as_ref().unwrap().success);
            assert_ne!(selected.escape.encounter.situation.as_deref(),Some("Surprise"));
            let replay=crate::cape_cold_mug::predict_with_boost_inputs(before,world.clone(),Some(&selected.market.search.inputs.levels)).unwrap();
            let route=black_market::after_mug_with_switches(&replay,selected.market.search.cape_time_switches).unwrap();
            assert_eq!(route.after_night,selected.market.route.after_night);
    }
    #[test]
    fn flats_day_switch_can_avoid_surprise_without_changing_the_market_plan() {
        let mut route=market();
        let mut found=false;
        for seed in 0..256 {
            route.after_night=CrtRng::new(seed);
            let harbor=after_market(&route).unwrap();
            if first_encounter(&harbor,5).unwrap().1.situation.as_deref()!=Some("Surprise") {continue;}
            if let Some(chosen)=safe_voyage(&route,5).unwrap() {
                assert_eq!(chosen.day_switch_location,DaySwitchLocation::Flats);
                assert_eq!(chosen.after_shopping,route.after_night);
                found=true; break;
            }
        }
        assert!(found);
    }
}
