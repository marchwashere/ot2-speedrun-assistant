//! Source-audited Gravell Optimize result and carried island equipment.
use crate::{Result,ensure,world::{WorldState,Loader}};
use std::collections::BTreeMap;
use serde::Deserialize;
#[derive(Deserialize)]struct Item {id:String,#[serde(rename="EquipmentRevision")] stats:BTreeMap<String,i64>,#[serde(rename="Ailment")] effects:Vec<Effect>}
#[derive(Deserialize)]struct Effect {#[serde(rename="AilmentName")] name:String,#[serde(rename="InvocationValue")] value:u32}
#[derive(Deserialize)]struct Data {equipment:BTreeMap<u32,BTreeMap<String,String>>,defaults:BTreeMap<u32,BTreeMap<String,String>>,items:Vec<Item>}
fn take(w:&mut WorldState,item:&str)->Result<()> {let n=w.route_items.get_mut(item).ok_or_else(||crate::SimError(format!("Nameless entry is missing {item}")))?;ensure(*n>0,format!("Nameless entry is missing {item}"))?;*n-=1;if *n==0{w.route_items.remove(item);}Ok(())}
/// Optimize after the restored weapons are sold, then manually equip the
/// accessory. CategorizeEquipmentItems excludes others' equipped gear; with
/// this route's inventory only Firestarter, the spear and Skull Helm improve
/// Osvald's slots. His Stick/clothes stay equipped and accessories are skipped.
pub(crate) fn equip_gravell_osvald(l:&mut Loader<'_>)->Result<()> {
 ensure(l.state.map=="Twn_Wld_3_1_A","Osvald Optimize belongs in Gravell")?;
 ensure(l.state.character_progression[&6].jobs.secondary_job==Some(2),"Osvald must be Warrior")?;
 let d:Data=serde_json::from_str(include_str!("../data/galdera-nameless.json"))?;
 let mut w=l.state.clone();
 for (slot,item) in &d.equipment[&6] {equip(&mut w,&d,6,slot,item)?;}
 l.state=w;
 l.boundary("After restored-weapon sales: Optimize Osvald (Firestarter, Scorched Bone Spear, Skull Helm), manually equip Elemental Augmentor; keep Stick and Prisoner's Clothes. No Magic nuts used");
 Ok(())
}
pub fn apply(l:&mut Loader<'_>)->Result<()> {
 ensure(l.state.map==crate::galdera_nameless::MAP||l.state.map==crate::galdera_shipwreck::MAP,"Equipment menu belongs at Nameless Isle or Shipwreck entry")?;
 ensure(l.state.character_progression[&6].jobs.secondary_job==Some(2),"Osvald must be Warrior")?;
 ensure(l.state.character_progression[&7].jobs.secondary_job==Some(11),"Throné must retain Inventor")?;
 let d:Data=serde_json::from_str(include_str!("../data/galdera-nameless.json"))?;
 if d.equipment.iter().all(|(id,gear)|l.state.character_progression[id].jobs.equipped_route_items==*gear){return Ok(());}
 ensure(d.equipment.iter().all(|(id,gear)|l.state.character_progression[id].jobs.equipped_route_items.iter().all(|(slot,item)|gear.get(slot)==Some(item))),"Unrecognized preexisting route equipment")?;
 let mut w=l.state.clone();
 for (&id,gear) in &d.equipment {
  for (slot,item) in gear {
   equip(&mut w,&d,id,slot,item)?;
  }
 }
 l.state=w;
 l.boundary("Island entry: retain earlier equipment, jobs and supports; neither Osvald nor Throné uses nuts during manipulation");
 Ok(())
}

fn equip(w:&mut WorldState,d:&Data,id:u32,slot:&str,item:&str)->Result<()> {
 if w.character_progression[&id].jobs.equipped_route_items.get(slot).is_some_and(|old|old==item){return Ok(());}
 ensure(!w.character_progression[&id].jobs.equipped_route_items.contains_key(slot),"Unexpected equipment in route slot")?;
   take(w,item)?;
   let row=d.items.iter().find(|r|r.id==item).unwrap();
   let old=&d.defaults[&id][slot];
   let previous=d.items.iter().find(|r|r.id==*old);
   let p=w.character_progression.get_mut(&id).unwrap();let c=p.character.combat.as_mut().unwrap();
   if let Some(weapon)=match slot{"Sword"=>Some(ot2_battle_engine::combat::Weapon::Sword),"Lance"=>Some(ot2_battle_engine::combat::Weapon::Lance),_=>None}{
    let equipped=c.weapons.iter_mut().find(|v|v.weapon==weapon).ok_or_else(||crate::SimError("Weapon slot unavailable".into()))?;
    equipped.attack=row.stats["ATK"] as u32;equipped.magic=row.stats["MATK"] as u32;
    c.fire_damage_bonus+=row.effects.iter().filter(|e|e.name=="FIRE_ABILITY_UP").map(|e|e.value-100).sum::<u32>();
   }else{
    for (stat,value) in [("HP",&mut p.character.stats.max_hp),("MP",&mut p.character.stats.max_sp),("AGI",&mut p.character.stats.speed),("ATK",&mut c.attack),("DEF",&mut c.defense),("MATK",&mut c.magic),("MDEF",&mut c.magic_defense),("ACC",&mut c.accuracy),("EVA",&mut c.evasion),("CON",&mut c.critical)]{
     *value=u32::try_from(i64::from(*value)+row.stats[stat]-previous.map_or(0,|r|r.stats[stat])).map_err(|_|crate::SimError("Invalid equipment stat".into()))?;
    }
   }
   p.jobs.equipped_route_items.insert(slot.into(),item.into());
   if old!="None" {*w.route_items.entry(old.clone()).or_default()+=1;}
 Ok(())
}
/// Accept the shop's equip-now prompt, using the same source-derived stat deltas.
pub(crate) fn equip_roque_purchase(l:&mut Loader<'_>,item:&str)->Result<()> {
 ensure(l.state.map=="Twn_Sea_3_1_A","Roque purchase equip belongs in the shop")?;
 let d:Data=serde_json::from_str(include_str!("../data/galdera-nameless.json"))?;
 let slot=d.equipment[&7].iter().find(|(_,id)|id.as_str()==item).map(|(slot,_)|slot).ok_or_else(||crate::SimError("Unexpected Roque equipment purchase".into()))?;
 let mut w=l.state.clone();equip(&mut w,&d,7,slot,item)?;l.state=w;
 l.boundary(format!("Equip {item} to Throne directly from purchase prompt"));Ok(())
}

/// Historical snapshots predate the post-Temenos Scholar menu. Upgrade their
/// explicit skill ledger in tests; never infer or grant supports in production.
#[cfg(test)]
pub(crate) fn track_augmentation_for_legacy_fixture(w:&mut WorldState) {
 w.character_progression.get_mut(&6).unwrap().jobs.learned_abilities.insert(6,
  ["ABI_SET_SCH_010","ABI_SET_SCH_020","ABI_SET_SCH_060","ABI_SET_SCH_070"].into_iter().map(str::to_owned).collect());
 crate::gravell_setup::learn(w,6,6,&["ABI_SET_SCH_030"]).unwrap();
 crate::scholar::equip_elemental_augmentation(w,6).unwrap();
}
