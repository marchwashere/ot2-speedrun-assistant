//! Daytime inn supplies Steal -> night -> Magic Nut Entreat -> Dancer License.
use crate::{Result,CrtRng,ensure,world::{self,Loader},snows_escape::Entry};
use serde::{Deserialize,Serialize};
use std::sync::OnceLock;
#[derive(Deserialize)]struct Data {shopkeeper:Shopkeeper,magic_merchant:Entreat,dancer_job:u32,guild_resource:String,guild_events:Vec<String>}
#[derive(Deserialize)]struct Shopkeeper {npc:String,items:Vec<StealItem>}
#[derive(Deserialize)]struct StealItem {item:String,name:String,proper_steal:u32,guaranteed_level:u32}
#[derive(Deserialize)]struct Entreat {npc:String,item:String,required_level:u32}
fn data()->&'static Data {static D:OnceLock<Data>=OnceLock::new();D.get_or_init(||serde_json::from_str(include_str!("../data/galdera-wellgrove-supplies.json")).unwrap())}
#[derive(Debug,Serialize)]pub struct Trip {
    pub throne_level:u32,
    pub shopkeeper_steal:Entry,pub night:Entry,pub magic_entreat:Entry,pub day:Entry,pub dancer_license:Entry,
    pub agnea_level:u32,pub required_level:u32,
    pub al_return:crate::galdera_al_return::Trip,
}
fn point(l:&mut Loader<'_>,before:CrtRng)->Entry {
    Entry {arrival:before,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
fn present(l:&Loader<'_>,npc:&str)->bool {l.state.existing_npcs.iter().any(|id|l.data.placements[id].resource==npc)}
fn steal_inn_supplies(l:&mut Loader<'_>)->Result<()> {
    let d=&data().shopkeeper;
    ensure(l.state.map=="Twn_Fst_2_1_A"&&l.state.time==2&&l.state.party.contains(&7)&&present(l,&d.npc),"Daytime Wellgrove Shopkeeper or Throné unavailable")?;
    let level=l.state.character_progression[&7].level;
    ensure(d.items.iter().all(|item|level>=item.guaranteed_level),"Wellgrove Sharp Nut (L) and Diffusing Serum steals are not guaranteed")?;
    l.draw(world::DrawKind::PathActionMenu,&d.npc,"Inn Shopkeeper: open Steal menu",None);
    for item in &d.items {
        let roll=l.draw(world::DrawKind::StealCheck,&item.item,format!("{}: ProperSteal{}, Throné level{level}, probability100%",item.name,item.proper_steal),Some(100))+1;
        if let Some(t)=l.trace.last_mut(){t.value=Some(roll);}
        *l.state.route_items.entry(item.item.clone()).or_default()+=1;
    }
    l.boundary("Wellgrove inn: steal Sharp Nut (L), then Diffusing Serum in one menu; switch path action and close without dialogue or closing lottery");
    Ok(())
}
pub(crate) fn finish(entry:&Entry)->Result<Trip> {
    let d=data();let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
    ensure(l.state.map=="Twn_Fst_2_1_A"&&l.state.time==2,"Wellgrove supplies begin at daytime town entry")?;
    let throne_level=l.state.character_progression[&7].level;
    let agnea_level=l.state.character_progression[&8].level;
    steal_inn_supplies(&mut l)?;
    let shopkeeper_steal=point(&mut l,entry.after);
    l.manual_time_switch(4,"Wellgrove: switch to night after Sharp Nut (L) and Diffusing Serum steals")?;
    let night=point(&mut l,shopkeeper_steal.after);
    ensure(present(&l,&d.magic_merchant.npc),"Wellgrove Mysterious Merchant unavailable")?;
    ensure(agnea_level>=d.magic_merchant.required_level,"Agnea does not meet Magic Nut (L) Entreat level")?;
    l.draw(world::DrawKind::PathActionMenu,&d.magic_merchant.npc,"Mysterious Merchant Magic Nut (L) Entreat: opening action timer",None);
    *l.state.route_items.entry(d.magic_merchant.item.clone()).or_default()+=1;
    l.boundary("Magic Nut (L) entreated from Mysterious Merchant north of tavern; change selected path action, then close without dialogue or closing lottery");
    let magic_entreat=point(&mut l,night.after);
    l.manual_time_switch(2,"Wellgrove: switch to day immediately after Magic Nut (L) Entreat")?;
    let day=point(&mut l,magic_entreat.after);
    ensure(present(&l,&d.guild_resource),"Dancer guild unavailable")?;
    for event in &d.guild_events {l.skipped_event(event)?;}
    l.state.job_licenses.entry(d.dancer_job).or_insert(1);
    l.boundary("First Dancer License acquired; no job assignment or skill purchases");
    let dancer_license=point(&mut l,day.after);
    let al_return=crate::galdera_al_return::finish(&dancer_license)?;
    Ok(Trip {throne_level,shopkeeper_steal,night,magic_entreat,day,dancer_license,agnea_level,required_level:d.magic_merchant.required_level,al_return})
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]
    fn inn_steals_share_one_menu_and_skip_closing_dialogue() {
        let f:serde_json::Value=serde_json::from_str(include_str!("../data/source/gravell-complete-route-fixture.json")).unwrap();
        let mut w:world::WorldState=serde_json::from_value(f["before"]["world_after"].clone()).unwrap();
        w.time=2;w.character_progression.get_mut(&7).unwrap().level=33;
        for item in ["ITM_CSM_0670","ITM_MRL_COM_0010","ITM_CSM_0230","ITM_CSM_0240"] {w.route_items.remove(item);}
        let mut l=Loader::new(world::catalog(),w,CrtRng::new(0)).unwrap();
        l.visit_map("Twn_Fst_2_1_A").unwrap();l.trace.clear();
        let before=l.rng;let counter=l.state.encounter_counter;
        steal_inn_supplies(&mut l).unwrap();
        let mut expected=before;expected.advance(3);assert_eq!(l.rng,expected);
        assert_eq!(l.state.encounter_counter,counter);assert_eq!(l.state.time,2);
        assert_eq!(l.trace.iter().filter(|t|matches!(t.kind,world::DrawKind::PathActionMenu)).count(),1);
        let items:Vec<_>=l.trace.iter().filter(|t|matches!(t.kind,world::DrawKind::StealCheck)).map(|t|t.subject.as_str()).collect();
        assert_eq!(items,["ITM_CSM_0670","ITM_MRL_COM_0010"]);
        assert!(!l.trace.iter().any(|t|matches!(t.kind,world::DrawKind::DialogueLottery)));
        for item in items {assert_eq!(l.state.route_items.get(item),Some(&1));}
        for item in ["ITM_CSM_0230","ITM_CSM_0240"] {assert!(!l.state.route_items.contains_key(item));}
    }
}
