//! Some code, implementation and extension for the engine

use bitflags::bitflags;
use magpie_engine::prelude::*;

use crate::lev;

bitflags! {
    /// Cost type value for filter
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CostType: u8 {
        /// Blood cost
        const BLOOD = 1;
        /// Bone cost
        const BONE = 1 << 1;
        /// Energy Cost
        const ENERGY = 1 << 2;
        /// Mox cost
        const MOX = 1 << 3;
    }
}

/// Extra Filter for query
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterExt {
    /// Fuzzy match the card name
    Fuzzy(String),
    /// Fuzzy match the card name
    CostType(CostType),
}

impl ToFilter<MagpieExt, MagpieCosts> for FilterExt {
    fn to_fn(self) -> FilterFn<MagpieExt, MagpieCosts> {
        match self {
            FilterExt::Fuzzy(str) => {
                Box::new(move |c| lev(&c.name, &str, 0.5) != 0. || c.name.contains(&str))
            }
            FilterExt::CostType(t) => Box::new(move |c| {
                if let Some(c) = &c.costs {
                    !(t.contains(CostType::BLOOD) && c.blood == 0
                        || t.contains(CostType::BONE) && c.bone == 0
                        || t.contains(CostType::ENERGY) && c.energy == 0
                        || t.contains(CostType::MOX) && c.mox.is_empty())
                } else {
                    false
                }
            }),
        }
    }
}

/// Magpie's [`Card`] Extension to unify all the extension
#[derive(Debug, Clone, Default)]
pub struct MagpieExt {
    /// Artist credit from [`AugExt`]
    pub artist: String,
}

/// Magpie's [`Costs`] extension to unify all cost
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MagpieCosts {
    /// Shattered mox count from [`AugCosts`]
    pub shattered_count: Option<MoxCount>,
    /// Mox energy from [`AugCosts`]
    pub max: isize,
    /// Links from [`DescCosts`]
    pub link: isize,
    /// Gold from [`DescCosts`]
    pub gold: isize,
}

// there no error here ra is just having a stroke
impl UpgradeCard<MagpieExt, MagpieCosts> for Card<AugExt, AugCosts> {
    fn upgrade(self) -> Card<MagpieExt, MagpieCosts> {
        upgrade_card! {
            extra: MagpieExt { artist: self.extra.artist },
            costs: |c: Costs<AugCosts>| MagpieCosts {
                shattered_count: c.extra.shattered_count,
                max: c.extra.max,
                link: 0,
                gold: 0,
            },
            ..self
        }
    }
}

impl UpgradeCard<MagpieExt, MagpieCosts> for Card<(), DescCosts> {
    fn upgrade(self) -> Card<MagpieExt, MagpieCosts> {
        upgrade_card! {
            extra: MagpieExt { artist: String::new() },
            costs: |c: Costs<DescCosts>| MagpieCosts {
                shattered_count: None,
                max: 0,
                link: c.extra.link,
                gold: c.extra.gold,
            },
            ..self
        }
    }
}
