//! Walk into New Delsta Harbor to tag fast travel, then immediately return.
use crate::{
    CrtRng, Result, ensure, flats,
    world::{self, Decision, Loader, Trace, WorldState},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub arrival: CrtRng,
    pub at_harbor: CrtRng,
    pub after: CrtRng,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
    pub world_at_harbor: WorldState,
    pub world_after: WorldState,
    pub tagged_destination: &'static str,
    pub verification: &'static str,
}

/// No endpoint if even the measured full walk would trigger an encounter.
pub fn after_flats(flats: &flats::Prediction) -> Result<Option<Prediction>> {
    if !flats.harbor_reach.walking.reaches_reset {
        return Ok(None);
    }
    with_world_state(flats.after, flats.world_after.clone()).map(Some)
}

pub fn with_world_state(arrival: CrtRng, state: WorldState) -> Result<Prediction> {
    ensure(
        state.map == "Fld_Cty_1_2" && state.time == 2 && state.first_encounter,
        "Harbor tag requires daytime Flats entry before its first encounter",
    )?;
    ensure(
        flats::harbor_reach(state.encounter_counter)?
            .walking
            .reaches_reset,
        "Harbor is not reachable without an encounter under the measured walk",
    )?;
    // Safe travel changes no CRT state. Its residual counter is overwritten by
    // RequestLoadFieldLevel's outgoing reset before any dependent load callback.
    // Do not assume equal walking/running update counts or add a movement roll.
    let mut loader = Loader::new(world::catalog(), state, arrival)?;
    loader.visit_map("Fld_Cty_1_1")?;
    loader.boundary("New Delsta Harbor: tagged, field control restored");
    let at_harbor = loader.rng;
    let world_at_harbor = loader.state.clone();
    loader.visit_map("Fld_Cty_1_2")?;
    loader.boundary("New Delsta Flats: returned after harbor tag");
    Ok(Prediction {
        arrival,
        at_harbor,
        after: loader.rng,
        trace: loader.trace,
        decisions: loader.decisions,
        world_at_harbor,
        world_after: loader.state,
        tagged_destination: "Fld_Cty_1_1",
        verification: "Source-derived, awaiting live verification. Reach harbor without an encounter, enter on foot and immediately return on foot; no ship, path action, menu or time change. Both map loads evaluate carried progression, NPC eligibility, streamed levels, environment callbacks and outgoing/incoming counter resets. SaveVisitedMap_Impl sets visit/icon bits with no CRT lottery. Harbor is LevelKind=field, so the first-town fast-travel tutorial is ineligible. Its WorldMap row permits fast travel without a temporary ban. Returned Flats gets a fresh first-encounter counter; no fixed transition offset.",
    })
}
