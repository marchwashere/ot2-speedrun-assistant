//! Fixed route from the Steal tutorial to control inside Diamante's Estate.
//! Preserve the world between steals, map loads and skipped story events.
use crate::{
    CrtRng, Result, ensure,
    world::{self, Decision, DrawKind, Loader, Trace, WorldState},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub arrival: CrtRng,
    pub after_clothes: CrtRng,
    pub after_boy: CrtRng,
    pub after_main_load: CrtRng,
    pub after_artist: CrtRng,
    pub after_entry: CrtRng,
    pub lookout_ambush: LookoutAmbush,
    pub acquired_items: std::collections::BTreeMap<String, u32>,
    pub expected_draws: u64,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub world_after: WorldState,
    pub verification: &'static str,
}

#[derive(Debug, Serialize)]
pub struct LookoutAmbush {
    pub before_menu: CrtRng,
    pub at_menu: CrtRng,
    pub after: CrtRng,
    pub expected_draws: u64,
    pub counter_reset: bool,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub world_after: WorldState,
    pub verification: &'static str,
}

/// One menu opening and the lookout's authored Ambush, ending before the
/// separate 1800 cutscene trigger. Supply the remaining counter at the menu;
/// this operation never consumes walking distance or resets that counter.
pub fn ambush_lookout(before_menu: CrtRng, state: WorldState) -> Result<LookoutAmbush> {
    ensure(
        state.map == "Dng_Cty_1_1"
            && state.time == 4
            && state.flag(8015)
            && !state.flag(8016)
            && state.existing_npcs.contains("NPC_Dng_Cty_1_1_0100_N000"),
        "Lookout Ambush requires the awake mansion lookout after 1600 and before 1700",
    )?;
    let mut loader = Loader::new(world::catalog(), state, before_menu)?;
    loader.draw(
        DrawKind::PathActionMenu,
        "Mansion lookout",
        "Mansion lookout: open Ambush menu (action timer)",
        None,
    );
    let at_menu = loader.rng;
    // The type-20 story override selects fixed dialogue and bypasses the
    // ordinary Ambush dialogue lottery. Ambush itself is a level check.
    // 8016 replaces N000 with the authored sleeping placement N010. Let the
    // normal placement evaluator consume its frame roll; do not add an offset.
    loader.skipped_event("MS_TOU_10_1700")?;
    loader.boundary("Mansion lookout ambushed; control returned before the next cutscene");
    Ok(LookoutAmbush {
        before_menu,
        at_menu,
        after: loader.rng,
        expected_draws: loader.rng.draws - before_menu.draws,
        counter_reset: loader.data.events["MS_TOU_10_1700"].reset_counter,
        trace: loader.trace,
        decisions: loader.decisions,
        world_after: loader.state,
        verification: "Source-derived, awaiting live validation: one Ambush menu opening, fixed 1700 dialogue, then sleeping-lookout placement initialization. No encounter-counter reset or automatic 1800 cutscene. The route assumes the short approach is encounter-free; its unmeasured walking cost is not deducted from the carried counter.",
    })
}

/// These route items have guaranteed Steal success at the route's level.
/// One menu visit retains its item list; generic dialogue is chosen once on
/// closing it. The clothes trigger 1400 instead. See the three Steal audits.
fn steal_session(loader: &mut Loader<'_>, npc: &str, items: &[&str], generic: bool) {
    loader.draw(
        DrawKind::PathActionMenu,
        npc,
        format!("{npc}: open Steal menu (action timer)"),
        None,
    );
    for item in items {
        let roll = loader.draw(
            DrawKind::StealCheck,
            item,
            format!("{npc}: steal {item} (guaranteed success check)"),
            Some(100),
        );
        // Kismet RandomIntegerInRange(1, 100) adds the lower bound.
        loader.trace.last_mut().unwrap().value = Some(roll + 1);
    }
    if generic {
        loader.draw(
            DrawKind::DialogueLottery,
            npc,
            format!("{npc}: close menu, choose success dialogue"),
            None,
        );
    }
}

pub fn after_parlor_revisit(previous: &crate::parlor_revisit::Prediction) -> Result<Prediction> {
    with_world_state(previous.after_popup, previous.world_after.clone())
}

pub fn with_world_state(arrival: CrtRng, state: WorldState) -> Result<Prediction> {
    ensure(
        state.map == "Twn_Cty_1_1_B" && state.flag(8012) && !state.flag(8013),
        "Mansion route starts after the Steal tutorial, before stealing the clothes",
    )?;
    ensure(
        state.time == 3 && state.party == [7],
        "Mansion route requires Throné alone in the scripted Dusk phase",
    )?;
    let mut loader = Loader::new(world::catalog(), state, arrival)?;
    steal_session(&mut loader, "Brothel girl", &["clothes"], false);
    loader.skipped_event("MS_TOU_10_1400")?;
    // 1400 FinishEvent invokes implicit FinishPre. No counter reset; evaluate
    // its flag-driven placement refresh rather than assuming no NPC changes.
    ensure(
        !loader.data.events["MS_TOU_10_1400"].reset_counter,
        "Re-audit clothes cleanup if EncountReset changes",
    )?;
    loader.refresh_bar()?;
    loader.refresh_npcs("MS_TOU_10_1400 implicit FinishPre")?;
    loader.boundary("Clothes stolen; Save popup dismissed");
    let after_clothes = loader.rng;
    steal_session(&mut loader, "Boy", &["Shadow Soulstone"], true);
    loader.boundary("Boy: Shadow Soulstone stolen; menu closed");
    let after_boy = loader.rng;
    loader.load_map("Twn_Cty_1_1_A")?;
    loader.boundary("New Delsta main area loaded");
    let after_main_load = loader.rng;
    steal_session(
        &mut loader,
        "Artist",
        &["Ice Soulstone", "Wind Soulstone", "Light Soulstone"],
        true,
    );
    loader.boundary("Artist: all three soulstones stolen; menu closed");
    let after_artist = loader.rng;
    loader.skipped_event("MS_TOU_10_1500")?;
    loader.boundary("Mansion entry: both cutscenes skipped; control returned");
    let lookout_ambush = ambush_lookout(loader.rng, loader.state.clone())?;
    Ok(Prediction {
        arrival,
        after_clothes,
        after_boy,
        after_main_load,
        after_artist,
        after_entry: loader.rng,
        lookout_ambush,
        acquired_items: [
            "shadow_soulstone",
            "ice_soulstone",
            "wind_soulstone",
            "light_soulstone",
        ]
        .into_iter()
        .map(|id| (id.into(), 1))
        .collect(),
        expected_draws: loader.rng.draws - arrival.draws,
        trace: loader.trace,
        decisions: loader.decisions,
        world_after: loader.state,
        verification: "Mansion endpoint confirmed in-game on 2026-09-28: seed 0 at 5562 / 0xDDCC3032. Continues through both parlor visits, clothes theft and Save popup, Boy's Shadow Soulstone, New Delsta main load, Artist's three soulstones in one menu visit, and skipped 1500/1600. Map loads evaluate carried world state. No intervening actions or automatic time changes; individual mansion calls are source-derived, not a per-call live capture.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn previous() -> crate::parlor_revisit::Prediction {
        let city = crate::city::after_second_victory(CrtRng {
            state: 0x41c7db10,
            draws: 5264,
        })
        .unwrap();
        let parlor = crate::parlor::after_city(&city).unwrap();
        crate::parlor_revisit::after_first_visit(&parlor).unwrap()
    }

    #[test]
    fn confirmed_endpoint_preserves_intermediate_progression_and_trace_continuity() {
        let previous = previous();
        let p = after_parlor_revisit(&previous).unwrap();
        assert_eq!(p.arrival, previous.after_popup);
        for (actual, draws, state) in [
            (p.after_clothes, 5469, 0xbeb9762f),
            (p.after_boy, 5472, 0x956ef860),
            (p.after_main_load, 5530, 0xb15bb212),
            (p.after_artist, 5535, 0x94b3ad61),
            (p.after_entry, 5562, 0xddcc3032),
        ] {
            assert_eq!(actual, CrtRng { draws, state });
        }
        let mut rng = p.arrival;
        for span in &p.trace {
            assert_eq!(span.before, rng);
            rng.advance(span.after.draws - span.before.draws);
            assert_eq!(span.after, rng);
        }
        assert_eq!(rng, p.after_entry);
        assert_eq!(p.world_after.map, "Dng_Cty_1_1");
        assert_eq!(p.world_after.time, 4);
        for flag in [8013, 8014, 8015, 1015, 1315, 25305] {
            assert!(p.world_after.flag(flag));
        }
        assert!(!p.world_after.flag(34049));
        assert!(!p.world_after.flag(652));
    }

    #[test]
    fn carries_relative_draw_counts_and_world_changes_instead_of_fixed_offsets() {
        let mut previous = previous();
        previous.after_popup.draws = 0;
        let baseline = after_parlor_revisit(&previous).unwrap();
        assert_eq!(baseline.after_entry.draws, 95);
        assert_eq!(baseline.after_entry.state, 0xddcc3032);
        previous.world_after.resident_levels.remove("Btl_SinExpose");
        let recreated = after_parlor_revisit(&previous).unwrap();
        assert_eq!(recreated.after_entry.draws, 96);
        assert_ne!(recreated.after_entry.state, baseline.after_entry.state);
        assert!(with_world_state(baseline.after_entry, baseline.world_after).is_err());
    }

    #[test]
    fn lookout_ambush_consumes_menu_and_replacement_draws_without_resetting_counter() {
        let p = after_parlor_revisit(&previous()).unwrap();
        let mut state = p.world_after.clone();
        state.encounter_counter = 137; // Remaining counter after approaching him.
        let a = ambush_lookout(p.after_entry, state).unwrap();
        assert_eq!(a.before_menu, p.after_entry);
        assert_eq!(
            a.at_menu,
            CrtRng {
                state: 0x3425562d,
                draws: 5563
            }
        );
        assert_eq!(
            a.after,
            CrtRng {
                state: 0x5121903c,
                draws: 5564
            }
        );
        assert!(!a.counter_reset);
        assert_eq!(a.world_after.encounter_counter, 137);
        assert_eq!(a.world_after.first_encounter, p.world_after.first_encounter);
        assert!(a.world_after.flag(8016));
        assert!(!a.world_after.flag(25305));
        assert!(!a.world_after.flag(8017)); // Next cutscene has not run.
        assert!(
            !a.world_after
                .existing_npcs
                .contains("NPC_Dng_Cty_1_1_0100_N000")
        );
        assert!(
            a.world_after
                .existing_npcs
                .contains("NPC_Dng_Cty_1_1_0100_N010")
        );
        let calls: Vec<_> = a.trace.iter().filter(|s| s.before != s.after).collect();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].kind, DrawKind::PathActionMenu);
        assert_eq!(calls[1].kind, DrawKind::NpcFrame);
        assert_eq!(calls[1].subject, "NPC_Dng_Cty_1_1_0100_N010");
        assert!(a.trace.windows(2).all(|w| w[0].after == w[1].before));
        assert!(ambush_lookout(a.after, a.world_after).is_err());
    }
}
