//! Implementation for the [Augmented] set.
//!
//! [Augmented]: https://steamcommunity.com/sharedfiles/filedetails/?id=2966485639&searchtext=augmented

use std::{collections::HashMap, fmt::Display};

use serde::Deserialize;

use crate::{
    fetch::{fetch_json, FetchError},
    self_upgrade, Attack, Card, Costs, Mox, MoxCount, Rarity, Set, SetCode, Temple, Traits,
    TraitsFlag,
};

/// Augmented's [`Card`] extensions.
#[derive(Debug, Default, Clone)]
pub struct AugExt {
    /// Artist credit.
    pub artist: String,
}

/// Augmented's [`Costs`] extensions.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AugCosts {
    /// Shattered mox cost count.
    pub shattered_count: Option<MoxCount>,
    /// Max energy cell cost.
    pub max: isize,
}

self_upgrade!(AugExt, AugCosts);

/// Fetch Augmented from the
/// [sheet](https://docs.google.com/spreadsheets/d/1tvTXSsFDK5xAVALQPdDPJOitBufJE6UB_MN4q5nbLXk).
#[allow(clippy::too_many_lines)]
pub fn fetch_aug_set(code: SetCode) -> Result<Set<AugExt, AugCosts>, AugError> {
    let raw_card: Vec<AugCard> =
        fetch_json("https://opensheet.elk.sh/1tvTXSsFDK5xAVALQPdDPJOitBufJE6UB_MN4q5nbLXk/Cards")
            .map_err(AugError::CardFetchError)?;

    let sigil: Vec<AugSigil> =
        fetch_json("https://opensheet.elk.sh/1tvTXSsFDK5xAVALQPdDPJOitBufJE6UB_MN4q5nbLXk/Sigils")
            .map_err(AugError::SigilFetchError)?;

    let mut cards = Vec::with_capacity(raw_card.len());

    let mut sigils_description = HashMap::with_capacity(sigil.len());

    for s in sigil {
        sigils_description.insert(s.name, s.text.replace('\n', ""));
    }

    sigils_description.insert(
        String::from("UNDEFINDED SIGILS"),
        "THIS SIGIL IS NOT DEFINED BY THE SET".to_owned(),
    );

    for card in raw_card {
        let costs;

        let mut mox_count = MoxCount::default();
        let mut shattered_count = MoxCount::default();

        if card.cost != "free" && !card.cost.is_empty() {
            let mut t: Costs<AugCosts> = Costs::default();

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
                    "max" => t.extra.max += count,
                    "shattered" => match cost.pop().unwrap().as_str() {
                        "ruby" => {
                            t.mox |= Mox::O;
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
                            t.mox |= Mox::O;
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

            // only include the moxes if they are not the default all 1
            if mox_count != MoxCount::default() {
                t.mox_count = Some(mox_count);
            }

            if shattered_count != MoxCount::default() {
                t.extra.shattered_count = Some(shattered_count);
            }
            costs = Some(t);
        } else {
            costs = None;
        }

        let card = Card {
            portrait: format!("https://raw.githubusercontent.com/answearingmachine/card-printer/main/dist/printer/assets/art/{}.png", card.name.replace(' ', "%20")),

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
            },
            tribes: (!card.tribes.is_empty()).then_some(card.tribes),

            attack: Attack::Num(card.attack.parse().unwrap_or(0)),
            health: card.health.parse().unwrap_or(0),
            sigils: if card.sigils.is_empty() {
                vec![]
            } else {
                card.sigils.split(", ").map(|s| {
                    let s = s.to_owned();
                    if sigils_description.contains_key(&s) {
                        s
                    } else {
                        String::from("UNDEFINEDED SIGILS")
                    }
                }).collect()
            },

            costs,

            traits: (!card.traits.is_empty()).then_some(Traits {
                strings: Some(
                     card
                    .traits
                    .split(", ")
                    .map(ToOwned::to_owned)
                    .collect::<Vec<String>>()
                ),

                flags: TraitsFlag::empty()
            }),
            related: if card.token.is_empty() {
                vec![]
            } else {
                card.token.split(", ").map(ToOwned::to_owned).collect()
            },

            extra: AugExt {
                artist: card.artist,
            }
        };

        cards.push(card);
    }

    Ok(Set {
        code,
        name: String::from("Augmented"),
        cards,
        sigils_description,
    })
}

/// Error that happen when calling [`fetch_aug_set`].
#[derive(Debug)]
pub enum AugError {
    /// Error when trying to [`fetch_json`] cards.
    CardFetchError(FetchError),
    /// Error when trying to [`fetch_json`] sigils.
    SigilFetchError(FetchError),
    /// Unkown rarity.
    UnknownRarity(String),
    /// Unkown temple.
    UnknownTemple(String),
    /// Invalid cost format. The cost doesn't follow each component are a number then the cost
    /// with space between and every cost is separted by `'+'`.
    InvalidCostFormat(String),
    /// Unknow cost.
    UnknowCost(String),
    /// Unkown Mox color.
    UnknowMox(String),
}

impl Display for AugError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AugError::CardFetchError(e) => write!(f, "cannot fetch augmented cards due to: {e}"),
            AugError::SigilFetchError(e) => write!(f, "cannot fetch sigils due to: {e}"),
            AugError::UnknownRarity(r) => write!(f, "unknown augmented rarity: {r}"),
            AugError::UnknownTemple(r) => write!(f, "unknown augmented temple: {r}"),
            AugError::InvalidCostFormat(s) => write!(f, "invalid augmented cost format: {s}"),
            AugError::UnknowCost(c) => write!(f, "unknow augmented cost: {c}"),
            AugError::UnknowMox(m) => write!(f, "unknow augmented mox: {m}"),
        }
    }
}

/// Json scheme for aug card.
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

/// Json scheme for aug sigil.
#[derive(Deserialize)]
struct AugSigil {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Text")]
    text: String,
}
