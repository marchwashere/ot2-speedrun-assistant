//! Wellgrove tavern Steal -> nighttime Lychee Entreat -> first Dancer License.
use crate::{Result,CrtRng,ensure,world::{self,Loader},snows_escape::Entry};
use serde::{Deserialize,Serialize};
use std::sync::OnceLock;
#[derive(Deserialize)]struct Data {shopkeeper:Shopkeeper,mother:Mother,dancer_job:u32,guild_resource:String,guild_events:Vec<String>}
#[derive(Deserialize)]struct Shopkeeper {npc:String,items:Vec<Item>}
#[derive(Deserialize)]struct Item {item:String,proper_steal:u32,guaranteed_level:u32}
#[derive(Deserialize)]struct Mother {npc:String,item:String,required_level:u32}
fn data()->&'static Data {static D:OnceLock<Data>=OnceLock::new();D.get_or_init(||serde_json::from_str(include_str!("../data/galdera-wellgrove-supplies.json")).unwrap())}
#[derive(Debug,Serialize)]pub struct Trip {
    pub tavern_steal:Entry,pub night:Entry,pub mother_entreat:Entry,pub dancer_license:Entry,
    pub throne_level:u32,pub steal_chances:Vec<u32>,pub steal_rolls:Vec<u32>,pub agnea_level:u32,pub required_level:u32,
}
fn point(l:&mut Loader<'_>,before:CrtRng)->Entry {
    Entry {arrival:before,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
fn present(l:&Loader<'_>,npc:&str)->bool {l.state.existing_npcs.iter().any(|id|l.data.placements[id].resource==npc)}
pub(crate) fn finish(entry:&Entry)->Result<Trip> {
    let d=data();let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
    ensure(l.state.map=="Twn_Fst_2_1_A"&&l.state.time==2,"Wellgrove supplies begin at daytime town entry")?;
    ensure(l.state.party.contains(&7)&&present(&l,&d.shopkeeper.npc),"Throné or tavern Shopkeeper unavailable")?;
    let throne_level=l.state.character_progression[&7].level;
    ensure(d.shopkeeper.items.iter().all(|i|throne_level>=i.guaranteed_level),"Wellgrove tavern steals are not guaranteed at the carried level")?;
    l.draw(world::DrawKind::PathActionMenu,&d.shopkeeper.npc,"Open tavern Shopkeeper Steal menu: action timer",None);
    let mut steal_rolls=vec![];
    for i in &d.shopkeeper.items {
        ensure(!l.state.route_items.contains_key(&i.item),"Wellgrove tavern item already obtained")?;
        let roll=l.draw(world::DrawKind::StealCheck,&i.item,format!("ProperSteal{}, Throné level{throne_level}, probability100%",i.proper_steal),Some(100))+1;
        if let Some(t)=l.trace.last_mut(){t.value=Some(roll);}
        steal_rolls.push(roll);*l.state.route_items.entry(i.item.clone()).or_default()+=1;
    }
    l.boundary("Sharp Nut (L) and Diffusing Serum stolen in one session; change selected path action, then close without dialogue or closing lottery");
    let tavern_steal=point(&mut l,entry.after);
    l.manual_time_switch(4,"Wellgrove: switch to night for Girl's Mother Entreat")?;
    let night=point(&mut l,tavern_steal.after);
    ensure(l.state.party.contains(&8)&&present(&l,&d.mother.npc),"Agnea or Girl's Mother unavailable")?;
    let agnea_level=l.state.character_progression[&8].level;
    ensure(agnea_level>=d.mother.required_level,"Agnea does not meet Empowering Lychee (M) Entreat level")?;
    l.draw(world::DrawKind::PathActionMenu,&d.mother.npc,"Girl's Mother Entreat: opening action timer",None);
    *l.state.route_items.entry(d.mother.item.clone()).or_default()+=1;
    l.boundary("Empowering Lychee (M) entreated; change selected path action, then close without dialogue or closing lottery");
    let mother_entreat=point(&mut l,night.after);
    ensure(present(&l,&d.guild_resource),"Dancer guild unavailable")?;
    for event in &d.guild_events {l.skipped_event(event)?;}
    l.state.job_licenses.entry(d.dancer_job).or_insert(1);
    l.boundary("First Dancer License acquired; no job assignment or skill purchases");
    let dancer_license=point(&mut l,mother_entreat.after);
    Ok(Trip {tavern_steal,night,mother_entreat,dancer_license,throne_level,steal_chances:vec![100;d.shopkeeper.items.len()],steal_rolls,agnea_level,required_level:d.mother.required_level})
}
