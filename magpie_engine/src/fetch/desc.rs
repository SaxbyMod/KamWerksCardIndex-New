use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    fetch::fetch_json, Attack, Card, Costs, Mox, Rarity, Set, SetCode, Temple, Traits, TraitsFlag,
};

use super::{SetError, SetResult};

/// Descryption's [`Costs`] extension.
#[derive(Default, Clone, PartialEq)]
pub struct DescCosts {
    /// Links cost.
    pub link: isize,
    /// Gold cost.
    pub gold: isize,
}

/// Fetch Descryption from the
/// [sheet](https://docs.google.com/spreadsheets/d/1EjOtqUrjsMRl7wiVMN7tMuvAHvkw7snv1dNyFJIFbaE).
pub fn fetch_desc_set(code: SetCode) -> SetResult<(), DescCosts> {
    let card_url = "https://opensheet.elk.sh/1EjOtqUrjsMRl7wiVMN7tMuvAHvkw7snv1dNyFJIFbaE/2";
    let card_raw: Vec<DescCard> =
        fetch_json(card_url).map_err(|e| SetError::FetchError(e, card_url.to_string()))?;

    let sigil_url = "https://opensheet.elk.sh/1EjOtqUrjsMRl7wiVMN7tMuvAHvkw7snv1dNyFJIFbaE/4";
    let sigils: Vec<DescSigil> =
        fetch_json(sigil_url).map_err(|e| SetError::FetchError(e, sigil_url.to_string()))?;

    let mut cards = Vec::with_capacity(card_raw.len());
    let sigils_description = {
        let mut h = HashMap::with_capacity(sigils.len());
        for s in sigils {
            h.insert(s.name, s.text);
        }
        h
    };

    for card in card_raw {
        if card.name.is_empty() {
            continue;
        }

        let mut temple = Temple::empty();

        if !is_empty(&card.temple) {
            for t in card.temple.split(", ") {
                temple |= match t {
                    "Leshy" => Temple::BEAST,

                    "Grimora" => Temple::UNDEAD,
                    "P03" => Temple::TECH,
                    "Magnificus" => Temple::MAGICK,
                    "Galliard" => Temple::ARTISTRY,

                    _ => return Err(SetError::UnknownTemple(t.to_owned())),
                }
            }
        }

        let mut costs = Costs::<DescCosts>::default();

        if !is_empty(&card.cost) {
            if card.cost.contains(',') | !card.cost.contains(' ') {
                for m in card.cost.split(", ") {
                    costs.mox |= match m {
                        "Orange" => Mox::O,
                        "Green" => Mox::G,
                        "Blue" => Mox::B,
                        "Black" => {
                            if costs.mox.is_empty() {
                                Mox::K
                            } else {
                                Mox::P1
                            }
                        }
                        _ => return Err(SetError::UnknownMoxColor(m.to_owned())),
                    }
                }
            } else {
                let (count, cost) = {
                    let mut t = card.cost.split_whitespace();
                    (
                        t.next().unwrap().parse::<isize>().unwrap(),
                        t.next().unwrap(),
                    )
                };

                match cost.to_lowercase().as_str() {
                    "blood" => costs.blood += count,
                    "bone" | "bones" => costs.bone += count,
                    "energy" => costs.energy += count,
                    "links" | "link" => costs.extra.link += count,
                    "gold" | "golds" => costs.extra.gold += count,
                    _ => return Err(SetError::UnknownCost(cost.to_owned())),
                }
            }
        }

        let card = Card {
            set: code,
            portrait: format!(
                "https://raw.githubusercontent.com/EternalHours/Descryption/main/images/portraits/{}_{}.png", 
                if card.traits_unique.contains("Full Art") { 
                    "fullpixel"
                } else {
                   "pixelportrait"
                },
                card.name
                    .to_lowercase()
                    .replace([' ', '\'', '(', ')', '-', '.'], "")
            ),
            name: card.name,
            description: String::new(),
            rarity: if is_empty(&card.rarity) {
                Rarity::COMMON
            } else {
                match card.rarity.as_str() {
                    "Common" => Rarity::COMMON,
                    "Rare" => Rarity::RARE,
                    "Unique" => Rarity::UNIQUE,
                    _ => return Err(SetError::UnknownRarity(card.rarity)),
                }
            },
            temple,
            tribes: (!is_empty(&card.tribes)).then_some(card.tribes),
            attack: if let Ok(a) = card.attack.parse() {
                Attack::Num(a)
            } else {
                Attack::Str(card.attack)
            },
            health: card.health.parse().unwrap_or(0),
            sigils: if is_empty(&card.sigils) {
                vec![]
            } else {
                card.sigils
                    .split(", ")
                    .map(|s| {
                        let s = s.to_owned();
                        if sigils_description.contains_key(&s) {
                            s
                        } else {
                            String::from("UNDEFINEDED SIGILS")
                        }
                    })
                    .collect()
            },
            costs: if is_empty(&card.cost) {
                None
            } else {
                Some(costs)
            },
            traits: Some(Traits {
                strings: {
                    (!(is_empty(&card.traits_unique) && is_empty(&card.traits))).then(|| {
                        card.traits_unique
                            .split("; ")
                            .chain(card.traits.split("; "))
                            .map(ToOwned::to_owned)
                            .filter(|t| !is_empty(t))
                            .collect()
                    })
                },
                flags: TraitsFlag::empty(),
            }),
            related: vec![],
            extra: (),
        };

        cards.push(card);
    }

    Ok(Set {
        code,
        name: String::from("Descryption"),
        cards,
        sigils_description,
    })
}

fn is_empty(str: &str) -> bool {
    str.is_empty() || str == "-" || str == "N/A"
}

/// Json scheme for desc card.
#[derive(Deserialize)]
struct DescCard {
    #[serde(rename = "Name")]
    #[serde(default)]
    name: String,

    #[serde(rename = "Scrybes")]
    #[serde(default)]
    temple: String,
    #[serde(rename = "Rarity")]
    #[serde(default)]
    rarity: String,

    #[serde(rename = "Cost")]
    #[serde(default)]
    cost: String,

    #[serde(rename = "Power")]
    #[serde(default)]
    attack: String,
    #[serde(rename = "Health")]
    #[serde(default)]
    health: String,

    #[serde(rename = "Sigils")]
    #[serde(default)]
    sigils: String,

    #[serde(rename = "Traits")]
    #[serde(default)]
    traits: String,
    #[serde(rename = "Traits (Named)")]
    #[serde(default)]
    traits_unique: String,

    #[serde(rename = "Tribes")]
    #[serde(default)]
    tribes: String,
}

/// Json scheme for desc sigil.
#[derive(Deserialize)]
struct DescSigil {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Text")]
    text: String,
}
