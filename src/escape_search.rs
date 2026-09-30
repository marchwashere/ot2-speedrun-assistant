//! Cost-ordered field Escape candidates. Battle/RNG replay lives in the core.
use crate::{
    CrtRng, Result,
    highroad::{Escape, EscapeMethod},
    manipulation::{describe_inputs, optimal_inputs},
};
use ot2_battle_engine::{
    combat::Weapon,
    escape::Situation,
    field_escape::{FieldBreakMenu, FieldEscapeMenu, opening_break_menu},
};
use std::collections::BTreeMap;

pub const BREAK_INPUT_COST: u32 = 10;
pub struct FieldSearch<'a> {
    pub menu: FieldEscapeMenu,
    entry: CrtRng,
    situation: Situation,
    level: u32,
    enemies: &'a [&'a str],
    danger: u32,
    breaks: BTreeMap<(u32, u8, u8), Option<FieldBreakMenu>>,
}
impl<'a> FieldSearch<'a> {
    pub fn new(
        entry: CrtRng,
        situation: Situation,
        level: u32,
        enemies: &'a [&'a str],
        danger: u32,
    ) -> Result<Self> {
        Ok(Self {
            menu: FieldEscapeMenu::new(entry, situation, level, enemies)?,
            entry,
            situation,
            level,
            enemies,
            danger,
            breaks: BTreeMap::new(),
        })
    }
    fn base(&self) -> Result<Escape> {
        Ok(Escape {
            menu: self.menu.clone(),
            baseline: self.menu.clone().attempt(&[], self.danger)?,
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
        let mut out = vec![];
        if cost % 2 == 0 && repeat_limit.is_none_or(|n| cost / 2 <= n) {
            let path: Vec<_> = (0..cost / 2).flat_map(|_| [1, 0]).collect();
            let attempt = self.menu.clone().attempt(&path, self.danger)?;
            if attempt.success {
                let mut p = self.base()?;
                p.inputs = Some(describe_inputs(cost / 2, 0, path)?);
                p.attempt = Some(attempt);
                p.method = EscapeMethod::WithinLimit;
                p.status = "First Escape succeeds";
                p.weighted_input_cost = cost;
                p.boost_repeat_count = cost / 2;
                out.push(p);
            }
        }
        if !allow_break || cost < BREAK_INPUT_COST || self.enemies.len() != 1 || self.level != 5 {
            return Ok(out);
        }
        let available = cost - BREAK_INPUT_COST;
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
            for (weapon_index, weapon) in [Weapon::Dagger, Weapon::Sword].into_iter().enumerate() {
                let key = (repeats, boost, weapon_index as u8);
                if !self.breaks.contains_key(&key) {
                    self.breaks.insert(
                        key,
                        opening_break_menu(
                            self.entry,
                            self.situation,
                            self.level,
                            self.enemies,
                            weapon,
                            &attack,
                        )?,
                    );
                }
                let Some(menu) = self.breaks[&key].as_ref() else {
                    continue;
                };
                let count = available - attack_count;
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
                    let b = menu.clone().attempt(&inputs.levels)?;
                    let mut p = self.base()?;
                    p.inputs = Some(describe_inputs(repeats, boost, attack.clone())?);
                    p.boost_repeat_count = repeats + inputs.extra_repeats;
                    p.escape_boost_inputs = Some(inputs);
                    p.attempt = Some(b.attempt.clone());
                    p.break_action = Some(b);
                    p.method = EscapeMethod::BreakThenEscape;
                    p.status = "Break, adjust at next menu, then guaranteed Escape";
                    p.weighted_input_cost = cost;
                    out.push(p);
                }
            }
        }
        Ok(out)
    }
    pub fn unavailable(&self) -> Result<Escape> {
        self.base()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(p.weighted_input_cost, 16);
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
