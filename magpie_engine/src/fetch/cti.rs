//! Implementation for the [Custom TCG Inscryption] set
//!
//! [Custom TCG Inscryption]: https://www.notion.so/inscryption-pvp-wiki/Custom-TCG-Inscryption-3f22fc55858d4cfab2061783b5120f87

use std::{collections::HashMap, fmt::Display};

use serde::Deserialize;

use crate::{
    fetch::{fetch_json, FetchError},
    Attack, Card, Costs, Mox, MoxCount, Rarity, Set, SetCode, Temple,
};

/// Fetch Custom TCG Inscryption from the
/// [sheet](https://docs.google.com/spreadsheets/d/152SuTx1fVc4zsqL4_zVDPx69sd9vYWikc2Ce9Y5vhJE/edit?gid=0#gid=0).
#[allow(clippy::too_many_lines)]
pub fn fetch_cti_set(code: SetCode) -> Result<Set<(), ()>, CtiError> {
    let raw_card: Vec<CtiCard> =
        fetch_json("https://opensheet.elk.sh/152SuTx1fVc4zsqL4_zVDPx69sd9vYWikc2Ce9Y5vhJE/1")
            .map_err(CtiError::CardFetchError)?;

    let sigil: Vec<CtiSigil> =
        fetch_json("https://opensheet.elk.sh/152SuTx1fVc4zsqL4_zVDPx69sd9vYWikc2Ce9Y5vhJE/2")
            .map_err(CtiError::SigilFetchError)?;

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
        if card.cost != "Free" && !card.cost.is_empty() {
            let mut t: Costs<()> = Costs::default();
            let mut mox_count = MoxCount::default();

            for c in card
                .cost
                .to_lowercase()
                .replace("bones", "bone")
                .split(", ")
            {
                let (count, cost) = {
                    let s = c.to_lowercase().trim().to_string();
                    let mut t = s.split_whitespace().map(ToOwned::to_owned);

                    let first = t
                        .next()
                        .ok_or_else(|| CtiError::InvalidCostFormat(card.cost.clone()))?
                        .parse::<isize>()
                        .map_err(|_| CtiError::InvalidCostFormat(card.cost.clone()))?;

                    (
                        first,
                        t.next()
                            .ok_or_else(|| CtiError::InvalidCostFormat(card.cost.clone()))?,
                    )
                };

                match cost.as_str() {
                    "blood" => t.blood += count,
                    "bone" => t.bone += count,
                    "energy" => t.energy += count,
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
                    c => return Err(CtiError::UnknowCost(c.to_string())),
                }
            }

            // only include the moxes if they are not the default all 1
            if mox_count != MoxCount::default() {
                t.mox_count = Some(mox_count);
            }

            costs = Some(t);
        } else {
            costs = None;
        }

        cards.push(Card {
            portrait: format!("https://raw.githubusercontent.com/SaxbyMod/NotionAssets/main/Formats/Custom%20TCG%20Inscryption/Portraits/{}.png", card.name.replace(' ', "%20")),

            set: code,

            name: card.name,
            description: card.description,

            rarity: match card.rarity.as_str() {
                "Common" | "Common (Joke Card)" | "" => Rarity::COMMON,
                "Uncommon" => Rarity::UNCOMMON,
                "Rare" => Rarity::RARE,
                "Talking" | "Deathcard" => Rarity::UNIQUE,
                "Side-Deck" => Rarity::SIDE,
                _ => return Err(CtiError::UnknownRarity(card.rarity)),
            },
            temple:match card.temple.as_str() {
                "Beast" => Temple::BEAST,
                "Undead" => Temple::UNDEAD,
                "Tech" => Temple::TECH,
                "Magicks" => Temple::MAGICK,
                "Terrain/Extras" => Temple::empty(),
                _ => return Err(CtiError::UnknownTemple(card.temple))
            },
            tribes: None,

            attack: Attack::Num(card.attack.parse().unwrap_or(0)),
            health: card.health.parse().unwrap_or(0),

            sigils: [card.sigil_1, card.sigil_2, card.sigil_3, card.sigil_4]
                .into_iter()
                .filter(|s| !s.is_empty())
                .map(
                    |s|
                    if sigils_description.contains_key(&s) { s }
                    else { String::from("UNDEFINED SIGIL") }
                )
                .collect(),

            costs,

            traits: None,
            related: if card.token.is_empty() {
                vec![]
            } else {
                card.token.split(", ").map(ToOwned::to_owned).collect()
            },

            extra: ()
        });
    }

    Ok(Set {
        code,
        name: String::from("Custom TCG Inscryption"),
        cards,
        sigils_description,
    })
}

/// Error that happen when calling [`fetch_cti_set`].
#[derive(Debug)]
pub enum CtiError {
    /// Error when trying to [`fetch_json`] cards.
    CardFetchError(FetchError),
    /// Error when trying to [`fetch_json`] sigils.
    SigilFetchError(FetchError),
    /// Invalid Rarity.
    UnknownRarity(String),
    /// Invalid Temple.
    UnknownTemple(String),
    /// Invalid cost format. The cost doesn't follow each component are a number then the cost
    /// with space between and every cost is separated by `,`.
    InvalidCostFormat(String),
    /// Unknow cost.
    UnknowCost(String),
    /// Invalid Mox color.
    UnknowMox(String),
}

impl Display for CtiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CtiError::CardFetchError(e) => write!(f, "cannot fetch cards due to: {e}"),
            CtiError::SigilFetchError(e) => write!(f, "cannot fetch sigils due to: {e}"),
            CtiError::UnknownRarity(r) => write!(f, "unknown rarity: {r}"),
            CtiError::UnknownTemple(r) => write!(f, "unknown temple: {r}"),
            CtiError::InvalidCostFormat(s) => write!(f, "invalid cost: {s}"),
            CtiError::UnknowCost(c) => write!(f, "unknow cost: {c}"),
            CtiError::UnknowMox(m) => write!(f, "unknow mox: {m}"),
        }
    }
}

/// Json scheme for Cti card.
///
/// We make our own portrait url because there some issue with the one on the sheet
#[derive(Deserialize)]
struct CtiCard {
    // Normal name are sometime wrong so we will just grab the internal name
    #[serde(rename = "Internal Name")]
    name: String,
    #[serde(rename = "Flavor")]
    description: String,

    #[serde(rename = "Temple")]
    temple: String,
    #[serde(rename = "Rarity")]
    rarity: String,

    #[serde(rename = "Cost")]
    cost: String,

    #[serde(rename = "Power")]
    attack: String,
    #[serde(rename = "Health")]
    health: String,

    #[serde(rename = "Token")]
    token: String,

    #[serde(rename = "Sigil 1")]
    sigil_1: String,
    #[serde(rename = "Sigil 2")]
    sigil_2: String,
    #[serde(rename = "Sigil 3")]
    sigil_3: String,
    #[serde(rename = "Sigil 4")]
    sigil_4: String,
}

/// Json scheme for Cti sigil
#[derive(Deserialize)]
struct CtiSigil {
    // I can't find any different between the internal and normal so im just going to grab normal
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Description")]
    text: String,
}
