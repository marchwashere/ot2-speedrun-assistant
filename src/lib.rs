//! Headless assistant logic. UI is confined to the separate ot2-gui binary.
pub mod profile;
pub mod black_market;
pub mod cape_cold;
pub mod cape_cold_mug;
pub mod city;
pub mod crackridge;
pub mod encounter;
pub mod early_cait;
pub mod snows_escape;
pub mod snows_rare;
pub mod eastern_escape;
pub mod winterbloom;
pub mod winter_voyage;
pub mod flats;
pub mod flats_escape;
pub mod harbor;
pub mod highroad;
pub mod inventor;
pub mod formation_search;
pub mod manipulation;
pub mod mansion;
pub mod mansion_escape;
pub mod opening;
pub mod osvald;
pub mod oresrush;
pub mod ryu;
pub mod hikari;
pub mod scholar;
pub mod canalbrine;
pub mod hikari_manipulation;
pub mod parlor;
pub mod parlor_revisit;
pub mod pirro;
pub mod pirro_aftermath;
pub mod route;
pub mod second;
pub mod second_manipulation;
pub mod village_return;
pub mod world;
pub(crate) mod state_key;
pub use ot2_battle_engine::{CrtRng, Result, SimError};
pub(crate) fn ensure(condition: bool, message: impl Into<String>) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(SimError(message.into()))
    }
}
pub mod escape_search;
pub mod conning;

pub mod gravell_setup;
pub mod galdera_coast;
pub mod galdera_oresrush;

pub mod voice_identification;
pub mod identified_route;
