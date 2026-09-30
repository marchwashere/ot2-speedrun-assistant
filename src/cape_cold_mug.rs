//! Cape Cold daytime arrival -> nighttime placements -> Mug -> field control.
//! Original-gear level-one Osvald, first Mug, no intervening menu reopens.
use crate::{
    CrtRng, Result, cape_cold, ensure,
    world::{self, Decision, DrawKind, Loader, Trace, WorldState},
};
use ot2_battle_engine::mug;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Deserialize)]
struct Data {
    night_placement: String,
    before_events: [String; 2],
    before_weights: [f32; 2],
}
fn data() -> &'static Data {
    static DATA: OnceLock<Data> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../data/cape_cold_mug.json"))
            .expect("audited Mug field data")
    })
}
#[derive(Debug, Serialize)]
pub struct Prediction {
    pub before: CrtRng,
    pub after_night: CrtRng,
    pub before_battle: CrtRng,
    pub mug: mug::Prediction,
    pub after: CrtRng,
    pub world_after: WorldState,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub verification: &'static str,
}
pub fn after_cape_cold(p: &cape_cold::Prediction) -> Result<Option<Prediction>> {
    let (Some(before), Some(state)) = (p.after_loading, &p.world_after) else {
        return Ok(None);
    };
    predict(before, state.clone()).map(Some)
}

pub fn predict(before: CrtRng, state: WorldState) -> Result<Prediction> {
    predict_with_boost_inputs(before, state, None)
}

pub fn predict_with_boost_inputs(
    before: CrtRng, state: WorldState, round_two: Option<&[u8]>,
) -> Result<Prediction> {
    ensure(
        state.map == cape_cold::MAP
            && state.time == 2
            && state.party == [7, 6]
            && state.flag(7020)
            && state.flag(1370),
        "Cape Cold Mug requires daytime arrival after skipped-story Osvald recruitment",
    )?;
    let mut loader = Loader::new(world::catalog(), state, before)?;
    loader.set_time(4, "Cape Cold: switch to night before Mug")?;
    loader.refresh_npcs("Cape Cold: nighttime NPC placements")?;
    let after_night = loader.rng;
    ensure(
        loader.state.existing_npcs.contains(&data().night_placement),
        "Mug target is not present",
    )?;
    ensure(
        loader.state.resident_levels.contains("Btl_Twn_Snw_D"),
        "Cape Cold Mug battle sublevel must already be resident",
    )?;
    let menu = loader.rng;
    loader.rng.next_u15();
    loader.trace.push(Trace {
        reason: "Open Mug preview: field-action timer".into(),
        kind: DrawKind::PathActionMenu,
        subject: data().night_placement.clone(),
        value: None,
        before: menu,
        after: loader.rng,
    });
    let dialogue = loader.rng;
    let weights = data().before_weights;
    let roll = loader.rng.unit() * (weights[0] + weights[1]);
    let selected = usize::from(roll > weights[0]);
    loader.trace.push(Trace {
        reason: "Mug: select opening line".into(),
        kind: DrawKind::DialogueLottery,
        subject: data().before_events[selected].clone(),
        value: Some(selected as u32),
        before: dialogue,
        after: loader.rng,
    });
    // Audited fixed-line events contain only flag branches/talk/end, with
    // EncountReset=false. No NPC event override and no battle-map construction:
    // Cape Cold's battle sublevel is already resident from arrival.
    let before_battle = loader.rng;
    let mug = mug::predict_with_boost_inputs(before_battle, round_two)?;
    loader.rng = mug.after;
    loader.restore_field_after_battle("Cape Cold: return from Mug")?;
    // FC_GAK_ROB_Success also has EncountReset=false; getting the three
    // valuables is deterministic. It does not add a second counter reset.
    Ok(Prediction {
        before,
        after_night,
        before_battle,
        mug,
        after: loader.rng,
        world_after: loader.state,
        trace: loader.trace,
        decisions: loader.decisions,
        verification: "Seed0+6154 through first menu6192 and battle completion/field return6299 verified against retail CRT captures. First Mug of Cape Cold's man holding three valuables; original-gear level-one Osvald; English at 2x; Icewind, 2 BP Fireball, unboosted staff if needed. Entry is daytime Cape Cold; exit is after loot/success dialogue and field restoration. Uses shared nighttime NPC evaluation and field return callbacks, not a fixed RNG offset.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn night_refresh_menu_and_mug_dialogue_have_distinct_draws() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../data/source/capecold-mug-entry-6154.json"))
                .unwrap();
        let before = serde_json::from_value(fixture["before"].clone()).unwrap();
        let state = serde_json::from_value(fixture["world"].clone()).unwrap();
        let p = predict(before, state).unwrap();
        let live: serde_json::Value = serde_json::from_str(include_str!(
            "../data/source/cape-cold-mug-first-menu-live.json"
        )).unwrap();
        let calls = live["calls"].as_array().unwrap();
        let mut spans: Vec<_> = p.trace.iter()
            .filter(|s| s.before.draws < 6165)
            .map(|s| (s.before, s.after))
            .chain(p.mug.trace.iter().filter(|s| s.before.draws < 6192)
                .map(|s| (s.before, s.after)))
            .collect();
        spans.sort_by_key(|(before, _)| before.draws);
        let mut replay = p.before;
        for (before, after) in spans {
            assert_eq!(before, replay);
            while replay.draws < after.draws {
                let call = &calls[(replay.draws - 6154) as usize];
                assert_eq!(call["position"], replay.draws);
                assert_eq!(call["before"], replay.state);
                replay.next_u15();
            }
            assert_eq!(after, replay);
        }
        assert_eq!(replay, p.mug.menus[0].rng);
        assert_eq!(replay.draws, 6192);
        assert_eq!(p.after_night.draws, 6163);
        assert_eq!(
            p.trace
                .iter()
                .filter(|s| s.kind == DrawKind::NpcFrame)
                .count(),
            8
        );
        assert_eq!(
            p.trace
                .iter()
                .filter(|s| s.kind == DrawKind::FlagLottery)
                .count(),
            1
        );
        assert_eq!(p.before_battle.draws, 6165);
        assert_eq!(
            p.trace
                .iter()
                .filter(|s| s.kind == DrawKind::PathActionMenu)
                .count(),
            1
        );
        assert_eq!(
            p.trace
                .iter()
                .filter(|s| s.kind == DrawKind::DialogueLottery)
                .count(),
            1
        );
        assert_eq!(p.mug.after.draws, 6297);
        let completion: serde_json::Value = serde_json::from_str(include_str!(
            "../data/source/cape-cold-mug-completion-live.json"
        )).unwrap();
        let mut field_rng = p.mug.after;
        for call in completion["calls"].as_array().unwrap().iter()
            .filter(|c| c["position"].as_u64().unwrap() >= 6297) {
            assert!(call["node"].as_str().unwrap().ends_with("CalcEncountStepCount"));
            assert_eq!(call["before"], field_rng.state);
            field_rng.next_u15();
        }
        assert_eq!(field_rng, p.after);
        assert_eq!(
            p.after,
            CrtRng {
                state: 0xdc86178d,
                draws: 6299
            }
        );
        assert_eq!(
            p.trace
                .iter()
                .filter(|s| s.kind == DrawKind::Counter)
                .count(),
            2
        );
    }
}
