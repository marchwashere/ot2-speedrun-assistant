//! Cost-ordered field Escape candidates. Battle/RNG replay lives in the core.
use crate::{
    CrtRng, Result,
    highroad::{Escape, EscapeMethod},
    manipulation::{describe_inputs, optimal_inputs},
};
use ot2_battle_engine::{
    combat::Weapon,
    escape::Situation,
    field_escape::{FieldBreakMenu, FieldBreakBoostCursor, FieldEscapeMenu, FieldEscapeAttempt},
};
use std::collections::BTreeMap;

pub const BREAK_INPUT_COST: u32 = 11;
pub const FIREBALL_BREAK_INPUT_COST: u32 = 26;
fn break_action_cost(weapon_index:usize)->u32 {
    match weapon_index {0..=2=>BREAK_INPUT_COST,4=>FIREBALL_BREAK_INPUT_COST,_=>6}
}
/// User-specified Defend cost in boost-input equivalents.
pub const DEFEND_INPUT_COST: u32 = 4;
pub const ROUND_INPUT_COST:u32=6;
pub fn elapsed_round_cost(before:u32,after:u32)->u32 {after.saturating_sub(before)*ROUND_INPUT_COST}
pub struct FieldSearch {
    pub menu: FieldEscapeMenu,
    require_break: bool,
    spell_breaks: bool,
    carried_players: Option<Vec<ot2_battle_engine::PlayerCharacter>>,
    baseline: FieldEscapeAttempt,
    direct_prefix: FieldEscapeMenu,
    direct_repeats: u32,
    entry: CrtRng,
    situation: Situation,
    level: u32,
    enemies: Vec<String>,
    danger: u32,
    party_levels: Vec<u32>,
    night: bool,
    history: ot2_battle_engine::knowledge::EnemyHistory,
    breaks: BTreeMap<(u32, u8, u8), Option<FieldBreakMenu>>,
    break_prefixes: BTreeMap<(u32, u8, u8), FieldBreakBoostCursor>,
}
impl FieldSearch {
    pub fn new(
        entry: CrtRng,
        situation: Situation,
        level: u32,
        enemies: &[&str],
        danger: u32,
    ) -> Result<Self> {
        Self::with_party(entry,situation,&[level],enemies,danger,false)
    }
    pub fn with_party(entry:CrtRng,situation:Situation,levels:&[u32],enemies:&[&str],danger:u32,night:bool) -> Result<Self> {
        Self::with_history(entry,situation,levels,enemies,danger,night,&Default::default())
    }
    pub fn with_history(entry:CrtRng,situation:Situation,levels:&[u32],enemies:&[&str],danger:u32,night:bool,history:&ot2_battle_engine::knowledge::EnemyHistory)->Result<Self> {
        let menu=FieldEscapeMenu::with_history(entry,situation,levels,enemies,history)?;
        Self::from_menu(menu,entry,situation,levels,enemies,danger,night,history)
    }
    /// Exact carried roster; weapon breaks support original-gear Throné/Osvald.
    pub fn with_players(entry:CrtRng,situation:Situation,players:&[ot2_battle_engine::PlayerCharacter],levels:&[u32],enemies:&[&str],danger:u32,history:&ot2_battle_engine::knowledge::EnemyHistory)->Result<Self> {
        let menu=FieldEscapeMenu::with_players(entry,situation,players,levels,enemies,history)?;
        let mut search=Self::from_menu(menu,entry,situation,levels,enemies,danger,false,history)?;
        search.carried_players=Some(players.to_vec());Ok(search)
    }
    fn from_menu(menu:FieldEscapeMenu,entry:CrtRng,situation:Situation,levels:&[u32],enemies:&[&str],danger:u32,night:bool,history:&ot2_battle_engine::knowledge::EnemyHistory)->Result<Self> {
        let baseline=menu.clone().attempt(&[],danger)?;
        Ok(Self {
            require_break:false,spell_breaks:false,carried_players:None,
            direct_prefix:menu.clone(), direct_repeats:0, baseline,
            menu,
            entry,
            situation,
            level:levels[0],
            enemies:enemies.iter().map(|id|(*id).to_owned()).collect(),
            danger,
            party_levels:levels.to_vec(),night,history:history.clone(),
            breaks: BTreeMap::new(),
            break_prefixes: BTreeMap::new(),
        })
    }
    pub fn enable_spell_breaks(&mut self,players:Vec<ot2_battle_engine::PlayerCharacter>) {self.spell_breaks=true;self.carried_players=Some(players);}
    pub fn require_break(&mut self) { self.require_break=true; }
    fn base(&self) -> Result<Escape> {
        Ok(Escape {
            menu: self.menu.clone(),
            baseline: self.baseline.clone(),
            inputs: None,
            escape_boost_inputs: None,
            weighted_input_cost: 0,
            boost_repeat_count: 0,
            attempt: None,
            method: EscapeMethod::Unavailable,
            break_action: None,
            status: "No Escape solution",
            flats_search: None,
            flats_approach_search: None,
            flats_break_search: None,
            tested_candidates: 0,
        })
    }
    /// All non-redundant RNG allocations with exactly this estimated time cost.
    pub fn candidates(
        &mut self,
        cost: u32,
        allow_break: bool,
        repeat_limit: Option<u32>,
    ) -> Result<Vec<Escape>> {
        self.candidates_with_policy(cost,allow_break,repeat_limit,false)
    }
    /// Physical breaks spend no BP; enabled spells may finish at +1 BP.
    pub fn unboosted_break_candidates(&mut self,cost:u32) -> Result<Vec<Escape>> {
        self.candidates_with_policy(cost,true,None,true)
    }
    /// Visit in exactly the same order as `unboosted_break_candidates`, stopping
    /// before constructing subsequent candidates once the visitor accepts one.
    pub fn visit_unboosted_break_candidates<T>(&mut self,cost:u32,visit:impl FnMut(Escape)->Result<Option<T>>) -> Result<Option<T>> {
        self.visit_candidates_with_policy(cost,true,None,true,visit)
    }
    fn candidates_with_policy(&mut self,cost:u32,allow_break:bool,repeat_limit:Option<u32>,zero_bp_only:bool) -> Result<Vec<Escape>> {
        let mut out=vec![];
        self.visit_candidates_with_policy(cost,allow_break,repeat_limit,zero_bp_only,|p| {out.push(p);Ok(None::<()>)})?;
        Ok(out)
    }
    fn visit_candidates_with_policy<T>(&mut self,cost:u32,allow_break:bool,repeat_limit:Option<u32>,zero_bp_only:bool,mut visit:impl FnMut(Escape)->Result<Option<T>>) -> Result<Option<T>> {
        let _profile=crate::profile::span("Escape candidates");
        // Break actions have their own costs/parity; only direct Escape requires
        // an even number of boost inputs.
        if !self.require_break && cost % 2 == 0 && repeat_limit.is_none_or(|n| cost / 2 <= n) {
            if self.direct_repeats>cost/2 {
                self.direct_prefix=self.menu.clone();self.direct_repeats=0;
            }
            while self.direct_repeats<cost/2 {
                self.direct_prefix.boost_cancel_cycle();self.direct_repeats+=1;
            }
            let attempt = self.direct_prefix.clone().attempt(&[], self.danger)?;
            if attempt.success {
                let mut p = self.base()?;
                let path: Vec<_> = (0..cost / 2).flat_map(|_| [1, 0]).collect();
                p.inputs = Some(describe_inputs(cost / 2, 0, path)?);
                p.attempt = Some(attempt);
                p.method = EscapeMethod::WithinLimit;
                p.status = "First Escape succeeds";
                p.weighted_input_cost = cost;
                p.boost_repeat_count = cost / 2;
                if let Some(result)=visit(p)? {return Ok(Some(result));}
            }
        }
        if !allow_break || (self.enemies.len() != 1 && !self.spell_breaks) || (self.level != 5 && self.carried_players.is_none()) {
            return Ok(None);
        }
        for weapon_index in 0..if self.spell_breaks {6}else{3} {
        let Some(available) = cost.checked_sub(break_action_cost(weapon_index)) else {continue;};
        for attack_count in 0..=available {
            let boost = (attack_count % 2) as u8;
            let repeats = attack_count / 2;
            if repeat_limit.is_some_and(|n| repeats > n) {
                continue;
            }
            let mut attack: Vec<_> = (0..repeats).flat_map(|_| [1, 0]).collect();
            if boost == 1 {
                attack.push(1);
            }
                let weapon=[Weapon::Dagger,Weapon::Sword,Weapon::Staff][weapon_index.min(2)];
                if zero_bp_only && weapon_index<3 && boost!=0 {continue;}
                let key = (repeats, boost, weapon_index as u8);
                if !self.breaks.contains_key(&key) {
                    let ids=self.enemies.iter().map(String::as_str).collect::<Vec<_>>();
                    let menu=if weapon_index<3 {
                        ot2_battle_engine::field_escape::opening_break_menu_with_players(self.entry,self.situation,&self.party_levels,&ids,weapon,&attack,self.night,&self.history,self.carried_players.as_deref())?
                    } else {
                        let skill=["ABI_SET_THI_020","ABI_SET_SCH_010","ABI_SET_SCH_020"][weapon_index-3];
                        ot2_battle_engine::field_escape::opening_spell_break_menu(self.entry,self.situation,&self.party_levels,&ids,skill,&attack,self.night,&self.history,self.carried_players.as_ref().unwrap())?
                    };
                    self.breaks.insert(key,menu);
                }
                let Some(menu) = self.breaks[&key].as_ref() else {
                    continue;
                };
                let mut defended=Some(menu.clone());
                for defend_count in 0..=if self.spell_breaks {4}else{0} {
                if defend_count>0 {defended=defended.take().unwrap().safe_defend()?;}
                let Some(menu)=defended.as_ref() else {break};
                let defend_cost=defend_count*DEFEND_INPUT_COST+elapsed_round_cost(1,menu.round());
                if available-attack_count<defend_cost {break;}
                let key=(key.0,key.1+(defend_count as u8)*2,key.2);
                let count = available - attack_count - defend_cost;
                if count % 2 != 0 || (menu.available_bp() == 0 && count != 0) {
                    continue;
                }
                for draws in 0..=u32::from(menu.available_bp().min(3)) * count / 2 {
                    let inputs = optimal_inputs(draws, menu.available_bp(), 0)?;
                    if inputs.input_count as u32 != count
                        || repeat_limit.is_some_and(|n| repeats + inputs.extra_repeats > n)
                    {
                        continue;
                    }
                    // Keep one growing prefix per opening break, before the
                    // final BP reduction. Paths changing earlier inputs reset
                    // to the immutable second menu; rejected Escape checks
                    // always consume a clone, never this reusable prefix.
                    let prefix_len=inputs.levels.len().saturating_sub(1);
                    let prefix=self.break_prefixes.entry(key).or_insert_with(||menu.clone().boost_cursor());
                    if !inputs.levels[..prefix_len].starts_with(prefix.inputs()) {
                        *prefix=menu.clone().boost_cursor();
                    }
                    prefix.extend(&inputs.levels[prefix.inputs().len()..prefix_len])?;
                    let mut trial=prefix.clone();trial.extend(&inputs.levels[prefix_len..])?;
                    let b = trial.attempt()?;
                    if self.require_break && b.attack_actor!=0 {continue;}
                    let mut p = self.base()?;
                    p.inputs = Some(describe_inputs(repeats, boost, attack.clone())?);
                    p.boost_repeat_count = repeats + inputs.extra_repeats;
                    p.escape_boost_inputs = Some(inputs);
                    p.attempt = Some(b.attempt.clone());
                    p.break_action = Some(b);
                    p.method = EscapeMethod::BreakThenEscape;
                    p.status = "Break, adjust at next menu, then guaranteed Escape";
                    p.weighted_input_cost = cost;
                    if let Some(result)=visit(p)? {return Ok(Some(result));}
                }
                }
            }
        }
        Ok(None)
    }
    pub fn unavailable(&self) -> Result<Escape> {
        self.base()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ot2_battle_engine::field_escape::{opening_break_menu,opening_break_menu_with_history};
    #[test]
    fn spell_search_includes_boosted_cast_at_odd_cost() {
        let f:serde_json::Value=serde_json::from_str(include_str!("../../OT2BattleEngineRust/data/source/live-snows-fireball-20261007.json")).unwrap();
        let players=serde_json::from_value(f["players"].clone()).unwrap();
        let history=serde_json::from_value(f["history"].clone()).unwrap();
        let entry=serde_json::from_value(f["entry"].clone()).unwrap();
        let mut search=FieldSearch::with_history(entry,Situation::Normal,&[5,1],&["ENE_NML_FLD_SNW_010","ENE_NML_FLD_SNW_010"],5,false,&history).unwrap();
        search.enable_spell_breaks(players);
        let candidates=search.unboosted_break_candidates(FIREBALL_BREAK_INPUT_COST+1).unwrap();
        let p=candidates.iter().find(|p|p.break_action.as_ref().is_some_and(|b|b.skill.as_deref()==Some("ABI_SET_SCH_010") && b.boost==1)).expect("Missing cheaper boosted Fireball");
        assert_eq!(p.weighted_input_cost,FIREBALL_BREAK_INPUT_COST+1);
        assert_eq!(p.break_action.as_ref().unwrap().attack_inputs,vec![1]);
        assert!(p.attempt.as_ref().unwrap().success);
    }
    #[test]
    fn spell_search_includes_safe_defend_at_its_action_cost() {
        let players=vec![ot2_battle_engine::field_escape::route_player(7,63).unwrap(),ot2_battle_engine::field_escape::route_player(6,0).unwrap()];
        for seed in 0..64 {
            let mut search=FieldSearch::with_party(CrtRng::new(seed),Situation::Normal,&[5,1],&["ENE_NML_FLD_SNW_010","ENE_NML_FLD_SNW_010"],5,false).unwrap();
            search.enable_spell_breaks(players.clone());
            for p in search.unboosted_break_candidates(FIREBALL_BREAK_INPUT_COST+DEFEND_INPUT_COST+ROUND_INPUT_COST).unwrap() {
                if let Some(b)=p.break_action.as_ref().filter(|b|b.defends.len()==1) {
                    assert_eq!(b.skill.as_deref(),Some("ABI_SET_SCH_010"));
                    assert_eq!(p.weighted_input_cost,FIREBALL_BREAK_INPUT_COST+DEFEND_INPUT_COST+ROUND_INPUT_COST);
                    assert_eq!(b.escape_round,2);
                    assert_eq!(p.boost_repeat_count,0);
                    assert_eq!(p.attempt.unwrap().chance_percent,100);
                    return;
                }
            }
        }
        panic!("Missing safe Defend candidate");
    }
    #[test]
    fn incremental_candidates_match_independent_fresh_attempts_across_cost_revisits() {
        let enemies=["ENE_NML_FLD_SNW_010"];
        let mut direct=0;
        let mut broken=0;
        for levels in [&[5][..],&[5,1][..]] {
            for seed in [0,123,512] {
                let entry=CrtRng {state:seed,draws:6100};
                let mut search=FieldSearch::with_party(entry,Situation::Normal,levels,&enemies,5,false).unwrap();
                for cost in (0..=24).chain([4,18,0,24,2]) {
                    let candidates=if levels.len()==1 {search.candidates(cost,true,None)} else {search.unboosted_break_candidates(cost)}.unwrap();
                    for p in candidates {
                        let before=search.menu.clone().attempt(&[],5).unwrap();
                        assert_eq!(serde_json::to_value(&p.baseline).unwrap(),serde_json::to_value(before).unwrap());
                        let inputs=&p.inputs.as_ref().unwrap().levels;
                        if let Some(actual)=p.break_action {
                            broken+=1;
                            let menu=opening_break_menu_with_history(entry,Situation::Normal,levels,&enemies,actual.weapon,inputs,false,&Default::default()).unwrap().unwrap();
                            let expected=menu.attempt(&p.escape_boost_inputs.as_ref().unwrap().levels).unwrap();
                            assert_eq!(serde_json::to_value(actual).unwrap(),serde_json::to_value(expected).unwrap());
                        } else {
                            direct+=1;
                            let expected=search.menu.clone().attempt(inputs,5).unwrap();
                            assert_eq!(serde_json::to_value(p.attempt.unwrap()).unwrap(),serde_json::to_value(expected).unwrap());
                        }
                    }
                }
            }
        }
        assert!(direct>0 && broken>0);
    }

    #[test]
    fn streaming_candidates_preserve_order_and_stop_after_acceptance() {
        let enemies=["ENE_NML_FLD_CTY_010"];
        for cost in [10,12,16,20] {
            let make=||FieldSearch::new(CrtRng::new(123),Situation::Normal,5,&enemies,5).unwrap();
            let expected=make().unboosted_break_candidates(cost).unwrap();
            for accept_at in 0..expected.len() {
                let mut search=make();
                let mut visited=vec![];
                let chosen=search.visit_unboosted_break_candidates(cost,|p| {
                    let value=serde_json::to_value(&p).unwrap();visited.push(value);
                    Ok((visited.len()==accept_at+1).then_some(p))
                }).unwrap().unwrap();
                assert_eq!(serde_json::to_value(chosen).unwrap(),serde_json::to_value(&expected[accept_at]).unwrap());
                assert_eq!(visited,expected[..=accept_at].iter().map(|p|serde_json::to_value(p).unwrap()).collect::<Vec<_>>());
            }
        }
    }
    #[test]
    fn round_two_boosting_beats_equivalent_round_one_draws_and_replays_snapshot() {
        let entry = CrtRng {
            state: 123,
            draws: 6000,
        };
        let enemies = ["ENE_NML_FLD_CTY_010"];
        let source = opening_break_menu(entry, Situation::Normal, 5, &enemies, Weapon::Dagger, &[])
            .unwrap()
            .unwrap();
        let target = source
            .clone()
            .attempt(&[1, 2, 1, 2, 1, 0])
            .unwrap()
            .attempt
            .after;
        let p = crate::highroad::escape_plan_with_limit(
            entry,
            Situation::Normal,
            5,
            &enemies,
            5,
            Some(4),
            None,
            |a| Ok(a.after == target),
        )
        .unwrap();
        assert_eq!(p.weighted_input_cost, BREAK_INPUT_COST+ROUND_INPUT_COST+6);
        let b = p.break_action.unwrap();
        assert_eq!(b.boost, 0);
        assert!(!b.escape_inputs.is_empty());
        assert_eq!(b.available_escape_bp, 2);
        assert_eq!(b.attempt.after, target);
        assert_eq!(p.boost_repeat_count, 3);
        assert_eq!(
            source.attempt(&[]).unwrap().at_escape_menu,
            b.at_escape_menu
        );
        let direct = crate::highroad::escape_plan_with_limit(
            entry,
            Situation::Normal,
            5,
            &enemies,
            5,
            None,
            None,
            |a| Ok(a.after.draws >= target.draws),
        )
        .unwrap();
        assert!(direct.weighted_input_cost > p.weighted_input_cost);
    }
}

#[cfg(test)]
mod action_cost_tests {
 #[test]
 fn defend_charges_round_transition_only_when_it_happens() {
  use super::*;
  assert_eq!(DEFEND_INPUT_COST+elapsed_round_cost(1,1),4);
  assert_eq!(DEFEND_INPUT_COST+elapsed_round_cost(1,2),10);
  assert_eq!(BREAK_INPUT_COST+elapsed_round_cost(1,1),11);
  assert_eq!(BREAK_INPUT_COST+elapsed_round_cost(1,2),17);
 }
}
