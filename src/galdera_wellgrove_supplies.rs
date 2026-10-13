//! Daytime Pom M x2 Steal -> night -> tavern/Lychee Entreat -> Dancer License.
use crate::{Result,CrtRng,ensure,world::{self,Loader},snows_escape::Entry};
use serde::{Deserialize,Serialize};
use std::sync::OnceLock;
#[derive(Deserialize)]struct Data {merchant:Merchant,shopkeeper:Shopkeeper,mother:Mother,magic_merchant:Mother,dancer_job:u32,guild_resource:String,guild_events:Vec<String>}
#[derive(Deserialize)]struct Merchant {npc:String,item:String,quantity:u32,proper_steal:u32,guaranteed_level:u32}
#[derive(Deserialize)]struct Shopkeeper {npc:String,items:Vec<Item>}
#[derive(Deserialize)]struct Item {item:String,required_level:u32}
#[derive(Deserialize)]struct Mother {npc:String,item:String,required_level:u32}
fn data()->&'static Data {static D:OnceLock<Data>=OnceLock::new();D.get_or_init(||serde_json::from_str(include_str!("../data/galdera-wellgrove-supplies.json")).unwrap())}
#[derive(Debug,Serialize)]pub struct Trip {
    pub pom_steal:Entry,pub pom_steal_rolls:Vec<u32>,pub pom_steal_chance:u32,pub throne_level:u32,
    pub night:Entry,pub tavern_entreat:Entry,pub mother_entreat:Entry,pub magic_entreat:Entry,pub dancer_license:Entry,
    pub tavern_required_levels:Vec<u32>,pub agnea_level:u32,pub required_level:u32,
    pub al_return:crate::galdera_al_return::Trip,
}
fn point(l:&mut Loader<'_>,before:CrtRng)->Entry {
    Entry {arrival:before,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
fn present(l:&Loader<'_>,npc:&str)->bool {l.state.existing_npcs.iter().any(|id|l.data.placements[id].resource==npc)}
pub(crate) fn finish(entry:&Entry)->Result<Trip> {
    let d=data();let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
    ensure(l.state.map=="Twn_Fst_2_1_A"&&l.state.time==2,"Wellgrove supplies begin at daytime town entry")?;
    ensure(l.state.party.contains(&7)&&present(&l,&d.merchant.npc),"Throné or Wellgrove Merchant unavailable")?;
    let throne_level=l.state.character_progression[&7].level;
    ensure(throne_level>=d.merchant.guaranteed_level,"Wellgrove Pom M steals are not guaranteed at the carried Throné level")?;
    l.draw(world::DrawKind::PathActionMenu,&d.merchant.npc,"Open Merchant Steal menu: action timer",None);
    let mut pom_steal_rolls=Vec::new();
    for slot in 1..=d.merchant.quantity {
        let roll=l.draw(world::DrawKind::StealCheck,&d.merchant.item,format!("Pom M slot{slot}: ProperSteal{}, Throné level{throne_level}, probability100%",d.merchant.proper_steal),Some(100))+1;
        if let Some(t)=l.trace.last_mut(){t.value=Some(roll);}
        pom_steal_rolls.push(roll);
        *l.state.route_items.entry(d.merchant.item.clone()).or_default()+=1;
    }
    l.boundary("Two Energizing Pomegranates (M) stolen in one menu; change selected path action, then close without dialogue or closing lottery");
    let pom_steal=point(&mut l,entry.after);
    l.manual_time_switch(4,"Wellgrove: switch to night after stealing both Pom M")?;
    let night=point(&mut l,pom_steal.after);
    ensure(l.state.party.contains(&8)&&present(&l,&d.shopkeeper.npc),"Agnea or tavern Shopkeeper unavailable")?;
    let agnea_level=l.state.character_progression[&8].level;
    ensure(d.shopkeeper.items.iter().all(|i|agnea_level>=i.required_level),"Agnea does not meet Wellgrove tavern Entreat requirements")?;
    l.draw(world::DrawKind::PathActionMenu,&d.shopkeeper.npc,"Open tavern Shopkeeper Entreat menu: action timer",None);
    for i in &d.shopkeeper.items {
        ensure(!l.state.route_items.contains_key(&i.item),"Wellgrove tavern item already obtained")?;
        *l.state.route_items.entry(i.item.clone()).or_default()+=1;
    }
    l.boundary("Sharp Nut (L) and Diffusing Serum entreated in one session; change selected path action, then close without dialogue or closing lottery");
    let tavern_entreat=point(&mut l,night.after);
    ensure(l.state.party.contains(&8)&&present(&l,&d.mother.npc),"Agnea or Girl's Mother unavailable")?;
    ensure(agnea_level>=d.mother.required_level,"Agnea does not meet Empowering Lychee (M) Entreat level")?;
    l.draw(world::DrawKind::PathActionMenu,&d.mother.npc,"Girl's Mother Entreat: opening action timer",None);
    *l.state.route_items.entry(d.mother.item.clone()).or_default()+=1;
    l.boundary("Empowering Lychee (M) entreated; change selected path action, then close without dialogue or closing lottery");
    let mother_entreat=point(&mut l,tavern_entreat.after);
    ensure(present(&l,&d.magic_merchant.npc),"Wellgrove Mysterious Merchant unavailable")?;
    ensure(agnea_level>=d.magic_merchant.required_level,"Agnea does not meet Magic Nut (L) Entreat level")?;
    l.draw(world::DrawKind::PathActionMenu,&d.magic_merchant.npc,"Mysterious Merchant Magic Nut (L) Entreat: opening action timer",None);
    *l.state.route_items.entry(d.magic_merchant.item.clone()).or_default()+=1;
    l.boundary("Magic Nut (L) entreated from Mysterious Merchant north of tavern; change selected path action, then close without dialogue or closing lottery");
    let magic_entreat=point(&mut l,mother_entreat.after);
    ensure(present(&l,&d.guild_resource),"Dancer guild unavailable")?;
    for event in &d.guild_events {l.skipped_event(event)?;}
    l.state.job_licenses.entry(d.dancer_job).or_insert(1);
    l.boundary("First Dancer License acquired; no job assignment or skill purchases");
    let dancer_license=point(&mut l,magic_entreat.after);
    let al_return=crate::galdera_al_return::finish(&dancer_license)?;
    Ok(Trip {pom_steal,pom_steal_rolls,pom_steal_chance:100,throne_level,night,tavern_entreat,mother_entreat,magic_entreat,dancer_license,tavern_required_levels:d.shopkeeper.items.iter().map(|i|i.required_level).collect(),agnea_level,required_level:d.mother.required_level,al_return})
}
