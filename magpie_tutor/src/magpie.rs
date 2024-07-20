//! Some code, implementation and extension for magpie

use magpie_engine::query::{Filter, FilterFn};

use crate::fuzzy::lev;

/// Extra Filter for query
#[derive(Clone, PartialEq, Eq)]
pub enum FilterExt {
    /// Fuzzy match the card name
    Fuzzy(String),
}

impl<C> Filter<C> for FilterExt {
    fn to_fn(self) -> FilterFn<C> {
        match self {
            FilterExt::Fuzzy(str) => {
                Box::new(move |c| lev(&c.name, &str, 0.5) != 0. || c.name.contains(&str))
            }
        }
    }
}
