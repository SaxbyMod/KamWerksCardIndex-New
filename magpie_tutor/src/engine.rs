//! Some code, implementation and extension for the engine

use std::fmt::Display;

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

impl Display for CostType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = vec![];

        let flags = [
            (CostType::BLOOD, "blood"),
            (CostType::BONE, "bone"),
            (CostType::ENERGY, "energy"),
            (CostType::MOX, "mox"),
        ];

        for (f, v) in flags {
            if self.contains(f) {
                out.push(v);
            }
        }

        write!(f, "{}", out.join(" and "))
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

impl Display for FilterExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterExt::Fuzzy(n) => write!(f, "name similar to {n}"),
            FilterExt::CostType(t) => write!(f, "cost includes {t}"),
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

impl Display for MagpieCosts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = vec![];

        if let Some(ref m) = self.shattered_count {
            if m.o != 0 {
                out.push(format!("{} shattered orange", m.o));
            }
            if m.g != 0 {
                out.push(format!("{} shattered green", m.g));
            }
            if m.b != 0 {
                out.push(format!("{} shattered blue", m.b));
            }
            if m.y != 0 {
                out.push(format!("{} shattered gray", m.y));
            }
            if m.k != 0 {
                out.push(format!("{} shattered black", m.k));
            }
        }

        if self.max != 0 {
            out.push(format!("{} max energy", self.max));
        }
        if self.link != 0 {
            out.push(format!("{} link", self.link));
        }
        if self.gold != 0 {
            out.push(format!("{} gold", self.gold));
        }

        write!(f, "{}", out.join(" and "))
    }
}

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