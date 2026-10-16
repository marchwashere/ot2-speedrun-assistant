//! Replacement flags are sampled on trigger activation, not on every battle.
use crate::{ensure, Result, SimError};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::OnceLock};
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ActiveEncounter {
    pub map: String,
    pub base: String,
    pub zone: String,
}
#[derive(Debug, Deserialize)]
struct Replacement {
    flag: u32,
    zone: String,
}
#[derive(Debug, Deserialize)]
struct Binding {
    key: String,
    map: String,
    base: String,
    start_flags: Vec<u32>,
    end_flags: Vec<u32>,
    replacements: Vec<Replacement>,
}
fn bindings() -> &'static Vec<Binding> {
    static DATA: OnceLock<Vec<Binding>> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../data/encounter-activation.json"))
            .expect("source trigger bindings")
    })
}
pub fn activate(
    map: &str,
    base: &str,
    flags: &BTreeSet<u32>,
    binding: Option<&str>,
) -> Result<ActiveEncounter> {
    let mut selected = None;
    let mut known = false;
    for b in bindings()
        .iter()
        .filter(|b| b.map == map && b.base == base && binding.is_none_or(|key| key == b.key))
    {
        known = true;
        if !b.start_flags.iter().all(|f| flags.contains(f))
            || b.end_flags.iter().any(|f| flags.contains(f))
        {
            continue;
        }
        let zone = b
            .replacements
            .iter()
            .find(|r| flags.contains(&r.flag))
            .map_or(base, |r| r.zone.as_str());
        ensure(
            selected.is_none_or(|old| old == zone),
            "Ambiguous encounter activation; specify the physical trigger binding",
        )?;
        selected = Some(zone);
    }
    if (binding.is_some() || known) && selected.is_none() {
        return Err(SimError("Encounter binding missing or inactive".into()));
    }
    Ok(ActiveEncounter {
        map: map.into(),
        base: base.into(),
        zone: selected.unwrap_or(base).into(),
    })
}
impl crate::world::WorldState {
    /// Call only for an actual trigger entry/canoe transition; no RNG is consumed.
    pub fn enter_encounter_zone(&mut self, base: &str) -> Result<()> {
        self.active_encounter = Some(activate(&self.map, base, &self.flags, None)?);
        Ok(())
    }
    pub fn enter_encounter_binding(&mut self, base: &str, binding: &str) -> Result<()> {
        self.active_encounter = Some(activate(&self.map, base, &self.flags, Some(binding))?);
        Ok(())
    }
    pub fn encounter_zone<'a>(&'a self, base: &'a str) -> Result<&'a str> {
        if let Some(a) = &self.active_encounter {
            if a.map == self.map && a.base == base {
                return Ok(&a.zone);
            }
        }
        ensure(
            !bindings()
                .iter()
                .any(|b| b.map == self.map && b.base == base),
            "Replacement-bearing encounter requires an explicit trigger activation checkpoint",
        )?;
        Ok(base)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_binding_retains_selected_table_until_reactivated() {
        for b in bindings() {
            let mut flags: BTreeSet<_> = b.start_flags.iter().copied().collect();
            let initial = activate(&b.map, &b.base, &flags, Some(&b.key)).unwrap();
            for replacement in &b.replacements {
                flags.insert(replacement.flag);
                assert_eq!(
                    initial.zone, b.base,
                    "flag change must not mutate active table"
                );
                let again = activate(&b.map, &b.base, &flags, Some(&b.key)).unwrap();
                assert_eq!(
                    again.zone,
                    b.replacements
                        .iter()
                        .find(|r| flags.contains(&r.flag))
                        .unwrap()
                        .zone
                );
            }
        }
    }
    #[test]
    fn world_preserves_activation_across_flags_time_and_battle_return() {
        let f: serde_json::Value =
            serde_json::from_str(include_str!("../data/source/quicksand-post-boss-live.json"))
                .unwrap();
        let w: crate::world::WorldState = serde_json::from_value(f["world_after"].clone()).unwrap();
        let mut l = crate::world::Loader::new(
            crate::world::catalog(),
            w,
            serde_json::from_value(f["after"].clone()).unwrap(),
        )
        .unwrap();
        let base = "ENT_DNG_DST_2_2_LND";
        let active = l.state.active_encounter.clone();
        l.state.set_flag(40010, true);
        l.manual_time_switch(2, "activation regression").unwrap();
        l.restore_field_after_battle("activation regression")
            .unwrap();
        assert_eq!(l.state.active_encounter, active);
        let prior = crate::state_key::WorldKey::new(&l.state);
        let rng = l.rng;
        l.state.enter_encounter_zone(base).unwrap();
        assert_eq!(rng, l.rng);
        assert_eq!(
            l.state.encounter_zone(base).unwrap(),
            "ENT_DNG_DST_2_2_L_LND"
        );
        assert_ne!(prior, crate::state_key::WorldKey::new(&l.state));
        l.state.active_encounter = None;
        assert!(l.state.encounter_zone(base).is_err());
    }
    #[test]
    fn nameless_land_water_and_reentry() {
        let mut flags = BTreeSet::new();
        let land = activate("Dng_Ocn_1_3", "ENT_DNG_OCN_1_3_LND", &flags, None).unwrap();
        flags.insert(40003);
        assert_eq!(land.zone, "ENT_DNG_OCN_1_3_LND");
        let water = activate("Dng_Ocn_1_3", "ENT_DNG_OCN_1_3_WTR", &flags, None).unwrap();
        assert_eq!(water.zone, "ENT_DNG_OCN_1_3_WTR");
        assert_eq!(
            activate("Dng_Ocn_1_3", "ENT_DNG_OCN_1_3_LND", &flags, None)
                .unwrap()
                .zone,
            "ENT_DNG_OCN_1_3_L_LND"
        );
    }
}
