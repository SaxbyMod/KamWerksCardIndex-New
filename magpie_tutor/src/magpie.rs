//! Some code, implementation and extension for magpie

use magpie_engine::{
    bitsflag,
    fetch::AugCost,
    prelude::AugExt,
    query::{FilterFn, ToFilter},
};

use crate::lev;

bitsflag! {
    /// Cost type value for filter
    pub struct CostType: u8 {
        /// Blood cost
        BLOOD = 1;
        /// Bone cost
        BONE = 1 << 1;
        /// Energy Cost
        ENERGY = 1 << 2;
        /// Mox cost
        MOX = 1 << 3;
    }
}

/// Extra Filter for query
#[derive(Clone, PartialEq, Eq)]
pub enum FilterExt {
    /// Fuzzy match the card name
    Fuzzy(String),
    /// Fuzzy match the card name
    CostType(CostType),
}

impl ToFilter<AugExt, AugCost> for FilterExt {
    fn to_fn(self) -> FilterFn<AugExt, AugCost> {
        match self {
            FilterExt::Fuzzy(str) => {
                Box::new(move |c| lev(&c.name, &str, 0.5) != 0. || c.name.contains(&str))
            }
            FilterExt::CostType(t) => Box::new(move |c| {
                if let Some(c) = &c.costs {
                    !(t.contains(CostType::BLOOD) && c.blood == 0
                        || t.contains(CostType::BONE) && c.bone == 0
                        || t.contains(CostType::ENERGY) && c.energy == 0
                        || t.contains(CostType::MOX) && c.mox == 0)
                } else {
                    false
                }
            }),
        }
    }
}
