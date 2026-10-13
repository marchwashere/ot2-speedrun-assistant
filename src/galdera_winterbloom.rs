//! Pickaxe delivery -> Rusty Sword reward -> nighttime Soldier Entreat.
use crate::{Result,CrtRng,ensure,world::{self,Loader,EventChoices},snows_escape::Entry};
use serde::{Deserialize,Serialize};
use std::sync::OnceLock;
#[derive(Deserialize)]struct Data {event:String,quest_npc:String,pickaxe:String,sword:String,jam:String,reward_money:u32,soldier:Soldier}
#[derive(Deserialize)]struct Soldier {npc:String,item:String,required_level:u32}
fn data()->&'static Data {static D:OnceLock<Data>=OnceLock::new();D.get_or_init(||serde_json::from_str(include_str!("../data/galdera-winterbloom.json")).unwrap())}
#[derive(Debug,Serialize)]pub struct Trip {pub arrival:Entry,pub sword_quest:Entry,pub soldier_entreat:Entry,pub agnea_level:u32,pub required_level:u32,pub reward_money:u32}
fn point(l:&mut Loader<'_>,arrival:CrtRng)->Entry {Entry {arrival,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}}
fn present(l:&Loader<'_>,npc:&str)->bool {l.state.existing_npcs.iter().any(|id|l.data.placements[id].resource==npc)}
pub(crate) fn finish(entry:&Entry)->Result<Trip> {
    let d=data();let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
    ensure(l.state.time==4,"Winterbloom supplies continue at night after Sai Entreat")?;
    l.visit_map("Twn_Snw_2_1_A")?;l.skipped_event("MapInStorySelect")?;
    let arrival=point(&mut l,entry.after);
    ensure(present(&l,&d.quest_npc)&&!l.state.flag(20393),"Rusty Sword quest is unavailable or already completed")?;
    ensure(l.state.party.len()+usize::from(l.state.hired_merchant.is_some())<8,"Unsupported eight-member alternative Rusty Sword event")?;
    ensure(l.state.route_items.get(&d.pickaxe).copied().unwrap_or(0)>0,"Rusty Sword delivery needs Sturdy Pickaxe")?;
    l.skipped_event_with_choices(&d.event,&EventChoices::from([((d.event.clone(),8),"eDIALOG_YES".into())]))?;
    ensure(l.state.flag(20393)&&l.state.route_items.contains_key(&d.sword)&&l.state.route_items.contains_key(&d.jam),"Rusty Sword quest rewards were not granted")?;
    let sword_quest=point(&mut l,arrival.after);
    ensure(l.state.time==4&&l.state.party.contains(&8)&&present(&l,&d.soldier.npc),"Agnea or nighttime Soldier is unavailable")?;
    let agnea_level=l.state.character_progression[&8].level;
    ensure(agnea_level>=d.soldier.required_level,"Agnea level is too low for Fortifying Nut L")?;
    l.draw(world::DrawKind::PathActionMenu,&d.soldier.npc,"Entreat Soldier: PlayFieldCommandActions / K2_SetTimerDelegate",None);
    *l.state.route_items.entry(d.soldier.item.clone()).or_default()+=1;
    l.boundary("Fortifying Nut (L) entreated: change selected path action, then close without dialogue or closing lottery");
    let soldier_entreat=point(&mut l,sword_quest.after);
    Ok(Trip {arrival,sword_quest,soldier_entreat,agnea_level,required_level:d.soldier.required_level,reward_money:d.reward_money})
}
