//! Target-first scheduling estimates. Only full route replay may accept a plan.
use crate::{CrtRng,Result};
pub type Curve=[u64;8];
#[derive(Debug,Clone)]
pub struct Candidate {pub target:CrtRng,pub departure:usize,pub counts:Vec<u8>,pub score:u64}
pub fn targets(mut rng:CrtRng,draws:u64,ability:&str)->Result<Vec<CrtRng>> {
 let mut out=vec![];
 for _ in 0..draws {let at=rng; rng.advance(1);let mut probe=at;
  if ot2_battle_engine::bewildering_grace::Dance::new(3)?.select(&mut probe)?.unwrap().ability==ability {out.push(at);}
 }Ok(out)
}
pub fn defend_cost(n:u8)->u32 {u32::from(n)*crate::escape_search::DEFEND_INPUT_COST+if n>=4{crate::escape_search::ROUND_INPUT_COST}else{0}}
/// Every allocation is ranked against this target; there is no maximum-Defend default.
/// Departure tuples contain absolute predicted RNG draws and paid switch cost.
pub fn rank(targets:&[CrtRng],departures:&[(u64,u32)],natural:u64,curves:&[Curve],limit:u32,keep:usize)->Vec<Candidate> {
 let mut profiles=vec![(vec![],0u64,0u32)];
 for curve in curves {let mut next=vec![];for (p,draws,cost) in profiles {for n in 0..8 {
  if curve[n]==u64::MAX {continue;}let mut p=p.clone();p.push(n as u8);
  next.push((p,draws+curve[n],cost+defend_cost(n as u8)));
 }}profiles=next;}
 let mut out=vec![];
 for &target in targets {
  let mut rows=vec![];
  for (counts,draws,cost) in &profiles {
   let offset=natural+draws;
   let desired=target.draws.saturating_sub(offset);
   let center=departures.partition_point(|&(d,_)|d<=desired).saturating_sub(1);
   // Nearby departures cover formation/voice variation; overshoots are estimates only.
   for i in center.saturating_sub(2)..=(center+2).min(departures.len().saturating_sub(1)) {
    let Some(&(d,switch_cost))=departures.get(i)else{continue;};
    let paid=switch_cost+cost;if paid>=limit {continue;}
    let predicted=d+offset;let delta=target.draws as i128-predicted as i128;
    if !(-96..=512).contains(&delta){continue;}
    let fine=if delta>=0 {(delta as u64).div_ceil(2)}else{(-delta)as u64*2};
    rows.push(Candidate{target,departure:i,counts:counts.clone(),score:u64::from(paid)+fine});
   }
  }
  rows.sort_by_key(|c|(c.score,c.departure,c.counts.clone()));rows.truncate(keep);out.extend(rows);
 }
 out.sort_by_key(|c|(c.score,c.target.draws,c.departure));out
}
pub fn curve(entry:CrtRng,situation:ot2_battle_engine::escape::Situation,world:&crate::world::WorldState,ids:&[&str])->Result<Curve> {
 let players:Vec<_>=world.party.iter().map(|id|world.character_progression[id].character.clone()).collect();
 let levels:Vec<_>=world.party.iter().map(|id|world.character_progression[id].level).collect();
 let mut menu=ot2_battle_engine::field_escape::FieldEscapeMenu::with_step_ahead(entry,situation,&players,&levels,ids,&world.enemy_knowledge)?;
 let start=menu.rng.draws;let mut out=[u64::MAX;8];out[0]=0;
 for slot in out.iter_mut().skip(1) {let Some((next,_))=menu.safe_step_ahead_defend_through_next_round()?else{break};*slot=next.rng.draws-start;menu=next;}Ok(out)
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn target_scan_matches_source_lottery_positions(){
 assert!(targets(CrtRng::new(0),400,"ABI_DAN_MAKA_001").unwrap().iter().any(|t|t.draws==305));
 assert!(targets(CrtRng::new(0),3400,"ABI_DAN_MAKA_002").unwrap().iter().any(|t|t.draws==3285));
 }
 #[test]fn nearby_target_does_not_force_defends_or_switches(){
 let curves=[[0,20,40,60,100,120,140,160]];
 let r=rank(&[CrtRng{state:0,draws:110}],&[(100,0),(200,12)],10,&curves,100,8);
 assert_eq!(r[0].counts,[0]);assert_eq!(r[0].departure,0);
 }
 #[test]fn distant_target_prefers_useful_defends_and_leaves_the_rest_unused(){
 let curves=[[0,40,80,120,180,220,260,300];2];
 let r=rank(&[CrtRng{state:0,draws:190}],&[(100,0),(200,12)],10,&curves,200,32);
 assert_eq!(r[0].departure,0);
 assert!(r[0].counts.iter().map(|&n|defend_cost(n)).sum::<u32>()<12);
 assert!(r[0].counts.iter().all(|&n|n<7));
 }
}
