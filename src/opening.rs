//! Captured title -> New Game as Throné, skipping both opening cutscenes.
//! Headless CRT projection only; this does not render or load game objects.
use crate::{CrtRng, Result, SimError};
use serde::Serialize;

pub const PROFILE: &str = "throne_title_skip_both_v1";

pub fn transition_draws(recreate_shared_level: bool) -> u64 {
    12 + u64::from(recreate_shared_level)
}

#[derive(Debug, Serialize)]
pub struct TitleCheckpoint {
    pub label: &'static str,
    pub state: u32,
    pub transition_draws: u64,
    /// Decimal text retains full u64 precision in GUI and exports.
    pub total_advances: String,
}

#[derive(Debug, Serialize)]
pub struct TitleTransition {
    pub profile: &'static str,
    pub title_state: u32,
    pub draws: u64,
    pub battle_state: u32,
    pub battle_entry_advances: u64,
    pub checkpoints: Vec<TitleCheckpoint>,
}

pub fn prepare(
    initial_seed: u32,
    title_advances: u64,
    recreate_shared_level: bool,
) -> Result<TitleTransition> {
    let draws = transition_draws(recreate_shared_level);
    let battle_entry_advances = title_advances.checked_add(draws).ok_or_else(|| {
        SimError("Title advances plus opening transition exceed 18446744073709551615.".into())
    })?;
    let mut seed = CrtRng::new(initial_seed);
    seed.advance(title_advances);
    let title_state = seed.state;
    let mut rng = CrtRng::new(title_state);
    let mut checkpoints = Vec::new();
    // The shared entry is the last streamed level in the captured construction
    // order. Its retained path skips only this color draw, not any other phase.
    for (label, count) in [
        ("Title screen · before New Game", 0),
        ("Opening timer", 1),
        ("Field and battle level initialization", 5),
        (
            if recreate_shared_level {
                "Shared battle level recreated"
            } else {
                "Shared battle level retained (no draw)"
            },
            u64::from(recreate_shared_level),
        ),
        ("Field NPC starting frame", 1),
        ("First opening counter reset (B, A)", 2),
        ("Time-change flag lottery", 1),
        ("Second opening counter reset (B, A) · battle entry", 2),
    ] {
        rng.advance(count);
        checkpoints.push(TitleCheckpoint {
            label,
            state: rng.state,
            transition_draws: rng.draws,
            total_advances: (title_advances + rng.draws).to_string(),
        });
    }
    debug_assert_eq!(rng.draws, draws);
    Ok(TitleTransition {
        profile: PROFILE,
        title_state,
        draws,
        battle_state: rng.state,
        battle_entry_advances,
        checkpoints,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ot2_battle_engine::{Battle, BattleConfig, ItemCatalog};

    #[test]
    fn both_captured_transitions_reach_the_recorded_first_menu() {
        let fixtures: serde_json::Value =
            serde_json::from_str(include_str!("../data/source/title-transitions.json")).unwrap();
        for capture in fixtures["captures"].as_array().unwrap() {
            let p = prepare(
                0,
                capture["title_advances"].as_u64().unwrap(),
                capture["recreate_shared_level"].as_bool().unwrap(),
            )
            .unwrap();
            assert_eq!(p.draws, capture["records"].as_array().unwrap().len() as u64);
            let mut rng = CrtRng::new(p.title_state);
            for (i, row) in capture["records"].as_array().unwrap().iter().enumerate() {
                assert_eq!(row["before"], rng.state, "call {i}");
                rng.next_u15();
                if let Some(c) = p
                    .checkpoints
                    .iter()
                    .find(|c| c.transition_draws == i as u64 + 1)
                {
                    assert_eq!(c.state, rng.state);
                }
            }
            assert_eq!(rng.state, p.battle_state);
            assert_eq!(p.battle_state, 0x0EA79703);
            assert_eq!(p.battle_entry_advances, 4929);
            let battle = Battle::new_opening(
                BattleConfig::throne_opening(p.battle_state).unwrap(),
                ItemCatalog::builtin().unwrap(),
                false,
            )
            .unwrap();
            assert_eq!(battle.state().rng.state, 0xF485D421);
            assert_eq!(p.battle_entry_advances + battle.state().rng.draws, 4959);
            assert_eq!(battle.current_actor(), battle.actor_id("pirro"));
        }
    }

    #[test]
    fn recreation_adds_exactly_one_draw_at_the_level_phase() {
        let retained = prepare(0, 4916, false).unwrap();
        let recreated = prepare(0, 4916, true).unwrap();
        assert_eq!((retained.draws, recreated.draws), (12, 13));
        assert_eq!(
            retained.checkpoints[2].state,
            recreated.checkpoints[2].state
        );
        assert_eq!(retained.checkpoints[2].state, retained.checkpoints[3].state);
        let mut rng = CrtRng::new(retained.battle_state);
        rng.advance(1);
        assert_eq!(rng.state, recreated.battle_state);
        assert!(prepare(0, u64::MAX - 11, false).is_err());
        assert!(prepare(0, u64::MAX - 12, true).is_err());
        assert_eq!(
            prepare(0, u64::MAX - 13, true)
                .unwrap()
                .battle_entry_advances,
            u64::MAX
        );
    }
}
