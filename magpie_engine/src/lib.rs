//! Crate to fetch and query Inscryption cards.
//!
//! Currently only support [IMF] and [Augmented] set.
//!
//! A Set is a collection of cards and info related to them. Each set have a 3 characters set code
//! much like magic
//!
//! [IMF]: https://107zxz.itch.io/inscryption-multiplayer-godot
//! [Augmented]: https://steamcommunity.com/sharedfiles/filedetails/?id=2966485639&searchtext=augmented

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
