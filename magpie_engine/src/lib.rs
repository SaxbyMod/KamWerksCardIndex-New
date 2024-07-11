//! Crate to fetch and query Inscryption cards.
//!
//! Currently only support [IMF] and [Augmented] set.
//!
//! A Set is a collection of cards and info related to them. Each set have a 3 characters set code
//! much like magic
//!
//! [IMF]: https://107zxz.itch.io/inscryption-multiplayer-godot
//! [Augmented]: https://steamcommunity.com/sharedfiles/filedetails/?id=2966485639&searchtext=augmented

#[cfg(not(feature = "async"))]
use std::rc::Rc as Ptr;

#[cfg(feature = "async")]
use std::sync::Arc as Ptr;

pub mod prelude;

mod helper;

pub mod fetch;
pub mod query;

/// Contain data type for magpie
pub mod data {
    mod cards;
    mod sets;

    pub use cards::*;
    pub use sets::*;
}
