//! Safe 43-update approach -> Champion Seeker Steal -> one escape -> Wellgrove.
use crate::{Result,CrtRng,ensure,world::{self,Loader},snows_escape::Entry,galdera_coast::{self,SouthernSai},encounter::{self,PredictionOptions}};
use serde::{Deserialize,Serialize};
use std::sync::OnceLock;
const NPC_UPDATES:u32=43;
const TOTAL_UPDATES:u32=125;
#[derive(Deserialize)]struct Data {npc:String,item:String,proper_steal:u32,guaranteed_level:u32}
fn data()->&'static Data {static D:OnceLock<Data>=OnceLock::new();D.get_or_init(||serde_json::from_str(include_str!("../data/galdera-champion.json")).unwrap())}
#[derive(Debug,Serialize)]pub struct Trip {
    pub approach:Entry,pub steal:Entry,pub encounter:SouthernSai,pub exit:Entry,pub wellgrove:Entry,
    pub steal_chance:u32,pub steal_roll:u32,pub throne_level:u32,
    pub running_updates_to_npc:u32,pub total_running_updates:u32,pub run_decrement:u32,pub running_updates_after_escape:u32,
    pub supplies:crate::galdera_wellgrove_supplies::Trip,
}
fn point(l:&mut Loader<'_>,before:CrtRng)->Entry {
    Entry {arrival:before,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
pub(crate) fn finish(entry:&Entry)->Result<Option<Trip>> {
    let d=data();let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
    ensure(l.state.map=="Fld_Fst_2_2"&&l.state.time==2,"Champion Seeker requires daytime Eastern Wellgrove Trail")?;
    ensure(l.state.party.contains(&7)&&l.state.existing_npcs.iter().any(|id|l.data.placements[id].resource==d.npc),"Throné or Man with Horses is unavailable")?;
    let throne_level=l.state.character_progression[&7].level;
    ensure(throne_level>=d.guaranteed_level,"Champion Seeker steal is not guaranteed at the carried Throné level")?;
    let decrement=encounter::catalog().counter_decrement(&PredictionOptions {zone:"ENT_FLD_FST_2_2_LND".into(),period:"DayTime".into(),ability_percent:if l.state.evasive_maneuvers {50}else{100},dash:true,..Default::default()})?;
    let npc_cost=NPC_UPDATES*decrement;
    if l.state.encounter_counter<=npc_cost||l.state.encounter_counter>TOTAL_UPDATES*decrement {return Ok(None);}
    l.state.encounter_counter-=npc_cost;
    l.boundary(format!("Eastern Wellgrove Trail: run 100% to Man with Horses; {NPC_UPDATES} updates x {decrement}; no encounter before Steal"));
    let approach=point(&mut l,entry.after);
    ensure(!l.state.route_items.contains_key(&d.item),"Champion Seeker already obtained")?;
    l.draw(world::DrawKind::PathActionMenu,&d.npc,"Steal: PlayFieldCommandActions / K2_SetTimerDelegate",None);
    let steal_roll=l.draw(world::DrawKind::StealCheck,&d.item,format!("Champion Seeker: ProperSteal{}, Throné level{throne_level}, probability100%",d.proper_steal),Some(100))+1;
    if let Some(t)=l.trace.last_mut(){t.value=Some(steal_roll);}
    *l.state.route_items.entry(d.item.clone()).or_default()+=1;
    l.boundary("Champion Seeker stolen: change selected path action, then close without dialogue or closing lottery");
    let steal=point(&mut l,approach.after);
    let Some(encounter)=galdera_coast::sai_encounter(steal.clone(),"ENT_FLD_FST_2_2_LND")? else {return Ok(None);};
    let before_encounter=encounter.encounter.counter_updates.unwrap();
    let remaining=TOTAL_UPDATES-NPC_UPDATES-before_encounter;
    let mut l=Loader::new(world::catalog(),encounter.field_return.world_after.clone(),encounter.field_return.after)?;
    ensure(l.state.encounter_counter>remaining*decrement,"A second encounter would occur before Wellgrove")?;
    l.state.encounter_counter-=remaining*decrement;
    l.boundary(format!("Eastern Wellgrove Trail: after Escape run 100% to Wellgrove; {remaining} updates x {decrement}, no second encounter"));
    let exit=point(&mut l,encounter.field_return.after);
    l.visit_map("Twn_Fst_2_1_A")?;l.skipped_event("MapInStorySelect")?;
    let wellgrove=point(&mut l,exit.after);
    let supplies=crate::galdera_wellgrove_supplies::finish(&wellgrove)?;
    Ok(Some(Trip {approach,steal,encounter,exit,wellgrove,steal_chance:100,steal_roll,throne_level,running_updates_to_npc:NPC_UPDATES,total_running_updates:TOTAL_UPDATES,run_decrement:decrement,running_updates_after_escape:remaining,supplies}))
}
