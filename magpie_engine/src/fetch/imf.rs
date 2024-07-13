//! Implementation for [IMF] set
//!
//! [IMF]: https://107zxz.itch.io/inscryption-multiplayer-godot

use crate::Ptr;
use crate::{Card, Costs, Mox, Rarity, Set, SetCode, SpAtk, Temple, TraitFlag, Traits};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;

use super::{fetch_json, FetchError};

/// Fetch a IMF Set from a url
pub fn fetch_imf_set(url: &str, code: SetCode) -> Result<Set, ImfError> {
    let set: ImfSetJson = fetch_json(url).map_err(|e| ImfError::FetchError(e))?;

    // idk why i need explicit type here but rust want it there
    let mut cards: Vec<Ptr<Card>> = Vec::with_capacity(set.cards.len() + 1);

    let pools = {
        let mut m = HashMap::with_capacity(7);
        for p in vec!["rare", "ban", "beast", "undead", "tech", "magick"] {
            m.insert(p.to_string(), vec![]);
        }
        m
    };

    let undefined_sigil = Ptr::new("UNDEFINDED SIGILS".to_string());

    let mut sigil_rc = HashMap::with_capacity(set.sigils.len());
    let mut sigils_description = HashMap::with_capacity(set.sigils.len());

    for s in set.sigils {
        // Convert the sigil to a rc
        let rc = Ptr::new(s.0.clone());

        sigil_rc.insert(s.0, rc.clone());
        sigils_description.insert(rc.clone(), s.1);
    }

    for c in set.cards {
        let card = Ptr::new(Card {
            set: code.clone(),
            portrait: c
                .pixport_url
                .is_empty()
                .then_some(format!(
                    "https://github.com/107zxz/inscr-onln/raw/main/gfx/pixport/{}.png",
                    c.name.replace(" ", "%20")
                ))
                .unwrap_or(c.pixport_url),
            name: c.name,
            description: c.description,
            rarity: if c.rare { Rarity::RARE } else { Rarity::COMMON },
            temple: Temple::EMPTY
                .set_if(Temple::BEAST, c.blood_cost != 0)
                .set_if(Temple::UNDEAD, c.bone_cost != 0)
                .set_if(Temple::TECH, c.energy_cost != 0)
                .set_if(Temple::MAGICK, c.mox_cost.len() != 0)
                .into(),
            attack: c.attack,
            health: c.health,
            sigils: c
                .sigils
                .iter()
                .map(|s| sigil_rc.get(s).unwrap_or(&undefined_sigil).clone())
                .collect(),
            sp_atk: match c.atkspecial.as_str() {
                "" => None,
                atk => Some(match atk {
                    "mox" => SpAtk::MOX,
                    "green_mox" => SpAtk::GREEN_MOX,
                    "mirror" => SpAtk::MIRROR,
                    "ant" => SpAtk::ANT,
                    _ => return Err(ImfError::InvalidSpAtk(c.atkspecial)),
                }),
            },
            costs: ((c.blood_cost > 0)
                | (c.bone_cost > 0)
                | (c.energy_cost > 0)
                | (c.mox_cost.len() > 0))
                .then(|| Costs {
                    blood: c.blood_cost,
                    bone: c.bone_cost,
                    energy: c.energy_cost,
                    mox: c
                        .mox_cost
                        .iter()
                        .fold(Mox(0), |flags, mox| match mox.as_str() {
                            "Orange" => flags | Mox::R,
                            "Green" => flags | Mox::R,
                            "Blue" => flags | Mox::R,
                            _ => unreachable!(),
                        }),
                    mox_count: None,
                }),
            traits: (c.conduit | c.banned | c.nosac | c.nohammer).then(|| Traits {
                traits: None,
                flags: TraitFlag::EMPTY
                    .set_if(TraitFlag::CONDUCTIVE, c.conduit)
                    .set_if(TraitFlag::BAN, c.banned)
                    .set_if(TraitFlag::TERRAIN, c.nosac)
                    .set_if(TraitFlag::HARD, c.nohammer)
                    .into(),
            }),
        });

        cards.push(card)
    }
    Ok(Set {
        code,
        cards,
        name: set.ruleset,
        sigils_description,
        pools,
    })
}

#[derive(Debug)]
/// Error that happen when calling [`fetch_imf_set`]
pub enum ImfError {
    /// Error when calling [`fetch_json`]
    FetchError(FetchError),
    /// Invalid `atkspecial` when converting to [`Card`]
    InvalidSpAtk(String),
}

impl Display for ImfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImfError::FetchError(e) => write!(f, "unable to fetch json: {e}"),
            ImfError::InvalidSpAtk(e) => write!(f, "invalid special attack: {e}"),
        }
    }
}

impl Error for ImfError {}
/// Json scheme for IMF set
#[derive(Deserialize, Debug)]
struct ImfSetJson {
    ruleset: String,
    cards: Vec<ImfCardJson>,
    sigils: HashMap<String, String>,
}

/// Json scheme for IMF card
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ImfCardJson {
    pub name: String,

    #[serde(default)]
    pub description: String,

    pub attack: isize,
    pub health: isize,

    #[serde(default)]
    pub sigils: Vec<String>,

    #[serde(default)]
    pub atkspecial: String,

    #[serde(default)]
    pub blood_cost: isize,
    #[serde(default)]
    pub bone_cost: isize,
    #[serde(default)]
    pub energy_cost: isize,
    #[serde(default)]
    pub mox_cost: Vec<String>,

    #[serde(default)]
    pub pixport_url: String,

    #[serde(default)]
    pub conduit: bool,
    #[serde(default)]
    pub banned: bool,
    #[serde(default)]
    pub rare: bool,
    #[serde(default)]
    pub nosac: bool,
    #[serde(default)]
    pub nohammer: bool,

    #[serde(default)]
    pub evolution: String,
    #[serde(default)]
    pub left_half: String,
    #[serde(default)]
    pub right_half: String,
}
