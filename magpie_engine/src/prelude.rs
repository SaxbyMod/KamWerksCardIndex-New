//! Import commonly use types and traits
//!
//! Re-export types that you can use by just importing it
//! # Example
//!
//! Import the prelude with:
//! ```
//! use magpie_engine::prelude::*
//! ```
pub use crate::cards::{Card, Set, SetCode};
pub use crate::fetch::{aug::fetch_aug, imf::fetch_imf_set, FetchError};
pub use crate::query::{Filter, Filters, QueryBuilder};
