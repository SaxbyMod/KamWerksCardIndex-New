//! Import commonly use types and traits
//!
//! Re-export types that you can use by just importing it
//! # Example
//!
//! Import the prelude with:
//! ```
//! use magpie_engine::prelude::*
//! ```

pub use crate::{
    fetch::{aug::*, imf::*},
    query::{Filter, Filters, QueryBuilder},
    *,
};
