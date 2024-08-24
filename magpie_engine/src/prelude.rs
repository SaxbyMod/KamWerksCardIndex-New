//! Import commonly use types and traits.
//!
//! Re-export types that you can use by just importing it.
//! # Example
//!
//! Import the prelude with:
//! ```
//! use magpie_engine::prelude::*;
//! ```

pub use crate::{
    fetch::{
        fetch_aug_set, fetch_cti_set, fetch_desc_set, fetch_imf_set, AugCosts, AugExt, DescCosts,
        SetError,
    },
    query::{FilterFn, Filters, QueryBuilder, QueryOrder, ToFilter},
    *,
};
