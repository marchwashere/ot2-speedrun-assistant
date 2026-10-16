//! Field counter and encounter selection, ending before enemy actor initialization.
//! Story flags/physical zone membership are supplied by the caller's active zone.
use crate::{CrtRng, Result, SimError, ensure};
use ot2_battle_engine::opening_crt::EncounterCounter;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Debug, Deserialize)]
pub struct EncounterCatalog {
    pub maps: BTreeMap<String, Map>,
    pub zones: BTreeMap<String, Zone>,
    pub volumes: BTreeMap<String, Volume>,
    pub encounters: BTreeMap<String, EncounterTable>,
    pub groups: BTreeMap<String, Group>,
    pub enemies: BTreeMap<String, Enemy>,
    pub rare_labels: BTreeMap<String, [String; 3]>,
    pub dash_multiplier: f32,
    pub coverage: Value,
}
#[derive(Debug, Deserialize)]
pub struct Map {
    pub id: String,
    pub name: String,
    pub counter: EncounterCounter,
    pub zones: Vec<ZoneLink>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct ZoneLink {
    pub id: String,
    pub mapping: String,
    pub conditions: Value,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Zone {
    pub id: String,
    pub flags: Vec<i32>,
    pub periods: BTreeMap<String, Period>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Period {
    pub volume: String,
    pub decrement: u32,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Volume {
    pub id: String,
    pub area: String,
    pub water: bool,
    pub flags: Vec<i32>,
    pub cait: Rare,
    pub octopuff: Rare,
    pub encounters: Vec<ProgressRow>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Rare {
    pub enabled: bool,
    pub rate: u32,
    pub counts: [u32; 3],
}
#[derive(Debug, Deserialize, Serialize)]
pub struct ProgressRow {
    pub progress: u32,
    pub level: u32,
    pub danger: u32,
    pub table: String,
}
#[derive(Debug, Deserialize)]
pub struct EncounterTable {
    pub id: String,
    pub first: i32,
    pub groups: Vec<WeightedGroup>,
}
#[derive(Debug, Deserialize)]
pub struct WeightedGroup {
    pub id: String,
    pub weight: u32,
}
#[derive(Debug, Deserialize)]
pub struct Group {
    pub id: String,
    pub enemies: Vec<String>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Enemy {
    pub id: String,
    pub name: String,
    pub level: u32,
    pub size: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Boundary {
    Reset,
    Trigger,
    Walk,
    CurrentCounter,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct PredictionOptions {
    pub map: String,
    pub zone: String,
    pub period: String,
    pub progress: u32,
    pub first_encounter: bool,
    pub boundary: Boundary,
    pub party_levels: Vec<u32>,
    /// Additive percentage points, already aggregated from eligible equipment.
    pub cait_bonus: u32,
    pub octopuff_bonus: u32,
    pub rare_support_bonus: u32,
    pub preemptive_bonus: u32,
    pub surprise_reduction: u32,
    pub dash: bool,
    pub ability_percent: u32,
    pub equipment_percent: u32,
    /// CRT calls between the initial reset and encounter trigger, in Walk mode.
    pub intervening_advances: String,
    pub current_counter: Option<u32>,
    pub allow_unmapped_zone: bool,
}
impl Default for PredictionOptions {
    fn default() -> Self {
        Self {
            map: "Fld_Cty_3_1".into(),
            zone: "ENT_FLD_CTY_3_1_LND".into(),
            period: "DayTime".into(),
            progress: 0,
            first_encounter: false,
            boundary: Boundary::Walk,
            party_levels: vec![1],
            cait_bonus: 0,
            octopuff_bonus: 0,
            rare_support_bonus: 0,
            preemptive_bonus: 0,
            surprise_reduction: 0,
            dash: false,
            ability_percent: 100,
            equipment_percent: 100,
            intervening_advances: "0".into(),
            current_counter: None,
            allow_unmapped_zone: false,
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Checkpoint {
    pub label: String,
    pub state: u32,
    pub draws: u64,
}
#[derive(Debug, Clone, Serialize)]
pub struct CounterRoll {
    pub minimum: u32,
    pub roll_b: u32,
    pub roll_a: u32,
    pub ratio: f32,
    pub value: u32,
    pub supplied: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct Prediction {
    pub map: String,
    pub map_name: String,
    pub volume: Option<String>,
    pub counter: CounterRoll,
    pub trigger_reset: Option<CounterRoll>,
    pub decrement: Option<u32>,
    pub counter_updates: Option<u32>,
    pub encounter_table: Option<String>,
    pub encounter_level: Option<u32>,
    pub group: Option<String>,
    pub group_roll: Option<u32>,
    pub first_override: bool,
    pub situation: Option<String>,
    pub situation_roll: Option<u32>,
    pub rare_family: Option<String>,
    pub enemies: Vec<EnemySlot>,
    pub checkpoints: Vec<Checkpoint>,
    pub final_state: u32,
    pub draws: u64,
    pub warnings: Vec<String>,
    pub endpoint: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct EnemySlot {
    pub slot: usize,
    pub original_id: String,
    pub enemy: Enemy,
}

fn lookup<'a, T>(data: &'a BTreeMap<String, T>, key: &str, kind: &str) -> Result<&'a T> {
    data.get(key).ok_or_else(|| {
        SimError(format!(
            "Missing {kind}: {key}. No prediction was produced."
        ))
    })
}
pub fn catalog() -> &'static EncounterCatalog {
    static DATA: OnceLock<EncounterCatalog> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../data/encounters.json"))
            .expect("imported encounter catalog")
    })
}
impl EncounterCatalog {
    /// Same truncated binary32 decrement used by PlayerStepCounter. No RNG.
    pub fn counter_decrement(&self, o: &PredictionOptions) -> Result<u32> {
        let zone = lookup(&self.zones, &o.zone, "encounter zone")?;
        let period = lookup(&zone.periods, &o.period, "time period")?;
        Ok(self.decrement_value(period, o))
    }

    fn decrement_value(&self, period: &Period, o: &PredictionOptions) -> u32 {
        (((period.decrement as f32 * (o.ability_percent as f32 * 0.01_f32))
            * (o.equipment_percent as f32 * 0.01_f32))
            * if o.dash { self.dash_multiplier } else { 1.0 }) as u32
    }

    pub fn menu(&self) -> Value {
        json!({"maps":self.maps.values().map(|m|json!({"id":m.id,"name":m.name,"zones":m.zones,
            "counter":{"min":m.counter.min,"a":m.counter.range_a,"b":m.counter.range_b,"first":m.counter.first_ratio}})).collect::<Vec<_>>(),
            "zones":self.zones,"volumes":self.volumes,"coverage":{
                "maps":self.maps.len(),"zones":self.zones.len(),"volumes":self.volumes.len(),
                "mapped_maps":self.coverage["mappedMaps"],"unresolved_zones":self.coverage["unmatched"]}})
    }

    /// Transactional: unsupported rows never change the caller's RNG.
    pub fn predict(&self, rng: &mut CrtRng, o: &PredictionOptions) -> Result<Prediction> {
        crate::cancellation::check()?;
        let mut work = CrtRng::new(rng.state);
        let result = self.predict_inner(&mut work, o)?;
        ensure(
            rng.draws.checked_add(work.draws).is_some(),
            "CRT draw count overflow",
        )?;
        rng.state = work.state;
        rng.draws += work.draws;
        Ok(result)
    }

    fn predict_inner(&self, rng: &mut CrtRng, o: &PredictionOptions) -> Result<Prediction> {
        let map = lookup(&self.maps, &o.map, "map")?;
        ensure(
            (1..=4).contains(&o.party_levels.len())
                && o.party_levels.iter().all(|&n| (1..=99).contains(&n)),
            "Enter one to four party levels, each 1–99",
        )?;
        ensure(
            [
                o.cait_bonus,
                o.octopuff_bonus,
                o.rare_support_bonus,
                o.preemptive_bonus,
                o.surprise_reduction,
                o.ability_percent,
                o.equipment_percent,
            ]
            .iter()
            .all(|&n| n <= 1000),
            "Modifiers must be between 0 and 1000",
        )?;
        let gap = o
            .intervening_advances
            .parse::<u64>()
            .map_err(|_| SimError("Invalid intervening CRT advance count".into()))?;
        ensure(
            gap <= u64::MAX - 128,
            "Intervening CRT advance count is too large",
        )?;
        let walking = matches!(o.boundary, Boundary::Walk | Boundary::CurrentCounter);
        ensure(
            walking || gap == 0,
            "Intervening advances apply only to walking predictions",
        )?;
        let mut checkpoints = vec![checkpoint("Starting state", rng)];
        let counter = if o.boundary == Boundary::CurrentCounter {
            CounterRoll {
                minimum: 0,
                roll_b: 0,
                roll_a: 0,
                ratio: 1.0,
                supplied: true,
                value: o
                    .current_counter
                    .ok_or_else(|| SimError("Enter the current counter value".into()))?,
            }
        } else {
            let c = reset(&map.counter, rng, o.first_encounter);
            checkpoints.push(checkpoint(
                if o.boundary == Boundary::Trigger {
                    "Encounter-trigger counter reset (B, A)"
                } else {
                    "Field counter reset (B, A)"
                },
                rng,
            ));
            c
        };
        let mut p = Prediction {
            map: map.id.clone(),
            map_name: map.name.clone(),
            volume: None,
            counter,
            trigger_reset: None,
            decrement: None,
            counter_updates: None,
            encounter_table: None,
            encounter_level: None,
            group: None,
            group_roll: None,
            first_override: false,
            situation: None,
            situation_roll: None,
            rare_family: None,
            enemies: vec![],
            checkpoints,
            final_state: rng.state,
            draws: rng.draws,
            warnings: vec![],
            endpoint: "After counter reset".into(),
        };
        if o.boundary == Boundary::Reset {
            return Ok(p);
        }
        let zone = lookup(&self.zones, &o.zone, "encounter zone")?;
        let link = map.zones.iter().find(|z| z.id == o.zone);
        ensure(
            link.is_some() || o.allow_unmapped_zone,
            "Zone is not linked to this map. Explicit manual override is required",
        )?;
        if link.is_none() {
            p.warnings
                .push("Manual map/zone association; no placement link was found.".into());
        }
        if link.is_some_and(|z| z.mapping == "inferred") {
            p.warnings.push(
                "Zone/map association is inferred from its ID; placement was not recovered.".into(),
            );
        }
        let period = lookup(&zone.periods, &o.period, "time period")?;
        let volume = lookup(&self.volumes, &period.volume, "active encounter volume")?;
        if zone
            .flags
            .iter()
            .chain(volume.flags.iter())
            .any(|&f| f != 0)
            || link.is_some_and(|z| z.conditions.as_array().is_some_and(|a| !a.is_empty()))
        {
            p.warnings.push("This zone has story/placement conditions. Prediction assumes the selected zone is currently active.".into());
        }
        let row = volume
            .encounters
            .iter()
            .rev()
            .find(|r| r.table != "None" && r.progress <= o.progress)
            .or_else(|| volume.encounters.first().filter(|r| r.table != "None"))
            .ok_or_else(|| SimError("No encounter table for this zone/time".into()))?;
        let table = lookup(&self.encounters, &row.table, "encounter table")?;
        p.volume = Some(volume.id.clone());
        p.encounter_table = Some(row.table.clone());
        p.encounter_level = Some(row.level);
        let decrement = self.decrement_value(period, o);
        p.decrement = Some(decrement);
        if walking {
            ensure(
                decrement > 0 || p.counter.value == 0,
                "Counter does not decrease with these settings; no walking encounter can trigger",
            )?;
            p.counter_updates = Some(if p.counter.value == 0 {
                0
            } else {
                p.counter.value.div_ceil(decrement)
            });
            rng.advance(gap);
            if gap > 0 {
                p.checkpoints
                    .push(checkpoint("Supplied intervening CRT calls", rng));
            }
            p.trigger_reset = Some(reset(&map.counter, rng, o.first_encounter));
            p.checkpoints
                .push(checkpoint("Encounter-trigger counter reset (B, A)", rng));
            p.warnings.push(format!("Assumes the same active zone/time/modifiers until the encounter, no intervening counter resets, and exactly {gap} other CRT calls. Movement itself consumes no CRT."));
        }
        let (selected, roll, fixed) = lottery(table, rng, o.first_encounter)?;
        p.first_override = fixed;
        p.group_roll = roll;
        p.group = Some(selected.to_owned());
        let group = lookup(&self.groups, selected, "enemy group")?;
        for (slot, id) in group
            .enemies
            .iter()
            .enumerate()
            .filter(|(_, id)| id.as_str() != "None")
        {
            p.enemies.push(EnemySlot {
                slot,
                original_id: id.clone(),
                enemy: lookup(&self.enemies, id, "enemy")?.clone(),
            });
        }
        ensure(!p.enemies.is_empty(), "Selected group has no enemies")?;
        p.checkpoints.push(checkpoint(
            if fixed {
                "Fixed first encounter (no group draw)"
            } else {
                "Weighted enemy group selection"
            },
            rng,
        ));
        let enemy_average =
            p.enemies.iter().map(|e| e.enemy.level as i32).sum::<i32>() / p.enemies.len() as i32;
        let party_average = o.party_levels.iter().sum::<u32>() as i32 / o.party_levels.len() as i32;
        let advantage =
            5 + (party_average - enemy_average).clamp(-4, 5) + o.preemptive_bonus as i32;
        let back_threshold = 100 - (5 + 4 * (enemy_average - party_average).clamp(-1, 6))
            + o.surprise_reduction as i32;
        let roll = (rng.index(100) + 1) as i32;
        p.situation = Some(
            if roll <= advantage {
                "Preemptive"
            } else if roll > back_threshold {
                "Surprise"
            } else {
                "Normal"
            }
            .into(),
        );
        p.situation_roll = Some(roll as u32);
        p.checkpoints.push(checkpoint("Battle situation", rng));
        // BattlePlayerBP.Initialize: one fixed-range timing draw per traveler.
        rng.advance(o.party_levels.len() as u64);
        p.checkpoints
            .push(checkpoint("Party timing initialization", rng));
        let cait = volume.cait.rate + o.cait_bonus;
        let oct = volume.octopuff.rate + o.octopuff_bonus;
        let selected_family = match (volume.cait.enabled, volume.octopuff.enabled) {
            (true, true) => {
                let gate = rng.weighted_bool((cait + oct + o.rare_support_bonus) as f32 * 0.01_f32);
                p.checkpoints.push(checkpoint("Rare enemy gate", rng));
                if gate {
                    ensure(cait + oct > 0, "Rare family weights are zero")?;
                    let cat = rng.weighted_bool(cait as f32 / (cait + oct) as f32);
                    p.checkpoints
                        .push(checkpoint("Cait / Octopuff selection", rng));
                    Some(cat)
                } else {
                    None
                }
            }
            (true, false) | (false, true) => {
                let cat = volume.cait.enabled;
                let gate = rng.weighted_bool(
                    ((if cat { cait } else { oct }) + o.rare_support_bonus) as f32 * 0.01_f32,
                );
                p.checkpoints
                    .push(checkpoint("Rare enemy gate (one family enabled)", rng));
                gate.then_some(cat)
            }
            (false, false) => None,
        };
        if let Some(cat) = selected_family {
            p.rare_family = Some(if cat { "Cait" } else { "Octopuff" }.into());
            let labels = lookup(
                &self.rare_labels,
                &format!("{}_{}", if cat { "CAT" } else { "OCTOPUS" }, volume.area),
                "regional rare labels",
            )?;
            let rare = if cat { &volume.cait } else { &volume.octopuff };
            replace_rare(self, &mut p.enemies, rare, labels, cat, rng)?;
            p.checkpoints
                .push(checkpoint("Rare replacement slots", rng));
        }
        p.endpoint="Enemy composition finalized; before enemy actor initialization and opening turn-order/voice draws".into();
        p.final_state = rng.state;
        p.draws = rng.draws;
        Ok(p)
    }
}
fn checkpoint(label: &str, rng: &CrtRng) -> Checkpoint {
    Checkpoint {
        label: label.into(),
        state: rng.state,
        draws: rng.draws,
    }
}
fn reset(c: &EncounterCounter, rng: &mut CrtRng, first: bool) -> CounterRoll {
    let b = rng.index(c.range_b);
    let a = rng.index(c.range_a);
    let ratio = if first { c.first_ratio } else { 1.0 };
    CounterRoll {
        minimum: c.min,
        roll_b: b,
        roll_a: a,
        ratio,
        value: ((c.min + a + b) as f32 * ratio) as u32,
        supplied: false,
    }
}
fn lottery<'a>(
    t: &'a EncounterTable,
    rng: &mut CrtRng,
    first: bool,
) -> Result<(&'a str, Option<u32>, bool)> {
    if first && t.first > 0 {
        if let Some(g) = t.groups.get((t.first - 1) as usize) {
            return Ok((&g.id, None, true));
        }
    }
    let sum = t
        .groups
        .iter()
        .try_fold(0u32, |s, g| s.checked_add(g.weight))
        .ok_or_else(|| SimError("Encounter weight overflow".into()))?;
    ensure(sum > 0, "Encounter table has no positive group weights")?;
    let roll = rng.index(sum) + 1;
    let mut remaining = roll;
    for g in &t.groups {
        if remaining <= g.weight {
            return Ok((&g.id, Some(roll), false));
        }
        remaining -= g.weight;
    }
    Err(SimError("Invalid encounter weights".into()))
}
fn replace_rare(
    c: &EncounterCatalog,
    enemies: &mut [EnemySlot],
    r: &Rare,
    labels: &[String; 3],
    cat: bool,
    rng: &mut CrtRng,
) -> Result<()> {
    let mut small: Vec<_> = enemies
        .iter()
        .enumerate()
        .filter(|(_, e)| e.enemy.size == "eSMALL")
        .map(|(i, _)| i)
        .collect();
    let mut medium: Vec<_> = enemies
        .iter()
        .enumerate()
        .filter(|(_, e)| e.enemy.size == "eMEDIUM")
        .map(|(i, _)| i)
        .collect();
    for (tier, &count) in r.counts.iter().enumerate() {
        for _ in 0..count {
            let eligible = if cat {
                if tier == 2 { &mut medium } else { &mut small }
            } else if !small.is_empty() {
                &mut small
            } else {
                &mut medium
            };
            if eligible.is_empty() {
                continue;
            }
            let replacement = lookup(&c.enemies, &labels[tier], "rare enemy")?;
            let slot = eligible.remove(rng.index(eligible.len() as u32) as usize);
            enemies[slot].enemy = replacement.clone();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_override_skips_weighted_draw_and_invalid_index_falls_back() {
        let mut t = EncounterTable {
            id: "test".into(),
            first: 2,
            groups: vec![
                WeightedGroup {
                    id: "A".into(),
                    weight: 100,
                },
                WeightedGroup {
                    id: "B".into(),
                    weight: 0,
                },
            ],
        };
        let mut rng = CrtRng::new(0);
        assert_eq!(lottery(&t, &mut rng, true).unwrap(), ("B", None, true));
        assert_eq!(rng.draws, 0);
        t.first = 3;
        assert_eq!(lottery(&t, &mut rng, true).unwrap().0, "A");
        assert_eq!(rng.draws, 1);
        t.groups[0].weight = 0;
        assert!(lottery(&t, &mut rng, false).is_err());
        assert_eq!(rng.draws, 1);
    }
    #[test]
    fn rare_slots_follow_size_priority_without_reuse_or_ineligible_draws() {
        let c = catalog();
        let small = c
            .enemies
            .values()
            .find(|e| e.size == "eSMALL")
            .unwrap()
            .clone();
        let medium = c
            .enemies
            .values()
            .find(|e| e.size == "eMEDIUM")
            .unwrap()
            .clone();
        let slots = vec![
            EnemySlot {
                slot: 0,
                original_id: medium.id.clone(),
                enemy: medium.clone(),
            },
            EnemySlot {
                slot: 1,
                original_id: small.id.clone(),
                enemy: small,
            },
        ];
        let mut cats = slots;
        let mut rng = CrtRng::new(123);
        let labels = &c.rare_labels["CAT_CITY"];
        replace_rare(
            c,
            &mut cats,
            &Rare {
                enabled: true,
                rate: 100,
                counts: [1, 1, 1],
            },
            labels,
            true,
            &mut rng,
        )
        .unwrap();
        assert_eq!(cats[1].enemy.id, labels[0]);
        assert_eq!(cats[0].enemy.id, labels[2]);
        assert_eq!(rng.draws, 2);
        let mut octs = vec![EnemySlot {
            slot: 0,
            original_id: medium.id.clone(),
            enemy: medium,
        }];
        let labels = &c.rare_labels["OCTOPUS_CITY"];
        replace_rare(
            c,
            &mut octs,
            &Rare {
                enabled: true,
                rate: 100,
                counts: [1, 1, 1],
            },
            labels,
            false,
            &mut rng,
        )
        .unwrap();
        assert_eq!(octs[0].enemy.id, labels[0]);
        assert_eq!(rng.draws, 3);
    }
}
