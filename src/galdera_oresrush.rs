//! Source-audited Warrior guild, Smooth Talker Hire and Sturdy Pickaxe Steal.
use crate::{Result,CrtRng,ensure,world::{self,Loader,HiredMerchant},snows_escape::Entry};
use serde::{Deserialize,Serialize};
use std::sync::OnceLock;
#[derive(Deserialize)]struct Data {warrior_job:u32,guild_flag:u32,peddler:Peddler,pickaxe:Pickaxe,augmentor:Augmentor}
#[derive(Deserialize)]struct Augmentor {npc:String,item:String,required_level:u32}
#[derive(Deserialize)]struct Peddler {npc:String,unique_id:u32,supporter:String,talent:String,price:u32,events:Vec<String>}
#[derive(Deserialize)]struct Pickaxe {npc:String,item:String,proper_steal:u32,guaranteed_level:u32}
fn data()->&'static Data {static D:OnceLock<Data>=OnceLock::new();D.get_or_init(||serde_json::from_str(include_str!("../data/galdera-oresrush.json")).expect("Audited Oresrush actions"))}
#[derive(Debug,Serialize)]
pub struct Supplies {pub warrior:Entry,pub oresrush:Entry,pub night:Entry,pub hire:Entry,pub day:Entry,pub pickaxe:Entry,pub steal_chance:u32,pub steal_roll:u32,pub hire_event:String,#[serde(skip_serializing_if="Option::is_none")]pub steal_event:Option<String>,pub augmentor:AugmentorTrip,pub winterbloom:crate::galdera_winterbloom::Trip,pub wellgrove:crate::galdera_wellgrove::Trip}
#[derive(Debug,Serialize)]
pub struct AugmentorTrip {pub sai:Entry,pub east_district:Entry,pub night:Entry,pub entreat:Entry,pub required_level:u32,pub agnea_level:u32}
fn point(l:&mut Loader<'_>,arrival:CrtRng)->Entry {Entry {arrival,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}}
fn present(l:&Loader<'_>,npc:&str)->bool {l.state.existing_npcs.iter().any(|id|l.data.placements[id].resource==npc)}
fn dialogue(l:&mut Loader<'_>,npc:&str,events:&[String])->Result<String> {
    // BP_FCv2_FieldCommandFlowBase.DrawEventLabel: weights0.5/0.5,
    // RandomFloatInRange then <= cumulative weight. Native15-bit unit float.
    ensure(events.len()==2,"Unsupported dialogue lottery")?;
    let raw=l.draw(world::DrawKind::DialogueLottery,npc,"DrawEventLabel: success dialogue lottery (0.5 / 0.5)",None);
    let event=events[usize::from(raw>16383)].clone();l.skipped_event(&event)?;Ok(event)
}
pub(crate) fn finish(sai:&Entry)->Result<Supplies> {
    let d=data();let mut l=Loader::new(world::catalog(),sai.world_after.clone(),sai.after)?;
    ensure(l.state.map=="Twn_Dst_2_1_A"&&l.state.time==2,"Warrior License starts at daytime Sai")?;
    l.skipped_event("SYS_WAR_GUILD_0010")?;l.skipped_event("SYS_WAR_GUILD_0030")?;
    l.state.job_licenses.entry(d.warrior_job).or_insert(1);
    ensure(l.state.flag(d.guild_flag),"Warrior guild completion flag was not set")?;
    l.boundary("Warrior License acquired; no job assignment or skill purchases");
    let warrior=point(&mut l,sai.after);
    l.visit_map(crate::oresrush::TOWN)?;l.skipped_event("MapInStorySelect")?;
    let oresrush=point(&mut l,warrior.after);
    l.manual_time_switch(4,"Oresrush: switch to night to hire Smooth Talker Peddler")?;
    let night=point(&mut l,oresrush.after);
    ensure(l.state.party.contains(&4)&&present(&l,&d.peddler.npc),"Partitio or Smooth Talker Peddler is unavailable")?;
    ensure(l.state.hired_merchant.is_none(),"This hire profile requires no previous hired NPC")?;
    l.draw(world::DrawKind::PathActionMenu,&d.peddler.npc,"Hire: PlayFieldCommandActions / K2_SetTimerDelegate",None);
    let hire_event=dialogue(&mut l,&d.peddler.npc,&d.peddler.events)?;
    l.state.hired_merchant=Some(HiredMerchant {npc:d.peddler.npc.clone(),unique_id:d.peddler.unique_id,supporter:d.peddler.supporter.clone(),talent:d.peddler.talent.clone(),price:d.peddler.price});
    l.state.existing_npcs.retain(|id|l.data.placements[id].resource!=d.peddler.npc);
    l.boundary(format!("Hire Smooth Talker Peddler for {} leaves; register follower and suppress original placement. RefreshFollowCharacter reuses party follower actors",d.peddler.price));
    let hire=point(&mut l,night.after);
    l.manual_time_switch(2,"Oresrush: switch to day after hiring Peddler")?;let day=point(&mut l,hire.after);
    ensure(present(&l,&d.pickaxe.npc),"Retired Miner is unavailable")?;
    let level=l.state.character_progression[&7].level;
    ensure(!l.state.route_items.contains_key(&d.pickaxe.item),"Sturdy Pickaxe was already obtained")?;
    ensure(level>=d.pickaxe.guaranteed_level,"Pickaxe steal is not guaranteed at the carried Throné level")?;
    l.draw(world::DrawKind::PathActionMenu,&d.pickaxe.npc,"Steal: PlayFieldCommandActions / K2_SetTimerDelegate",None);
    let steal_roll=l.draw(world::DrawKind::StealCheck,&d.pickaxe.item,format!("Sturdy Pickaxe: ProperSteal{}, level{level}, probability100%",d.pickaxe.proper_steal),Some(100))+1;
    *l.state.route_items.entry(d.pickaxe.item.clone()).or_default()+=1;
    // User-specified closing behavior, matching the Conning Creek session:
    // change selected path action before closing; never execute DrawEventLabel.
    let steal_event=None;
    l.boundary("Sturdy Pickaxe stolen: change selected path action, then close without Steal dialogue; no closing lottery draw");
    let pickaxe=point(&mut l,day.after);
    let augmentor=augmentor(&pickaxe)?;
    let winterbloom=crate::galdera_winterbloom::finish(&augmentor.entreat)?;
    let wellgrove=crate::galdera_wellgrove::finish(&winterbloom.soldier_entreat)?;
    Ok(Supplies {warrior,oresrush,night,hire,day,pickaxe,steal_chance:100,steal_roll,hire_event,steal_event,augmentor,winterbloom,wellgrove})
}
fn augmentor(entry:&Entry)->Result<AugmentorTrip> {
    let d=&data().augmentor;
    let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
    l.visit_map("Twn_Dst_2_1_A")?;l.skipped_event("MapInStorySelect")?;
    let sai=point(&mut l,entry.after);
    l.visit_map("Twn_Dst_2_1_B")?;l.skipped_event("MapInStorySelect")?;
    let east_district=point(&mut l,sai.after);
    l.manual_time_switch(4,"Sai: East District: switch to night for Elemental Augmentor")?;
    let night=point(&mut l,east_district.after);
    ensure(l.state.party.contains(&8)&&present(&l,&d.npc),"Agnea or Elemental Augmentor Townsperson is unavailable")?;
    let agnea_level=l.state.character_progression[&8].level;
    ensure(agnea_level>=d.required_level,"Agnea does not meet Elemental Augmentor Entreat level")?;
    ensure(!l.state.route_items.contains_key(&d.item),"Elemental Augmentor already obtained")?;
    l.draw(world::DrawKind::PathActionMenu,&d.npc,"Entreat: PlayFieldCommandActions / K2_SetTimerDelegate",None);
    // BegItemExec checks level and grants the item; no probability lottery.
    *l.state.route_items.entry(d.item.clone()).or_default()+=1;
    l.boundary(format!("Agnea level{agnea_level} Entreat Elemental Augmentor (requires{}); change selected path action, then close without dialogue or closing lottery",d.required_level));
    let entreat=point(&mut l,night.after);
    Ok(AugmentorTrip {sai,east_district,night,entreat,required_level:d.required_level,agnea_level})
}
