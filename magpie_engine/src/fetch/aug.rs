//! Implementation for the [Augmented] set
//!
//! [Augmented]: https://steamcommunity.com/sharedfiles/filedetails/?id=2966485639&searchtext=augmented

use super::{fetch_json, FetchError};
use crate::{Card, Costs, Mox, MoxCount, Set, SetCode, Temple, Traits};
use crate::{Ptr, Rarity};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Not;

/// Augmented's [`Card`] extensions
#[derive(Debug, Default)]
pub struct AugExt {
    /// Shattered mox cost count.
    pub shattered_count: Option<MoxCount>,
    /// Max energy cell cost.
    pub max: isize,
    /// The card tribes.
    pub tribes: String,
    /// Artist credit.
    pub artist: String,
}

/// Fetch Augmented from the
/// [sheet](https://docs.google.com/spreadsheets/d/1tvTXSsFDK5xAVALQPdDPJOitBufJE6UB_MN4q5nbLXk/edit?gid=0#gid=0).
/// # Panics
#[allow(clippy::too_many_lines)]
pub fn fetch_aug_set(code: SetCode) -> Result<Set<AugExt>, AugError> {
    let raw_card: Vec<AugCard> =
        fetch_json("https://opensheet.elk.sh/1tvTXSsFDK5xAVALQPdDPJOitBufJE6UB_MN4q5nbLXk/Cards")
            .map_err(AugError::CardFetchError)?;

    let sigil: Vec<AugSigil> =
        fetch_json("https://opensheet.elk.sh/1tvTXSsFDK5xAVALQPdDPJOitBufJE6UB_MN4q5nbLXk/Sigils")
            .map_err(AugError::SigilFetchError)?;

    let name = Ptr::new("Augmented".to_owned());

    let mut cards = Vec::with_capacity(raw_card.len());

    let undefined_sigil = Ptr::new("UNDEFINDED SIGILS".to_string());

    let mut sigil_rc = HashMap::with_capacity(sigil.len());
    let mut sigils_description = HashMap::with_capacity(sigil.len());

    for s in sigil {
        let rc = Ptr::new(s.name.clone());
        sigil_rc.insert(s.name, rc.clone());
        sigils_description.insert(rc.clone(), s.text);
    }

    for card in raw_card {
        let traits = card
            .traits
            .split(", ")
            .map(ToOwned::to_owned)
            .collect::<Vec<String>>();

        let costs;

        let mut shattered_count = MoxCount::default();
        let mut mox_count = MoxCount::default();
        let mut max = 0;

        if card.cost != "free" && !card.cost.is_empty() {
            let mut t = Costs::default();

            for c in card
                .cost
                .replace("bones", "bone")
                .replace("rubies", "ruby")
                .replace("emeralds", "emerald")
                .replace("sapphires", "sapphire")
                .replace("prisms", "prism")
                .split('+')
            {
                let (count, mut cost): (isize, Vec<String>) = {
                    let s = c.to_lowercase().trim().to_string();
                    let mut t = s.split_whitespace().map(ToOwned::to_owned);

                    let first = t
                        .next()
                        .ok_or_else(|| AugError::InvalidCostFormat(card.cost.clone()))?
                        .parse::<isize>()
                        .map_err(|_| AugError::InvalidCostFormat(card.cost.clone()))?;
                    let mut rest = t.collect::<Vec<String>>();

                    rest.reverse();
                    (first, rest)
                };

                match cost
                    .pop()
                    .ok_or_else(|| AugError::InvalidCostFormat(card.cost.clone()))?
                    .as_str()
                {
                    "blood" => t.blood += count,
                    "bone" => t.bone += count,
                    "energy" => t.energy += count,
                    "max" => max += count,
                    "shattered" => match cost.pop().unwrap().as_str() {
                        "ruby" => {
                            t.mox |= Mox::R;
                            shattered_count.r += count as usize;
                        }
                        "emerald" => {
                            t.mox |= Mox::G;
                            shattered_count.g += count as usize;
                        }
                        "sapphire" => {
                            t.mox |= Mox::B;
                            shattered_count.b += count as usize;
                        }
                        "prism" => {
                            t.mox |= Mox::Y;
                            shattered_count.y += count as usize;
                        }
                        m => return Err(AugError::UnknowMox(m.to_owned())),
                    },
                    m @ ("ruby" | "sapphire" | "emerald" | "prism") => match m {
                        "ruby" => {
                            t.mox |= Mox::R;
                            mox_count.r += count as usize;
                        }
                        "emerald" => {
                            t.mox |= Mox::G;
                            mox_count.g += count as usize;
                        }
                        "sapphire" => {
                            t.mox |= Mox::B;
                            mox_count.b += count as usize;
                        }
                        "prism" => {
                            t.mox |= Mox::Y;
                            mox_count.y += count as usize;
                        }
                        _ => unreachable!(),
                    },
                    "asterisk" => (),
                    c => return Err(AugError::UnknowCost(c.to_string())),
                }
            }
            if mox_count != MoxCount::default() {
                t.mox_count = Some(mox_count);
            }
            costs = Some(t);
        } else {
            costs = None;
        }

        let card = Card {
            portrait: format!("https://github.com/answearingmachine/card-printer/raw/main/dist/printer/assets/art/{}.png", card.name.replace(' ', "%20")),

            set: code,

            name: card.name,
            description: card.description,
            rarity: match card.rarity.as_str() {
                "Common" | "" => Rarity::COMMON,
                "Uncommon" => Rarity::UNCOMMON,
                "Rare" => Rarity::RARE,
                "Talking" => Rarity::UNIQUE,
                "Side Deck" => Rarity::SIDE,
                _ => return Err(AugError::UnknownRarity(card.rarity)),
            },
            temple:match card.temple.as_str() {
                "Beast" => Temple::BEAST,
                "Undead" => Temple::UNDEAD,
                "Tech" => Temple::TECH,
                "Magick" => Temple::MAGICK,
                "Fool" => Temple::FOOL,
                _ => return Err(AugError::UnknownTemple(card.temple))
            }.into(),
            attack: card.attack.parse().unwrap_or(0),
            health: card.health.parse().unwrap_or(0),
            sigils: card.sigils.split(", ").map(|s| sigil_rc.get(s).unwrap_or(&undefined_sigil).clone()).collect(),
            // I don't pay enough attention to augmented to keep updating the code to accommodate
            // them so the value will just be parse as string
            sp_atk: None,
            costs,
            traits: traits.is_empty().then_some(Traits {
                traits: Some(traits),
                flags: 0
            }),
            related: card.token.is_empty().not().then(||card.token.split(", ").map(ToOwned::to_owned).collect()),
            extra: AugExt {
                artist: card.artist,
                max,
                shattered_count: if shattered_count.eq(&MoxCount::default()).not() { Some(shattered_count) } else { None },
                tribes: card.tribes
            }
        };

        cards.push(card);
    }

    Ok(Set {
        code,
        name,
        cards,
        sigils_description,
    })
}

/// Error that happen when calling [`fetch_aug`].
#[derive(Debug)]
pub enum AugError {
    /// Error when trying to [`fetch_json`] cards.
    CardFetchError(FetchError),
    /// Error when trying to [`fetch_json`] sigils.
    SigilFetchError(FetchError),
    /// Invalid Rarity.
    UnknownRarity(String),
    /// Invalid Temple.
    UnknownTemple(String),
    /// Invalid cost format. The cost doesn't follow each component are a number then the cost
    /// with space between and every cost is separted by `'+'`.
    InvalidCostFormat(String),
    /// Unknow cost.
    UnknowCost(String),
    /// Invalid Mox color.
    UnknowMox(String),
}

impl Display for AugError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AugError::CardFetchError(e) => write!(f, "cannot fetch cards due to: {e}"),
            AugError::SigilFetchError(e) => write!(f, "cannot fetch sigils due to: {e}"),
            AugError::UnknownRarity(r) => write!(f, "unknown rarity: {r}"),
            AugError::UnknownTemple(r) => write!(f, "unknown temple: {r}"),
            AugError::InvalidCostFormat(s) => write!(f, "invalid cost: {s}"),
            AugError::UnknowCost(c) => write!(f, "unknow cost: {c}"),
            AugError::UnknowMox(m) => write!(f, "unknow mox: {m}"),
        }
    }
}

/// Json scheme for aug card
#[derive(Deserialize)]
struct AugCard {
    #[serde(rename = "Card Name")]
    name: String,
    #[serde(rename = "Flavor Text")]
    description: String,

    #[serde(rename = "Temple")]
    temple: String,
    #[serde(rename = "Tier")]
    rarity: String,

    #[serde(rename = "Cost")]
    cost: String,

    #[serde(rename = "🗡")]
    attack: String,
    #[serde(rename = "♥")]
    health: String,

    #[serde(rename = "Sigils")]
    sigils: String,

    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "Traits")]
    traits: String,
    #[serde(rename = "Tribes")]
    tribes: String,

    #[serde(rename = "Credit")]
    artist: String,
}

/// Json scheme for aug sigil
#[derive(Deserialize)]
struct AugSigil {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Text")]
    text: String,
}
