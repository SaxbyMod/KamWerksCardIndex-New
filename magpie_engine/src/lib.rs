//! Crate to fetch and query Inscryption cards.
//!
//! Currently support:
//! - [IMF](https://107zxz.itch.io/inscryption-multiplayer-godot)
//! - [Augmented](https://steamcommunity.com/sharedfiles/filedetails/?id=2966485639&searchtext=augmented)
//! - [Descryption](https://docs.google.com/spreadsheets/d/1EjOtqUrjsMRl7wiVMN7tMuvAHvkw7snv1dNyFJIFbaE)
//! - [Custom TCG Inscryption](https://www.notion.so/inscryption-pvp-wiki/Custom-TCG-Inscryption-3f22fc55858d4cfab2061783b5120f87)
//!
//! A Set is a collection of cards and info related to them. Each set have a 3 characters set code
//! much like Magic the Gathering.

pub mod prelude;

mod helper;

pub mod fetch;
pub mod query;

pub use data::cards::*;
pub use data::sets::*;

mod data {
    pub mod cards;
    pub mod sets;
}
