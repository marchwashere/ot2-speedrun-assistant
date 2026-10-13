//! Dancer License -> New Delsta day -> Highroad -> Al's bag delivery.
use crate::{Result,CrtRng,ensure,world::{self,Loader,EventChoices},snows_escape::Entry};
use serde::{Deserialize,Serialize};
use std::sync::OnceLock;
#[derive(Deserialize)]struct Data {event:String,npc:String,bag:String,reward_money:u32,rewards:Vec<Reward>}
#[derive(Deserialize)]struct Reward {item:String,quantity:u32}
fn data()->&'static Data {static D:OnceLock<Data>=OnceLock::new();D.get_or_init(||serde_json::from_str(include_str!("../data/galdera-al-return.json")).unwrap())}
#[derive(Debug,Serialize)]pub struct Trip {pub new_delsta:Entry,pub day:Entry,pub highroad:Entry,pub delivery:Entry,pub reward_money:u32,pub flamechurch:crate::galdera_flamechurch::Trip}
fn point(l:&mut Loader<'_>,before:CrtRng)->Entry {
    Entry {arrival:before,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
pub(crate) fn finish(entry:&Entry)->Result<Trip> {
    let d=data();let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
    ensure(l.state.time==4,"Al return begins at night after Dancer License")?;
    l.visit_map("Twn_Cty_1_1_A")?;l.skipped_event("MapInStorySelect")?;
    let new_delsta=point(&mut l,entry.after);
    l.manual_time_switch(2,"New Delsta: switch to day immediately after returning from Wellgrove")?;
    let day=point(&mut l,new_delsta.after);
    l.visit_map("Fld_Cty_1_3")?;let highroad=point(&mut l,day.after);
    ensure(l.state.flag(20510)&&!l.state.flag(20511)&&l.state.route_items.get(&d.bag).copied().unwrap_or(0)>0,"Al's recovered bag is unavailable for hand-in")?;
    ensure(l.state.existing_npcs.iter().any(|id|l.data.placements[id].resource==d.npc),"Al is unavailable on New Delsta Highroad")?;
    // Direct entry-to-Al leg: no distance supplied, no invented encounter or
    // movement counter budget. The actual completion event resets the counter.
    l.skipped_event_with_choices(&d.event,&EventChoices::from([((d.event.clone(),4),"eDIALOG_YES".into())]))?;
    ensure(l.state.flag(20511)&&!l.state.route_items.contains_key(&d.bag),"Al's bag was not delivered")?;
    ensure(d.rewards.iter().all(|r|l.state.route_items.get(&r.item).copied().unwrap_or(0)>=r.quantity),"Traveler's Bag rewards missing")?;
    let delivery=point(&mut l,highroad.after);
    let flamechurch=crate::galdera_flamechurch::finish(&delivery)?;
    Ok(Trip {new_delsta,day,highroad,delivery,reward_money:d.reward_money,flamechurch})
}
