//! Import commonly use types and traits
//!
//! Re-export types that you can use by just importing it
//! # Example
//!
//! Import the prelude with:
//! ```
//! use magpie_engine::prelude::*
//! ```

pub use crate::data::{Card, Set, SetCode, SpAtk, Temple, Traits};
pub use crate::fetch::{imf::fetch_imf_set, FetchError};
pub use crate::query::{Filter, Filters, QueryBuilder};
