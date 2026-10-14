//! Source-audited Warrior guild, Smooth Talker Hire and nighttime Entreats.
use crate::{Result,CrtRng,ensure,world::{self,Loader,HiredMerchant},snows_escape::Entry};
use serde::{Deserialize,Serialize};
use std::sync::OnceLock;
#[derive(Deserialize)]struct Data {warrior_job:u32,guild_flag:u32,peddler:Peddler,pickaxe:Pickaxe,augmentor:Augmentor,jam:Jam}
#[derive(Deserialize)]struct Jam {npc:String,item:String,proper_steal:u32,guaranteed_level:u32}
#[derive(Deserialize)]struct Augmentor {npc:String,item:String,required_level:u32}
#[derive(Deserialize)]struct Peddler {npc:String,unique_id:u32,supporter:String,talent:String,price:u32,events:Vec<String>}
#[derive(Deserialize)]struct Pickaxe {npc:String,item:String,required_level:u32}
fn data()->&'static Data {static D:OnceLock<Data>=OnceLock::new();D.get_or_init(||serde_json::from_str(include_str!("../data/galdera-oresrush.json")).expect("Audited Oresrush actions"))}
#[derive(Debug,Serialize)]
pub struct Supplies {pub warrior:Entry,pub oresrush:Entry,pub night:Entry,pub hire:Entry,pub pickaxe:Entry,pub pickaxe_required_level:u32,pub pickaxe_agnea_level:u32,pub hire_event:String,pub augmentor:AugmentorTrip,pub winterbloom:crate::galdera_winterbloom::Trip,pub wellgrove:crate::galdera_wellgrove::Trip}
#[derive(Debug,Serialize)]
pub struct AugmentorTrip {pub sai:Entry,pub skull_helm:Entry,pub east_district:Entry,pub entreat:Entry,pub day:Entry,pub jam_steal:Entry,pub required_level:u32,pub agnea_level:u32}
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
    ensure(l.state.time==4&&l.state.party.contains(&8)&&present(&l,&d.pickaxe.npc),"Agnea or nighttime Retired Miner is unavailable")?;
    let pickaxe_agnea_level=l.state.character_progression[&8].level;
    ensure(!l.state.route_items.contains_key(&d.pickaxe.item),"Sturdy Pickaxe was already obtained")?;
    ensure(pickaxe_agnea_level>=d.pickaxe.required_level,"Agnea does not meet Sturdy Pickaxe Entreat level")?;
    l.draw(world::DrawKind::PathActionMenu,&d.pickaxe.npc,"Entreat: PlayFieldCommandActions / K2_SetTimerDelegate",None);
    *l.state.route_items.entry(d.pickaxe.item.clone()).or_default()+=1;
    // User-specified closing behavior, matching the Conning Creek session:
    // change selected path action before closing; never execute DrawEventLabel.
    l.boundary("Sturdy Pickaxe entreated at night: change selected path action, then close without dialogue; no success check or closing lottery draw");
    let pickaxe=point(&mut l,hire.after);
    let augmentor=augmentor(&pickaxe)?;
    let winterbloom=crate::galdera_winterbloom::finish(&augmentor.jam_steal)?;
    let wellgrove=crate::galdera_wellgrove::finish(&winterbloom.soldier_entreat)?;
    Ok(Supplies {warrior,oresrush,night,hire,pickaxe,pickaxe_required_level:d.pickaxe.required_level,pickaxe_agnea_level,hire_event,augmentor,winterbloom,wellgrove})
}
fn augmentor(entry:&Entry)->Result<AugmentorTrip> {
    let d=&data().augmentor;
    let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
    l.visit_map("Twn_Dst_2_1_A")?;l.skipped_event("MapInStorySelect")?;
    let sai=point(&mut l,entry.after);
    // PurchaseItemTable NPC_Twn_Dst_2_1_A_TALK_1500_NPCBUY_02:
    // Skull Helm, ProperBeg36. Soldier has night-only placements.
    let soldier="NPC_Twn_Dst_2_1_A_TALK_1500";
    ensure(l.state.time==4&&l.state.party.contains(&8)&&present(&l,soldier),"Agnea or nighttime Skull Helm Soldier unavailable")?;
    ensure(l.state.character_progression[&8].level>=36,"Agnea does not meet Skull Helm Entreat level36")?;
    l.draw(world::DrawKind::PathActionMenu,soldier,"Skull Helm Entreat: PlayFieldCommandActions / K2_SetTimerDelegate",None);
    *l.state.route_items.entry("ITM_EQP_HLM_180".into()).or_default()+=1;
    l.boundary("Sai Soldier on road to East District: Entreat Skull Helm (Agnea36); change selected path action, then close without dialogue or closing lottery");
    let skull_helm=point(&mut l,sai.after);
    l.visit_map("Twn_Dst_2_1_B")?;l.skipped_event("MapInStorySelect")?;
    let east_district=point(&mut l,skull_helm.after);
    ensure(l.state.time==4&&l.state.party.contains(&8)&&present(&l,&d.npc),"Agnea or nighttime Elemental Augmentor Townsperson is unavailable")?;
    let agnea_level=l.state.character_progression[&8].level;
    ensure(agnea_level>=d.required_level,"Agnea does not meet Elemental Augmentor Entreat level")?;
    ensure(!l.state.route_items.contains_key(&d.item),"Elemental Augmentor already obtained")?;
    l.draw(world::DrawKind::PathActionMenu,&d.npc,"Entreat: PlayFieldCommandActions / K2_SetTimerDelegate",None);
    // BegItemExec checks level and grants the item; no probability lottery.
    *l.state.route_items.entry(d.item.clone()).or_default()+=1;
    l.boundary(format!("Agnea level{agnea_level} Entreat Elemental Augmentor (requires{}); change selected path action, then close without dialogue or closing lottery",d.required_level));
    let entreat=point(&mut l,east_district.after);
    l.manual_time_switch(2,"Sai East District: switch to day immediately after Elemental Augmentor Entreat")?;
    let day=point(&mut l,entreat.after);
    let jam=&data().jam;
    ensure(l.state.party.contains(&7)&&present(&l,&jam.npc),"Throné or daytime Sai Jam Townsperson unavailable")?;
    ensure(l.state.character_progression[&7].level>=jam.guaranteed_level,"Sai Reinforcing Jam Steal is not guaranteed")?;
    l.draw(world::DrawKind::PathActionMenu,&jam.npc,"Open Townsperson Reinforcing Jam Steal menu: action timer",None);
    let roll=l.draw(world::DrawKind::StealCheck,&jam.item,format!("Reinforcing Jam: ProperSteal{}, guaranteed level{}, probability100%",jam.proper_steal,jam.guaranteed_level),Some(100))+1;
    if let Some(t)=l.trace.last_mut(){t.value=Some(roll);}
    *l.state.route_items.entry(jam.item.clone()).or_default()+=1;
    l.boundary("Reinforcing Jam stolen outside Sai East District orphanage; change selected path action, then close without dialogue or closing lottery");
    let jam_steal=point(&mut l,day.after);
    Ok(AugmentorTrip {sai,skull_helm,east_district,entreat,day,jam_steal,required_level:d.required_level,agnea_level})
}
