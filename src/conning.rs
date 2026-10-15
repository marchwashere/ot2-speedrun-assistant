//! Two first night coast encounters: exactly one SSM clear and one Escape.
use crate::{Result,CrtRng,ensure,SimError,encounter::{self,Boundary,PredictionOptions},
    manipulation::{BoostInputs,describe_inputs},snows_escape::Entry,world::{self,Loader,WorldState}};
use ot2_battle_engine::{escape::Situation,field_escape::{FieldEscapeMenu,FieldEscapeAttempt,FieldRareVictory}};
use serde::Serialize;
pub const NORTH:&str="Fld_Sea_2_2";
pub const WEST:&str="Fld_Sea_2_3";
pub const TOWN:&str="Twn_Sea_2_1_A";
pub const HARBOR:&str="Twn_Sea_2_1_B";
/// Finalization boundary: earlier search probes retain their bounded endpoint.
/// Only fully selected route alternatives pay for the coast extension. All
/// calculations remain typed library functions; JSON is the application's tree.
pub fn extend_selected_routes(value:&mut serde_json::Value)->Result<()> {
    if value["cropdale_exit"].is_object()&&value["destination"].is_object()&&value["cropdale_reach"].is_object() {
        let point=&value["destination"];
        let rng=serde_json::from_value(point["after"].clone())?;
        let entry=Entry {arrival:rng,after:rng,world_after:serde_json::from_value(point["world_after"].clone())?,trace:vec![],decisions:vec![]};
        let reach=serde_json::from_value(value["cropdale_reach"].clone())?;
        let (crop,mut exit)=crate::winter_voyage::search_oresrush(&entry,&reach)?;
        if let Some(ryu)=exit.oresrush.as_mut().and_then(|o|o.ryu.as_mut()) {
            if let Some(coasts)=ryu.canalbrine.as_mut().and_then(|c|c.conning.as_mut()) {
                let old=coasts.input_cost;
                search_pomegranate(coasts)?;
                if let Some(search)=ryu.hikari_search.as_mut() {
                    search.total_input_cost=search.total_input_cost-old+coasts.input_cost;
                }
            }
        }
        value["cropdale"]=serde_json::to_value(crop)?;
        value["cropdale_exit"]=serde_json::to_value(exit)?;
        return Ok(());
    }
    match value {
        serde_json::Value::Object(rows)=>for v in rows.values_mut() {extend_selected_routes(v)?;},
        serde_json::Value::Array(rows)=>for v in rows {extend_selected_routes(v)?;},
        _=>{}
    }
    Ok(())
}
#[derive(Debug,Clone,Serialize)]
pub struct Arrival {pub entry:Entry,pub night:Entry,pub encounter:encounter::Prediction,pub battle_entry:CrtRng}
#[derive(Debug,Serialize)]
pub struct Fight {
    pub defends:Vec<ot2_battle_engine::StepReport>,
    pub arrival:Arrival,pub menu:FieldEscapeMenu,pub inputs:BoostInputs,
    pub escape:Option<FieldEscapeAttempt>,pub victory:Option<FieldRareVictory>,
    pub field_return:Entry,pub day:Entry,
    #[serde(skip_serializing_if="Option::is_none")] pub leaf_steal:Option<Entry>,
}
impl Fight {fn departure(&self)->&Entry {self.leaf_steal.as_ref().unwrap_or(&self.day)}}
#[derive(Debug,Serialize)]
pub struct Prediction {pub northern:Fight,pub western:Fight,pub town:Entry,pub harbor:Entry,pub input_cost:u32,pub verification:&'static str,
    pub pomegranate:Option<Pomegranate>}
#[derive(Debug,Serialize)]
pub struct Pomegranate {
    pub town_switches:Vec<Entry>,pub harbor_switches:Vec<Entry>,pub steal:Entry,
    pub throne_level:u32,pub chance:u32,pub roll:u32,pub input_cost:u32,
    pub item:&'static str,pub quantity:u32,pub wind_chance:u32,pub wind_roll:u32,
    pub voyage:Option<CanalbrineVoyage>,
    pub inventor:Option<crate::inventor::Prediction>,
}
#[derive(Debug,Serialize)]
pub struct CanalbrineVoyage {pub new_delsta:Entry,pub anchorage:Entry,pub canalbrine:Entry,pub castti:Entry}
fn canalbrine_voyage(entry:&Entry)->Result<CanalbrineVoyage> {
    ensure(entry.world_after.visited_maps.contains(crate::crackridge::NEW_DELSTA_HARBOR),"New Delsta Harbor must be tagged")?;
    let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
    l.visit_map(crate::crackridge::NEW_DELSTA_HARBOR)?;
    let new_delsta=checkpoint(&mut l,entry.after);
    l.skipped_event("SC_LinerShip_Cty_to_Isd")?;
    ensure(l.state.map=="Fld_Isd_1_3","Beasting Bay boat destination mismatch")?;
    let anchorage=checkpoint(&mut l,new_delsta.after);
    l.skipped_event("SC_LinerShip_Isd_to_Sea")?;
    ensure(l.state.map=="Twn_Sea_1_1_A","Canalbrine boat destination mismatch")?;
    let canalbrine=checkpoint(&mut l,anchorage.after);
    let character=ot2_battle_engine::field_escape::route_player(3,0)?;
    let growth=ot2_battle_engine::progression::growth(3,0)?.clone();
    l.state.character_progression.insert(3,world::CharacterProgression {experience:0,jp_earned:0,jp_spent:0,level:growth.level,growth,character,jobs:Default::default()});
    let event="MS_KUS_00_0100";
    l.skipped_event_with_choices(event,&world::EventChoices::from([
        ((event.into(),32),"eDIALOG_YES".into()),
        ((event.into(),126),"eDIALOG_YES".into()),
        ((event.into(),135),"eDIALOG_NO".into()),
    ]))?;
    ensure(l.state.bench.contains(&3) && l.state.party==canalbrine.world_after.party,"Castti recruitment changed active party")?;
    let castti=checkpoint(&mut l,canalbrine.after);
    Ok(CanalbrineVoyage {new_delsta,anchorage,canalbrine,castti})
}
fn pomegranate_chance(level:u32)->u32 {steal_chance(level,8)}
fn steal_chance(level:u32,proper:u32)->u32 {
    // FieldCommandWidgetSteal.StealItemExec; GameParam level-difference thresholds.
    match i64::from(level)-i64::from(proper) {d if d<=-11=>3,d if d<=-4=>8,d if d<0=>15,
        0|1=>55,2|3=>65,4..=10=>80,_=>100}
}
fn pomegranate_tail(w:&WorldState,rng:CrtRng,town_count:u32,harbor_count:u32)->Result<Option<(Entry,Entry,Pomegranate)>> {
    if (town_count+harbor_count)%2!=0 {return Ok(None);}
    let mut l=Loader::new(world::catalog(),w.clone(),rng)?;
    l.visit_map(TOWN)?;let town=checkpoint(&mut l,rng);
    let mut town_switches=vec![];let mut harbor_switches=vec![];
    for _ in 0..town_count {let before=l.rng;let time=if l.state.time==2 {4}else{2};l.manual_time_switch(time,"Conning Creek time switch")?;town_switches.push(checkpoint(&mut l,before));}
    let before=l.rng;l.visit_map(HARBOR)?;let harbor=checkpoint(&mut l,before);
    for _ in 0..harbor_count {let before=l.rng;let time=if l.state.time==2 {4}else{2};l.manual_time_switch(time,"Conning Creek: Harbor time switch")?;harbor_switches.push(checkpoint(&mut l,before));}
    ensure(l.state.time==2,"Steal requires daytime")?;
    ensure(l.state.existing_npcs.contains("NPC_Twn_Sea_2_1_B_TALK_0100_D000"),"Harbor Townsperson is not present")?;
    let level=l.state.character_progression[&7].level;let chance=pomegranate_chance(level);let before=l.rng;
    l.draw(world::DrawKind::PathActionMenu,"Townsperson","Open Steal menu: action timer",None);
    let roll=l.draw(world::DrawKind::StealCheck,"ITM_CSM_0080",format!("Energizing Pomegranate (M): level {level}, chance {chance}%"),Some(100))+1;
    if let Some(row)=l.trace.last_mut() {row.value=Some(roll);}
    if roll>chance {return Ok(None);}
    let wind_chance=chance; // Both source PurchaseItem rows have ProperSteal=8.
    let wind_roll=l.draw(world::DrawKind::StealCheck,"ITM_CSM_0400",format!("Wind Soulstone (L): level {level}, chance {wind_chance}%"),Some(100))+1;
    if let Some(row)=l.trace.last_mut() {row.value=Some(wind_roll);}
    if wind_roll>wind_chance {return Ok(None);}
    // User-observed route: selecting another path action suppresses the Steal closing voice.
    // This applies only to this two-item Townsperson session.
    l.boundary("Townsperson: change selected path action, then close without Steal voice");
    let steal=checkpoint(&mut l,before);
    Ok(Some((town,harbor,Pomegranate {town_switches,harbor_switches,steal,throne_level:level,chance,roll,input_cost:7*(town_count.saturating_sub(2)+harbor_count.saturating_sub(2)),item:"ITM_CSM_0080",quantity:1,wind_chance,wind_roll,voyage:None,inventor:None})))
}
/// Freeze earlier controls; search the Western battle and subsequent town switches.
fn search_pomegranate(p:&mut Prediction)->Result<()> {
    let arrival=p.western.arrival.clone();let kill=p.western.victory.is_some();
    for cost in 0..u32::MAX/2 {
        for battle_cost in 0..=cost {for (repeats,defends) in controls(battle_cost) {
            let paid=cost-battle_cost;if paid%7!=0 {continue;}
            let Some(western)=fight_defends(arrival.clone(),repeats,kill,defends)? else {continue;};
            let max=paid/7+4;
            for total in 0..=max {for town_count in 0..=total {
                let harbor_count=total-town_count;
                if 7*(town_count.saturating_sub(2)+harbor_count.saturating_sub(2))!=paid {continue;}
                if let Some((town,harbor,mut tail))=pomegranate_tail(&western.departure().world_after,western.departure().after,town_count,harbor_count)? {
                    tail.voyage=Some(canalbrine_voyage(&tail.steal)?);
                    let approach=crate::inventor::approach(&tail.voyage.as_ref().unwrap().castti)?;
                    // Galdera's New Delsta controls can also select a safe
                    // entry counter; do not reopen earlier coast controls for
                    // an unsafe zero-switch walking candidate.
                    if !approach.highroad.world_after.galdera_route && !crate::inventor::safe(&approach) {continue;}
                    tail.inventor=Some(crate::inventor::finish(approach)?);
                    tail.input_cost=cost;p.input_cost=p.input_cost-control_cost(&p.western)+cost;
                    p.western=western;p.town=town;p.harbor=harbor;p.pomegranate=Some(tail);return Ok(());
                }
            }}
        }}
    }
    Err(SimError("Pomegranate search exhausted representable input costs".into()))
}
pub fn path(n:u32)->Vec<u8> {(0..n).flat_map(|_|[1,0]).collect()}
fn checkpoint(l:&mut Loader<'_>,arrival:CrtRng)->Entry {
    Entry {arrival,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}
}
pub fn arrival(w:&WorldState,rng:CrtRng,map:&str)->Result<Arrival> {
    ensure(w.time==2,"Coast approach must begin during day")?;
    let mut l=Loader::new(world::catalog(),w.clone(),rng)?;l.visit_map(map)?;
    let entry=checkpoint(&mut l,rng);
    l.manual_time_switch(4,"Coast: switch to night just before first encounter")?;
    let night=checkpoint(&mut l,entry.after);
    let w=&night.world_after;let mut rng=night.after;
    let zone=if map==NORTH {"ENT_FLD_SEA_2_2_LND"}else{"ENT_FLD_SEA_2_3_LND"};
    let encounter=encounter::catalog().predict(&mut rng,&PredictionOptions {
        map:map.into(),zone:zone.into(),period:"Night".into(),progress:w.main_progress_points,
        first_encounter:w.first_encounter,boundary:Boundary::CurrentCounter,current_counter:Some(w.encounter_counter),
        party_levels:w.party.iter().map(|id|w.character_progression[id].level).collect(),
        ability_percent:if w.evasive_maneuvers {50}else{100},dash:true,..Default::default()
    })?;
    Ok(Arrival {entry,night,encounter,battle_entry:rng})
}
fn safe(a:&Arrival)->bool {matches!(a.encounter.situation.as_deref(),Some("Normal"|"Preemptive"))}
fn cait(a:&Arrival)->bool {a.encounter.enemies.iter().any(|s|s.enemy.id=="ENE_NML_OTR_140")}
/// Northern Defends and boosts control the Western opening. Take a natural Northern
/// Cait if present; otherwise require it in Western. Never permit an enemy action.
pub fn nearest(north:Arrival)->Result<Prediction> {
    nearest_bounded(north,None)?.ok_or_else(||SimError("Coast boost search exhausted".into()))
}
pub fn nearest_bounded(north:Arrival,limit:Option<u32>)->Result<Option<Prediction>> {
    ensure(safe(&north),"Northern opening must be protected by Canalbrine boosts")?;
    let mut node=Node {kill:cait(&north),arrival:north,first:Default::default(),failed:Default::default()};
    for cost in 0..=limit.unwrap_or(u32::MAX/2) {if let Some(p)=node_at_cost(&mut node,cost)? {return Ok(Some(p));}}
    Ok(None)
}

fn menu(a:&Arrival)->Result<FieldEscapeMenu> {
    let w=&a.night.world_after;
    let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
    let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
    let ids:Vec<_>=a.encounter.enemies.iter().map(|s|s.enemy.id.as_str()).collect();
    FieldEscapeMenu::with_players_defend(a.battle_entry,if a.encounter.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal},&players,&levels,&ids,true,&w.enemy_knowledge).map_err(Into::into)
}
#[cfg(test)]
fn fight(a:Arrival,n:u32,kill:bool)->Result<Option<Fight>> {fight_defends(a,n,kill,0)}
fn fight_defends(a:Arrival,n:u32,kill:bool,count:usize)->Result<Option<Fight>> {
    if !safe(&a) {return Ok(None);}
    let mut m=menu(&a)?;let mut defends=vec![];
    for _ in 0..count {let Some((next,report))=m.safe_first_round_defend()? else {return Ok(None);};m=next;defends.push(report);}
    let inputs=describe_inputs(n,0,path(n))?;
    let mut w=a.night.world_after.clone();
    let (escape,victory,after)=if kill {
        ensure(cait(&a),"Only the targeted Cultured Cait pack uses SSM")?;
        let players:Vec<_>=w.party.iter().map(|id|w.character_progression[id].character.clone()).collect();
        let levels:Vec<_>=w.party.iter().map(|id|w.character_progression[id].level).collect();
        let ids:Vec<_>=a.encounter.enemies.iter().map(|s|s.enemy.id.as_str()).collect();
        let situation=if a.encounter.situation.as_deref()==Some("Preemptive") {Situation::Preemptive}else{Situation::Normal};
        let v=ot2_battle_engine::field_escape::carried_soulstone_after_defends(a.battle_entry,situation,&players,&levels,&ids,true,&w.enemy_knowledge,&inputs.levels,"shadow_soulstone_m",false,count)?
            .ok_or_else(||SimError("Shadow Soulstone M cannot clear coast formation".into()))?;
        apply_item_victory(&mut w,&v)?;
        let after=v.after;(None,Some(v),after)
    } else {
        let e=m.clone().attempt(&inputs.levels,a.encounter.encounter_level.unwrap())?;
        if !e.success {return Ok(None);}
        w.enemy_knowledge=e.enemy_knowledge.clone();let after=e.after;(Some(e),None,after)
    };
    w.first_encounter=false;
    let mut l=Loader::new(world::catalog(),w,after)?;
    l.restore_field_after_battle("Conning coast: battle field returned")?;
    let field_return=checkpoint(&mut l,after);
    l.manual_time_switch(2,"Coast: switch back to day after encounter")?;
    let day=checkpoint(&mut l,field_return.after);
    let leaf_steal=if day.world_after.galdera_route&&day.world_after.map==WEST {
        // A failed steal rejects this boost candidate, not the whole route.
        let Some(steal)=steal_leaves(&day)? else {return Ok(None);};
        Some(steal)
    }else{None};
    Ok(Some(Fight {defends,arrival:a,menu:m,inputs,escape,victory,field_return,day,leaf_steal}))
}
fn steal_leaves(entry:&Entry)->Result<Option<Entry>> {
    let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
    let npc="NPC_SS_TSe21_0100_0300";
    ensure(l.state.map==WEST&&l.state.time==2,"Coast Merchant steals require daytime Western Coast")?;
    ensure(l.state.party.contains(&7)&&l.state.existing_npcs.iter().any(|id|l.data.placements[id].resource==npc),"Coast Merchant or Throné unavailable")?;
    let level=l.state.character_progression[&7].level;

    l.draw(world::DrawKind::PathActionMenu,npc,"Merchant: open one Steal menu (action timer)",None);
    for (item,name,proper) in [("ITM_MRL_REV_0030","Pomegranate Leaf",6),("ITM_MRL_REV_0080","Whimsical Leaf",4)] {
        let chance=steal_chance(level,proper);
        let roll=l.draw(world::DrawKind::StealCheck,item,format!("{name}: Throné Lv{level}, chance {chance}%"),Some(100))+1;
        if let Some(t)=l.trace.last_mut(){t.value=Some(roll);}
        if roll>chance {return Ok(None);}
        *l.state.route_items.entry(item.into()).or_default()+=1;
    }
    l.boundary("Merchant: after both steals, change selected path action, then close without dialogue/closing voice lottery");
    Ok(Some(checkpoint(&mut l,entry.after)))
}
pub(crate) fn apply_item_victory(w:&mut WorldState,v:&FieldRareVictory)->Result<()> {
        w.enemy_knowledge=v.enemy_knowledge.clone();
        let mut damage=std::collections::BTreeMap::new();let mut overkill=false;let mut broken=false;
        for e in &v.report.events {
            use ot2_battle_engine::{Event,Resource};
            match *e {
                Event::Broken{actor,..} if actor.0>=w.party.len()=>broken=true,
                Event::Damage{target,amount,..}=>{damage.insert(target,amount);},
                Event::ResourceChanged{actor,resource:Resource::Hp,before,after:0} if actor.0>=w.party.len()&&before>0=>{
                    overkill|=damage.get(&actor).is_some_and(|&d|u64::from(d)>=2*u64::from(before));
                },_=>{}
            }
        }
        for (&id,r) in w.party.iter().zip(&v.party_resources_after) {w.character_progression.get_mut(&id).unwrap().character.resources=Some(r.clone());}
        w.award_victory(ot2_battle_engine::progression::Reward {
            exp:v.base_exp+(u32::from(overkill)+u32::from(broken))*(v.base_exp/10).max(1),
            jp:v.base_jp+(v.base_jp/10).max(1),money:v.base_money+(v.base_money/10).max(1),
        })?;
    Ok(())
}
/// Exact cost layer: ordinary first-round Defends cost three, boost cycles two.
/// Compare both legal kill locations, including escaping a northern Cait when
/// killing a western Cait permits a cheaper complete plan.
fn controls(cost:u32)->impl Iterator<Item=(u32,usize)> {
    (0..=3usize).filter_map(move |d|cost.checked_sub(d as u32*crate::escape_search::DEFEND_INPUT_COST).filter(|c|c%2==0).map(|c|(c/2,d)))
}
fn control_cost(f:&Fight)->u32 {2*f.inputs.extra_repeats+f.defends.len()as u32*crate::escape_search::DEFEND_INPUT_COST}
struct Node {arrival:Arrival,kill:bool,first:std::collections::BTreeMap<(u32,usize),Option<(Fight,Arrival)>>,failed:std::collections::BTreeSet<u32>}
#[derive(Default)]
pub struct Cache {nodes:std::collections::BTreeMap<(crate::state_key::WorldKey,u32,u64,bool),Option<Node>>}
impl Cache {
    pub fn at_cost(&mut self,w:&WorldState,rng:CrtRng,cost:u32)->Result<Option<Prediction>> {
        for kill in [true,false] {
            let key=(crate::state_key::WorldKey::new(w),rng.state,rng.draws,kill);
            if !self.nodes.contains_key(&key) {
                let n=arrival(w,rng,NORTH)?;
                self.nodes.insert(key.clone(),(safe(&n)&&(!kill||cait(&n))).then(||Node {kill,arrival:n,first:Default::default(),failed:Default::default()}));
            }
            if let Some(node)=self.nodes.get_mut(&key).unwrap() {if let Some(p)=node_at_cost(node,cost)? {return Ok(Some(p));}}
        }
        Ok(None)
    }
}
fn node_at_cost(node:&mut Node,cost:u32)->Result<Option<Prediction>> {
    if node.failed.contains(&cost) {return Ok(None);}
    for first_cost in 0..=cost {for (n,d) in controls(first_cost) {
        let key=(n,d);
        if !node.first.contains_key(&key) {
            let prefix=if let Some(northern)=fight_defends(node.arrival.clone(),n,node.kill,d)? {
                let west=arrival(&northern.day.world_after,northern.day.after,WEST)?;
                (safe(&west)&&(node.kill||cait(&west))).then_some((northern,west))
            }else{None};
            node.first.insert(key,prefix);
        }
        let Some((_,west))=&node.first[&key] else {continue;};
        for (m,wd) in controls(cost-first_cost) {
            let Some(western)=fight_defends(west.clone(),m,!node.kill,wd)? else {continue;};
            let mut l=Loader::new(world::catalog(),western.departure().world_after.clone(),western.departure().after)?;
            let scenarios=l.state.active_scenarios.clone();l.visit_map(TOWN)?;
            ensure(l.state.active_scenarios==scenarios,"Conning arrival must not start a chapter")?;
            let town=checkpoint(&mut l,western.departure().after);l.visit_map(HARBOR)?;
            let harbor=checkpoint(&mut l,town.after);
            let (northern,_)=node.first.remove(&key).unwrap().unwrap();
            return Ok(Some(Prediction {northern,western,town,harbor,pomegranate:None,input_cost:cost,
                verification:"Source-derived coast search: safe first-round Defends and boost adjustments; no enemy action before the Soulstone or successful Escape. Both openings reject surprise. One Cultured Cait pack cleared. Assumes one encounter per coast; awaiting retail validation."}));
        }
    }}
    node.failed.insert(cost);Ok(None)
}

pub fn at_cost(w:&WorldState,rng:CrtRng,cost:u32)->Result<Option<Prediction>> {Cache::default().at_cost(w,rng,cost)}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary_defends_stop_at_enemy_and_replay_soulstone() {
        let i=input();let p=nearest(arrival(&i.world_after,i.after,NORTH).unwrap()).unwrap();
        let mut a=if p.northern.victory.is_some(){p.northern.arrival}else{p.western.arrival};
        let mut covered=0;
        for situation in ["Normal","Preemptive"] {for seed in 0..32 {
            a.battle_entry=CrtRng{state:seed,draws:500};a.encounter.situation=Some(situation.into());
            let m=menu(&a).unwrap();
            let expected=m.current_order.iter().take_while(|&&id|id<a.night.world_after.party.len()).count().saturating_sub(1);
            for d in 0..=3 {
                let f=fight_defends(a.clone(),0,true,d).unwrap();
                assert_eq!(f.is_some(),d<=expected,"seed={seed}, situation={situation}, d={d}, order={:?}",m.current_order);
                if let Some(f)=f {
                    assert_eq!(f.defends.len(),d);assert_eq!(f.menu.round(),1);
                    assert!(f.defends.iter().all(|r|r.actor.0<a.night.world_after.party.len()));
                    assert_eq!(f.victory.as_ref().unwrap().report.rng_before,f.menu.rng);
                    let replay=fight_defends(a.clone(),0,true,d).unwrap().unwrap();
                    assert_eq!(f.day.after,replay.day.after);covered+=usize::from(d>0);
                }
            }
        }}
        assert!(covered>0);
    }
    #[test]
    fn coast_search_can_select_odd_cost_defends() {
        let i=input();
        for seed in 0..1024 {
            let rng=CrtRng{state:seed,draws:i.after.draws};
            let mut cache=Cache::default();
            for cost in 0..=40 {
                if let Some(p)=cache.at_cost(&i.world_after,rng,cost).unwrap() {
                    if p.northern.defends.len()+p.western.defends.len()>0 {
                        assert_eq!(p.input_cost,control_cost(&p.northern)+control_cost(&p.western));
                        for f in [&p.northern,&p.western] {
                            let replay=fight_defends(f.arrival.clone(),f.inputs.extra_repeats,f.victory.is_some(),f.defends.len()).unwrap().unwrap();
                            assert_eq!(replay.departure().after,f.departure().after);
                        }
                        eprintln!("Defend witness seed={seed}, cost={cost}");return;
                    }
                    break;
                }
            }
        }
        panic!("Expected a selected Defend witness");
    }
    #[test]
    fn pomegranate_uses_carried_level_and_preserves_prior_fight() {
        for (level,chance) in [(1,8),(4,8),(5,15),(7,15),(8,55),(9,55),(10,65),(11,65),(12,80),(18,80),(19,100),(40,100)] {assert_eq!(pomegranate_chance(level),chance);}
        let i=input();let mut p=nearest(arrival(&i.world_after,i.after,NORTH).unwrap()).unwrap();
        for f in [&p.northern,&p.western] {if let Some(v)=&f.victory {
            for id in &f.day.world_after.party {
                assert_eq!(f.day.world_after.character_progression[id].experience-f.arrival.night.world_after.character_progression[id].experience,v.base_exp,"Cultured Cait receives 11 damage: no Overkill bonus on the coast pack");
            }
        }}
        let northern=serde_json::to_value(&p.northern).unwrap();
        let opening=serde_json::to_value(&p.western.arrival).unwrap();
        search_pomegranate(&mut p).unwrap();let tail=p.pomegranate.as_ref().unwrap();
        assert_eq!(serde_json::to_value(&p.northern).unwrap(),northern);
        assert_eq!(serde_json::to_value(&p.western.arrival).unwrap(),opening);
        assert!(tail.roll<=tail.chance);
        assert_eq!(tail.throne_level,p.western.departure().world_after.character_progression[&7].level);
        assert_eq!(tail.steal.after.draws-tail.steal.arrival.draws,3);
        assert_eq!(tail.steal.world_after.time,2);
        assert_eq!(tail.steal.world_after.map,HARBOR);
        let voyage=tail.voyage.as_ref().unwrap();
        assert_eq!(voyage.new_delsta.arrival,tail.steal.after);
        assert_eq!(voyage.anchorage.arrival,voyage.new_delsta.after);
        assert_eq!(voyage.canalbrine.arrival,voyage.anchorage.after);
        assert_eq!(voyage.anchorage.world_after.map,"Fld_Isd_1_3");
        assert_eq!(voyage.canalbrine.world_after.map,"Twn_Sea_1_1_A");
        assert_eq!(voyage.canalbrine.world_after.party,tail.steal.world_after.party);
        assert_eq!(serde_json::to_value(&voyage.canalbrine.world_after.character_progression).unwrap(),serde_json::to_value(&tail.steal.world_after.character_progression).unwrap());
        assert_eq!(voyage.canalbrine.world_after.time,2);
        assert_eq!(voyage.castti.world_after.party,voyage.canalbrine.world_after.party);
        assert!(voyage.castti.world_after.bench.contains(&3));
        assert_eq!(voyage.castti.world_after.character_progression[&3].level,1);
        assert!(voyage.castti.world_after.main_progress_points>voyage.canalbrine.world_after.main_progress_points);
        let inventor=tail.inventor.as_ref().unwrap();
        assert!(crate::inventor::safe(&inventor.approach));
        assert!(inventor.escape.as_ref().unwrap().success);
        for id in &inventor.scope.world_after.party {
            assert!(inventor.scope.world_after.character_progression[id].jobs.equipped_support.contains(crate::gravell_setup::STEP_AHEAD));
            assert_eq!(inventor.scope.world_after.character_progression[id].jobs.secondary_job,inventor.bomb.world_after.character_progression[id].jobs.secondary_job);
        }
        let harbor=&inventor.crackridge_return;
        assert_eq!(harbor.menu.current_order.len(),4);
        assert!(harbor.escape.as_ref().unwrap().success);
        assert!(matches!(harbor.encounter.situation.as_deref(),Some("Normal"|"Preemptive")));
        assert_eq!(harbor.arrival.arrival,inventor.clockbank_adjustment.as_ref().unwrap().departure.after);
        assert_eq!(harbor.field_return.world_after.map,"Fld_Wld_1_2");
        let south=harbor.southern.as_ref().unwrap();
        assert!(south.escape.as_ref().unwrap().success);
        assert!(matches!(south.encounter.situation.as_deref(),Some("Normal"|"Preemptive")));
        assert_eq!(south.arrival.arrival,harbor.field_return.after);
        assert_eq!(south.field_return.world_after.map,"Fld_Wld_2_2");
        let west=south.southern.as_ref().unwrap();let back=west.southern.as_ref().unwrap();
        for fight in [west.as_ref()] {
            assert!(fight.escape.as_ref().unwrap().success);
            assert!(matches!(fight.encounter.situation.as_deref(),Some("Normal"|"Preemptive")));
            assert_eq!(fight.field_return.world_after.map,"Fld_Wld_2_1");
            for id in &fight.arrival.world_after.party {assert_eq!(fight.field_return.world_after.character_progression[id].experience,fight.arrival.world_after.character_progression[id].experience);}
        }
        assert_eq!(back.approach_stages[0].1.world_after.map,"Twn_Wld_2_1_A");
        assert_eq!(back.approach_stages[0].1.arrival,west.field_return.after);
        assert_eq!(back.arrival.arrival,back.approach_stages[0].1.after);
        assert!(back.escape.as_ref().is_some_and(|e|e.success)||back.victory.as_ref().is_some_and(|v|v.base_exp>0));
        assert!(tail.wind_roll<=tail.wind_chance);
        let gravell=back.gravell.as_ref().unwrap();
        assert_eq!(gravell.license.world_after.job_licenses.get(&0),Some(&1));
        assert_eq!(gravell.license.after,back.field_return.after);
        assert_eq!(gravell.setup.after,gravell.license.after);
        assert_eq!(gravell.setup.world_after.character_progression[&7].jobs.secondary_job,Some(11));
        assert_eq!(gravell.setup.world_after.character_progression[&1].jobs.secondary_job,Some(0));
        for id in &gravell.setup.world_after.party {assert!(gravell.setup.world_after.character_progression[id].jobs.equipped_support.contains(crate::gravell_setup::STEP_AHEAD));}
        assert!(!gravell.setup.world_after.character_progression[&1].jobs.equipped_support.contains(crate::gravell_setup::MONEY_UP));
        let adjustment=inventor.clockbank_adjustment.as_ref().unwrap();
        println!("BG plan: {} Clockbank switches, {} input equivalents; Guards/BG inputs {:?}; last Western action {}",adjustment.switches,adjustment.input_cost,gravell.fight.actions.iter().filter(|a|matches!(a.action,ot2_battle_engine::gravell_routine::Action::Guards|ot2_battle_engine::gravell_routine::Action::BewilderingGrace)).map(|a|(a.action,a.inputs.len())).collect::<Vec<_>>(),if back.victory.is_some(){"Wind L"}else{"Escape"});
        let target=adjustment.target;
        let lottery=gravell.fight.trace.iter().find(|s|s.reason=="bewildering_grace_lottery"&&s.before==target).unwrap();
        assert_eq!(lottery.before,target);
        assert_eq!(gravell.fight.evaluation,ot2_battle_engine::gravell_routine::Evaluation::Accepted);
        assert!(gravell.fight.actions.iter().filter(|a|a.action==ot2_battle_engine::gravell_routine::Action::BewilderingGrace).all(|a|a.inputs.len()<=100));
        assert_eq!(gravell.night.world_after.time,4);
        assert_eq!(gravell.night.world_after.map,"Fld_Wld_3_2");
        assert_eq!(gravell.night.world_after.encounter_counter,gravell.arrival.world_after.encounter_counter);
        assert!(gravell.encounter.enemies.iter().any(|e|e.enemy.id=="ENE_NML_OTR_150"));
        assert!(matches!(gravell.encounter.situation.as_deref(),Some("Normal"|"Preemptive")));
        for id in &south.arrival.world_after.party {assert_eq!(south.field_return.world_after.character_progression[id].experience,south.arrival.world_after.character_progression[id].experience);}
        for id in &harbor.arrival.world_after.party {assert_eq!(harbor.field_return.world_after.character_progression[id].experience,harbor.arrival.world_after.character_progression[id].experience);}
        assert!(inventor.approach.agnea_story.world_after.active_scenarios.contains(&801));
        assert!(inventor.approach.agnea_story.world_after.flag(6550));
        assert!(inventor.approach.agnea_story.world_after.flag(34167));
        assert_eq!(inventor.approach.highroad.arrival,inventor.approach.agnea_story.after);
        assert_eq!(inventor.scope.world_after.party,voyage.castti.world_after.party);
        assert_eq!(inventor.scope.world_after.job_licenses.get(&11),Some(&1));
        assert!(inventor.scope.world_after.inventor_abilities.contains("ABI_SET_INV_030"));
        assert!(inventor.scope.world_after.inventor_abilities.contains("ABI_SET_INV_040"));
        assert!(inventor.scope.world_after.flag(2410)&&inventor.scope.world_after.flag(2411));
        assert_eq!(inventor.clockbank.as_ref().unwrap().arrival,inventor.southern_clockbank.as_ref().unwrap().after);
        assert_eq!(inventor.partitio_story.as_ref().unwrap().arrival,inventor.clockbank.as_ref().unwrap().after);
        assert_eq!(inventor.partitio_story.as_ref().unwrap().world_after.map,"Twn_Cty_2_1_A");
        assert!(inventor.partitio_story.as_ref().unwrap().world_after.active_scenarios.contains(&401));
        assert!(inventor.partitio_story.as_ref().unwrap().world_after.flag(6076));
        assert_eq!(inventor.partitio_story.as_ref().unwrap().world_after.main_progress_points,inventor.scope.world_after.main_progress_points);
        assert_eq!(inventor.western_clockbank.as_ref().unwrap().arrival,inventor.scope.after);
        assert_eq!(inventor.southern_clockbank.as_ref().unwrap().arrival,inventor.western_clockbank.as_ref().unwrap().after);
        assert_eq!(inventor.western_clockbank.as_ref().unwrap().world_after.map,"Fld_Cty_1_4");
        assert_eq!(inventor.southern_clockbank.as_ref().unwrap().world_after.map,"Fld_Cty_2_1");
        assert_eq!(serde_json::to_value(&inventor.southern_clockbank.as_ref().unwrap().world_after.character_progression).unwrap(),serde_json::to_value(&inventor.scope.world_after.character_progression).unwrap());
        let gondolas:Vec<_>=voyage.canalbrine.trace.iter().filter(|r|r.reason=="Gondola initial bobbing time").collect();
        assert_eq!(gondolas.len(),6);
        assert_eq!(gondolas.iter().filter(|r|r.subject.contains("NoPaddle")).count(),3);
        for row in gondolas {assert_eq!(row.after.draws-row.before.draws,1);}
        for stage in [&voyage.anchorage,&voyage.canalbrine] {
            assert!(stage.after.draws>stage.arrival.draws);
            assert!(serde_json::to_string(&stage.decisions).unwrap().contains("Fld_Ocn_1_1"));
        }
        assert_eq!(tail.input_cost,control_cost(&p.western)+7*(tail.town_switches.len().saturating_sub(2)+tail.harbor_switches.len().saturating_sub(2)) as u32);
        // Exhaust every strictly cheaper local combination independently.
        for n in 0..=tail.input_cost/2 {for t in 0..=tail.input_cost/7+4 {for h in 0..=tail.input_cost/7+4 {
            if (t+h)%2!=0||2*n+7*(t.saturating_sub(2)+h.saturating_sub(2))>=tail.input_cost {continue;}
            if let Some(f)=fight(p.western.arrival.clone(),n,p.western.victory.is_some()).unwrap() {
                if let Some((_,_,candidate))=pomegranate_tail(&f.day.world_after,f.day.after,t,h).unwrap() {
                    let voyage=canalbrine_voyage(&candidate.steal).unwrap();
                    assert!(!crate::inventor::safe(&crate::inventor::approach(&voyage.castti).unwrap()));
                }
            }
        }}}
    }
    #[derive(serde::Deserialize)] struct Input {after:CrtRng,world_after:WorldState}
    #[test]
    fn lv16_leaf_steal_is_searched_with_western_battle_boosts() {
        let input:Input=serde_json::from_str(include_str!("../data/source/coast-leaf-lv16-regression.json")).unwrap();
        let before=serde_json::to_value(&input.world_after).unwrap();
        assert_eq!(input.world_after.character_progression[&7].experience,1945);
        assert_eq!(steal_chance(16,4),100);assert_eq!(steal_chance(16,6),80);assert_eq!(steal_chance(17,6),100);
        let west=arrival(&input.world_after,input.after,WEST).unwrap();
        assert!(fight(west.clone(),1,false).unwrap().is_none());
        let mut no_leaves=west.clone();no_leaves.night.world_after.galdera_route=false;
        let unadjusted=fight(no_leaves,1,false).unwrap().expect("One-cycle Escape itself succeeds; first steal rolls 97");
        assert!(steal_leaves(&unadjusted.day).unwrap().is_none());
        let selected=(0..32).find_map(|n|fight(west.clone(),n,false).unwrap()).expect("Boost search secures both steals");
        eprintln!("Pom-first selected boost cycles: {}",selected.inputs.extra_repeats);
        let steal=selected.leaf_steal.unwrap();
        assert_eq!(steal.world_after.character_progression[&7].level,16);
        let checks:Vec<_>=steal.trace.iter().filter(|t|t.kind==world::DrawKind::StealCheck).collect();
        assert_eq!(checks.len(),2);assert_eq!(checks[0].subject,"ITM_MRL_REV_0030");assert_eq!(checks[1].subject,"ITM_MRL_REV_0080");
        assert!(checks[0].value.unwrap()<=80);assert!(checks[1].value.unwrap()<=100);
        assert_eq!(steal.after.draws-steal.arrival.draws,3);
        for id in ["ITM_MRL_REV_0080","ITM_MRL_REV_0030"] {assert_eq!(steal.world_after.route_items.get(id).copied().unwrap_or(0),input.world_after.route_items.get(id).copied().unwrap_or(0)+1);}
        assert_eq!(serde_json::to_value(&input.world_after).unwrap(),before);
    }
    fn input()->Input {serde_json::from_str(include_str!("../data/source/conning-test-entry.json")).unwrap()}
    #[test]
    fn nearest_controls_preserve_required_outcomes_and_replay() {
        let input=input();
        let north=arrival(&input.world_after,input.after,NORTH).unwrap();
        let p=nearest(north.clone()).unwrap();
        assert!(safe(&p.northern.arrival)&&safe(&p.western.arrival));
        assert_ne!(p.northern.victory.is_some(),p.western.victory.is_some());
        let killed=if p.northern.victory.is_some(){&p.northern}else{&p.western};
        let escaped=if p.northern.escape.is_some(){&p.northern}else{&p.western};
        assert!(cait(&killed.arrival));assert!(escaped.escape.as_ref().unwrap().success);
        let n=p.northern.inputs.extra_repeats;
        for earlier in 0..n {
            if let Some(f)=fight(north.clone(),earlier,cait(&north)).unwrap() {
                let west=arrival(&f.day.world_after,f.day.after,WEST).unwrap();
                assert!(!safe(&west)||(!cait(&north)&&!cait(&west)));
            }
        }
        let replay=nearest(north).unwrap();
        assert_eq!(serde_json::to_value(p).unwrap(),serde_json::to_value(replay).unwrap());
    }
    #[test]
    fn coast_route_preserves_bench_and_awards_actual_party_before_next_encounter() {
        let input=input();let w=&input.world_after;
        let immutable=crate::state_key::WorldKey::new(w);
        let mut search=Cache::default();
        let result=(0..=6).step_by(2).find_map(|cost|search.at_cost(w,input.after,cost).unwrap()).unwrap();
        assert!(result.northern.victory.is_some()&&result.western.escape.as_ref().unwrap().success);
        assert!(safe(&result.northern.arrival)&&safe(&result.western.arrival));
        assert!(cait(&result.northern.arrival));
        assert_eq!(result.town.world_after.party,[7,1,8,4]);
        assert_eq!(result.town.world_after.bench,[6]);
        assert_eq!(result.town.world_after.active_scenarios,w.active_scenarios);
        assert_eq!(serde_json::to_value(&result.town.world_after.character_progression[&6]).unwrap(),serde_json::to_value(&w.character_progression[&6]).unwrap());
        for &id in &w.party {assert!(result.town.world_after.character_progression[&id].experience>w.character_progression[&id].experience);}
        assert_eq!(result.western.menu.party_levels,result.northern.day.world_after.party.iter().map(|id|result.northern.day.world_after.character_progression[id].level).collect::<Vec<_>>());
        assert_eq!(result.town.trace.iter().filter(|t|t.kind==world::DrawKind::EnvironmentInitialization).count(),33);
        assert_eq!(result.harbor.arrival,result.town.after);
        assert_eq!(result.harbor.world_after.map,HARBOR);
        assert_eq!(world::catalog().maps[HARBOR].id,125);
        assert_eq!(result.harbor.world_after.active_scenarios,result.town.world_after.active_scenarios);
        assert_eq!(serde_json::to_value(&result.harbor.world_after.character_progression).unwrap(),serde_json::to_value(&result.town.world_after.character_progression).unwrap());
        assert!(!result.harbor.trace.iter().any(|t|matches!(t.kind,world::DrawKind::EnvironmentInitialization|world::DrawKind::TavernLottery)));
        let tavern:Vec<_>=result.town.trace.iter().filter(|t|t.kind==world::DrawKind::TavernLottery).collect();
        assert!(!tavern.is_empty());
        assert!(tavern[0].value.unwrap()<100); // appearance lottery is index(100); later draws select members
        for f in [&result.northern,&result.western] {
            assert_eq!(f.arrival.night.world_after.time,4);assert_eq!(f.day.world_after.time,2);
            assert_eq!(f.arrival.entry.world_after.encounter_counter,f.arrival.night.world_after.encounter_counter,"Manual switch must not invent a counter reset");
            assert_eq!(f.field_return.after.draws,f.field_return.arrival.draws+2);
        }
        assert_eq!(crate::state_key::WorldKey::new(w),immutable);
        let fresh=at_cost(w,input.after,result.input_cost).unwrap().unwrap();
        assert_eq!(serde_json::to_value(&fresh).unwrap(),serde_json::to_value(&result).unwrap(),"Cached and independent complete replay must match");
    }
    #[test]
    fn west_can_supply_cait_after_north_escape_and_surprise_is_rejected() {
        let input=input();let mut rng=input.after;let mut covered=false;
        for _ in 0..512 {
            let n=arrival(&input.world_after,rng,NORTH).unwrap();
            if safe(&n)&&!cait(&n) {
                if let Some(p)=at_cost(&input.world_after,rng,8).unwrap() {
                    assert!(p.northern.escape.as_ref().unwrap().success);
                    assert!(p.western.victory.is_some());assert!(cait(&p.western.arrival));
                    covered=true;break;
                }
            } else if !safe(&n) {assert!(at_cost(&input.world_after,rng,8).unwrap().is_none());}
            rng.advance(1);
        }
        assert!(covered);
    }
}
