//! Explicit, transactional Nameless entry menu. Never Optimize or fabricate loot.
use crate::{Result,ensure,world::{WorldState,Loader}};
use std::collections::BTreeMap;
use serde::Deserialize;
#[derive(Deserialize)]struct Item {id:String,#[serde(rename="EquipmentRevision")] stats:BTreeMap<String,i64>,#[serde(rename="Ailment")] effects:Vec<Effect>}
#[derive(Deserialize)]struct Effect {#[serde(rename="AilmentName")] name:String,#[serde(rename="InvocationValue")] value:u32}
#[derive(Deserialize)]struct Data {equipment:BTreeMap<u32,BTreeMap<String,String>>,nuts:BTreeMap<u32,BTreeMap<String,u32>>,defaults:BTreeMap<u32,BTreeMap<String,String>>,items:Vec<Item>}
fn take(w:&mut WorldState,item:&str)->Result<()> {let n=w.route_items.get_mut(item).ok_or_else(||crate::SimError(format!("Nameless entry is missing {item}")))?;ensure(*n>0,format!("Nameless entry is missing {item}"))?;*n-=1;if *n==0{w.route_items.remove(item);}Ok(())}
pub fn apply(l:&mut Loader<'_>)->Result<()> {
 ensure(l.state.map==crate::galdera_nameless::MAP||l.state.map==crate::galdera_shipwreck::MAP,"Equipment menu belongs at Nameless Isle or Shipwreck entry")?;
 ensure(l.state.character_progression[&6].jobs.secondary_job==Some(2),"Osvald must be Warrior")?;
 ensure(l.state.character_progression[&7].jobs.secondary_job==Some(11),"Throné must retain Inventor")?;
 let d:Data=serde_json::from_str(include_str!("../data/galdera-nameless.json"))?;
 if d.equipment.iter().all(|(id,gear)|l.state.character_progression[id].jobs.equipped_route_items==*gear){return Ok(());}
 ensure([6,7].iter().all(|id|l.state.character_progression[id].jobs.equipped_route_items.is_empty()),"Unrecognized preexisting route equipment")?;
 let mut w=l.state.clone();
 for (&id,gear) in &d.equipment {
  for (slot,item) in gear {
   take(&mut w,item)?;
   let row=d.items.iter().find(|r|r.id==*item).unwrap();
   let old=&d.defaults[&id][slot];
   let previous=d.items.iter().find(|r|r.id==*old);
   let p=w.character_progression.get_mut(&id).unwrap();let c=p.character.combat.as_mut().unwrap();
   if let Some(weapon)=match slot.as_str(){"Sword"=>Some(ot2_battle_engine::combat::Weapon::Sword),"Lance"=>Some(ot2_battle_engine::combat::Weapon::Lance),_=>None}{
    let equipped=c.weapons.iter_mut().find(|v|v.weapon==weapon).ok_or_else(||crate::SimError("Weapon slot unavailable".into()))?;
    equipped.attack=row.stats["ATK"] as u32;equipped.magic=row.stats["MATK"] as u32;
    c.fire_damage_bonus+=row.effects.iter().filter(|e|e.name=="FIRE_ABILITY_UP").map(|e|e.value-100).sum::<u32>();
   }else{
    for (stat,value) in [("HP",&mut p.character.stats.max_hp),("MP",&mut p.character.stats.max_sp),("AGI",&mut p.character.stats.speed),("ATK",&mut c.attack),("DEF",&mut c.defense),("MATK",&mut c.magic),("MDEF",&mut c.magic_defense),("ACC",&mut c.accuracy),("EVA",&mut c.evasion),("CON",&mut c.critical)]{
     *value=u32::try_from(i64::from(*value)+row.stats[stat]-previous.map_or(0,|r|r.stats[stat])).map_err(|_|crate::SimError("Invalid equipment stat".into()))?;
    }
   }
   p.jobs.equipped_route_items.insert(slot.clone(),item.clone());
   if old!="None" {*w.route_items.entry(old.clone()).or_default()+=1;}
  }
 }
 for (&id,nuts) in &d.nuts {for (item,&count) in nuts {for _ in 0..count{
  take(&mut w,item)?;let effect=&d.items.iter().find(|r|r.id==*item).unwrap().effects[0];
  let stat=effect.name.strip_prefix("GROW_").ok_or_else(||crate::SimError("Nut effect missing".into()))?;
  let p=w.character_progression.get_mut(&id).unwrap();
  ot2_battle_engine::jobs::add_nut(&mut p.character,id,p.level,p.jobs.secondary_job,stat,effect.value)?;
 }}}
 l.state=w;
 l.boundary("Island entry manual menu: Osvald equips Firestarter, Scorched Bone Spear, Skull Helm and Elemental Augmentor; uses Magic Nut L x2. Throné equips Swift Shield and Quick Cloak; uses Critical Nut L and Fortifying Nut L. Keep current jobs and supports; previously applied menu is not repeated");
 Ok(())
}
