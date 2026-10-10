//! Exact owned search keys. Exhaustive destructuring keeps new state fields from
//! silently disappearing from cache identity when the world/core models grow.
use crate::world::{CharacterProgression, WorldState};
use ot2_battle_engine::{PlayerCharacter, Resources, Stats, combat::{CombatProfile, EnemyInitialization, Status, WeaponStats}, progression::Growth};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct EnemyHistoryKey(Vec<(String, u8, u8, bool)>);
impl EnemyHistoryKey {
    pub(crate) fn new(history: &ot2_battle_engine::knowledge::EnemyHistory) -> Self {
        Self(history.iter().map(|(id, known)| {
            let ot2_battle_engine::knowledge::EnemyKnowledge { weapons, elements, studied } = known;
            (id.clone(), *weapons, *elements, *studied)
        }).collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct WorldKey {
    map: String,
    visited_maps: BTreeSet<String>,
    time: u8,
    flags: BTreeSet<u32>,
    event_variables: BTreeMap<i32, i32>,
    party: Vec<u32>,
    bench: Vec<u32>,
    story_types: BTreeSet<i32>,
    active_scenarios: BTreeSet<u32>,
    main_progress_points: u32,
    completed_scenarios: BTreeSet<u32>,
    confirmed_scenarios: BTreeSet<u32>,
    prologues: BTreeSet<String>,
    resident_levels: BTreeSet<String>,
    shared_levels: BTreeSet<String>,
    existing_npcs: BTreeSet<String>,
    npc_start_delays: BTreeMap<String, i32>,
    npc_end_delays: BTreeMap<String, i32>,
    first_encounter: bool,
    encounter_counter: u32,
    evasive_maneuvers: bool,
    osvald_extra_skills: u8,
    party_experience: Vec<u32>,
    character_progression: BTreeMap<u32, CharacterKey>,
    job_licenses:BTreeMap<u32,u8>,
    inventor_abilities:BTreeSet<String>,
    enemy_knowledge: EnemyHistoryKey,
    suppress_flag_callbacks: bool,
    suppress_level_lottery: bool,
    suppress_time_callbacks: bool,
}
impl WorldKey {
    pub(crate) fn new(world: &WorldState) -> Self {
        let WorldState { map, visited_maps, time, flags, event_variables, party, bench,
            story_types, active_scenarios, main_progress_points, completed_scenarios, confirmed_scenarios,
            prologues, resident_levels, shared_levels, existing_npcs, npc_start_delays,
            npc_end_delays, first_encounter, encounter_counter, evasive_maneuvers,
            osvald_extra_skills, party_experience, character_progression, job_licenses, inventor_abilities, enemy_knowledge,
            suppress_flag_callbacks, suppress_level_lottery, suppress_time_callbacks } = world;
        Self {
            map: map.clone(), visited_maps: visited_maps.clone(), time: *time, flags: flags.clone(),
            event_variables: event_variables.clone(), party: party.clone(), bench: bench.clone(),
            story_types: story_types.clone(), active_scenarios: active_scenarios.clone(),
            main_progress_points: *main_progress_points, completed_scenarios: completed_scenarios.clone(),
            confirmed_scenarios: confirmed_scenarios.clone(),
            prologues: prologues.clone(), resident_levels: resident_levels.clone(), shared_levels: shared_levels.clone(),
            existing_npcs: existing_npcs.clone(), npc_start_delays: npc_start_delays.clone(), npc_end_delays: npc_end_delays.clone(),
            first_encounter: *first_encounter, encounter_counter: *encounter_counter,
            evasive_maneuvers: *evasive_maneuvers, osvald_extra_skills: *osvald_extra_skills,
            party_experience: party_experience.clone(),
            character_progression: character_progression.iter().map(|(&id, p)| (id, CharacterKey::new(p))).collect(),
            job_licenses:job_licenses.clone(),
            inventor_abilities:inventor_abilities.clone(),
            enemy_knowledge: EnemyHistoryKey::new(enemy_knowledge), suppress_flag_callbacks: *suppress_flag_callbacks,
            suppress_level_lottery: *suppress_level_lottery, suppress_time_callbacks: *suppress_time_callbacks,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CharacterKey {
    experience: u32,
    jp_earned: u32,
    jp_spent: u32,
    level: u32,
    growth: (u32, u32, BTreeMap<String, u32>, u32, u32),
    id: String,
    name: String,
    stats: (u32, u32, u32),
    resources: Option<(u32, u32, u8, u16)>,
    skills: Vec<String>,
    combat: Option<CombatKey>,
    jobs:crate::world::CharacterJobs,
}
impl CharacterKey {
    fn new(p: &CharacterProgression) -> Self {
        let CharacterProgression { experience, jp_earned, jp_spent, level, growth, character,jobs } = p;
        let Growth { level: growth_level, total_exp, parameters, growth_attack, growth_magic } = growth;
        let PlayerCharacter { id, name, stats, resources, skills, combat } = character;
        let Stats { max_hp, max_sp, speed } = stats;
        Self {
            experience: *experience, jp_earned: *jp_earned, jp_spent: *jp_spent, level: *level,
            growth: (*growth_level, *total_exp, parameters.clone(), *growth_attack, *growth_magic),
            id: id.clone(), name: name.clone(), stats: (*max_hp, *max_sp, *speed),
            resources: resources.as_ref().map(|Resources { hp, sp, bp, latent }| (*hp, *sp, *bp, *latent)),
            skills: skills.clone(), combat: combat.as_ref().map(CombatKey::new),
            jobs:jobs.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CombatKey {
    use_cat_damage_type: bool,
    enemy_initialization: Option<(u32, bool, bool)>,
    stats: [u32; 9],
    weapons: Vec<(u8, u32, u32)>,
    main_weapon: Option<u8>,
    shields: u8,
    weapon_weaknesses: Vec<u8>,
    element_weaknesses: Vec<u8>,
    statuses: Vec<(u8, u8)>,
    steal_item: Option<String>,
    enemy_bp_base: u8,
}
impl CombatKey {
    fn new(c: &CombatProfile) -> Self {
        let CombatProfile { use_cat_damage_type, enemy_initialization, attack, defense, magic, magic_defense,
            accuracy, evasion, critical, growth_attack, growth_magic, weapons, main_weapon,
            shields, weapon_weaknesses, element_weaknesses, statuses, steal_item, enemy_bp_base } = c;
        Self {
            use_cat_damage_type: *use_cat_damage_type,
            enemy_initialization: enemy_initialization.as_ref().map(|EnemyInitialization { initial_idle_frames, sync_to_parent, sync_children }| (*initial_idle_frames, *sync_to_parent, *sync_children)),
            stats: [*attack, *defense, *magic, *magic_defense, *accuracy, *evasion, *critical, *growth_attack, *growth_magic],
            weapons: weapons.iter().map(|WeaponStats { weapon, attack, magic }| (*weapon as u8, *attack, *magic)).collect(),
            main_weapon: main_weapon.map(|w| w as u8), shields: *shields,
            weapon_weaknesses: weapon_weaknesses.iter().map(|&w| w as u8).collect(),
            element_weaknesses: element_weaknesses.iter().map(|&e| e as u8).collect(),
            statuses: statuses.iter().map(|Status { kind, turns }| (*kind as u8, *turns)).collect(),
            steal_item: steal_item.clone(), enemy_bp_base: *enemy_bp_base,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_world_key_distinguishes_serialized_state_changes() {
        let mut world = crate::world::throne_after_second_state().unwrap();
        world.party = vec![7, 6, 8];
        world.party_experience = vec![0; 3];
        world.sync_progression().unwrap();
        world.job_licenses.insert(6,1);
        let jobs=&mut world.character_progression.get_mut(&7).unwrap().jobs;
        jobs.secondary_job=Some(6);
        jobs.learned_abilities.insert(6,BTreeSet::from(["ABI_SET_SCH_030".into()]));
        jobs.unlocked_support.insert("ABI_SCH_SUP_01".into());
        jobs.equipped_support.insert("ABI_SCH_SUP_01".into());
        world.enemy_knowledge.insert("test enemy".into(), Default::default());
        let original = serde_json::to_value(&world).unwrap();
        let key = WorldKey::new(&world);
        assert_eq!(key, WorldKey::new(&serde_json::from_value(original.clone()).unwrap()));
        fn changed_leaves(value: &serde_json::Value) -> Vec<serde_json::Value> {
            use serde_json::Value;
            match value {
                Value::Object(fields) => fields.iter().flat_map(|(name, value)| {
                    changed_leaves(value).into_iter().map(|changed| {
                        let mut copy = fields.clone(); copy.insert(name.clone(), changed); Value::Object(copy)
                    }).collect::<Vec<_>>()
                }).collect(),
                Value::Array(items) => items.iter().enumerate().flat_map(|(i, value)| {
                    changed_leaves(value).into_iter().map(|changed| {
                        let mut copy = items.clone(); copy[i] = changed; Value::Array(copy)
                    }).collect::<Vec<_>>()
                }).collect(),
                Value::Bool(v) => vec![Value::Bool(!v)],
                Value::Number(v) => vec![Value::from(v.as_i64().unwrap() + 1)],
                Value::String(v) => vec![Value::String(format!("{v} changed"))],
                Value::Null => vec![],
            }
        }
        let mut checked = 0;
        for changed in changed_leaves(&original) {
            // Invalid enum spellings are not representable world states.
            if let Ok(other) = serde_json::from_value::<WorldState>(changed.clone()) {
                assert_ne!(key, WorldKey::new(&other), "Key omitted a changed state field: {changed}");
                checked += 1;
            }
        }
        assert!(checked > 50, "Exercise nested carried progression as well as field state");
    }
}
