//! Implementation for [IMF] set
//!
//! [IMF]: https://107zxz.itch.io/inscryption-multiplayer-godot

use crate::cards::{Card, Costs, Mox, Rarity, Set, SetCode, SpAtk, Temple, TraitFlag, Traits};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::rc::Rc;

use super::{fetch_json, FetchError};

/// Fetch a IMF Set from a url
pub fn fetch_imf_set(url: &str, code: SetCode) -> Result<ImfSet, ImfError> {
    let set: ImfSetJson = match fetch_json(url) {
        Ok(it) => it,
        Err(e) => return Err(ImfError::FetchError(e)),
    };

    // idk why i need explicit type here but rust want it there
    let mut cards: Vec<Rc<dyn Card>> = Vec::with_capacity(set.cards.len() + 1);

    let pools = {
        let mut m = HashMap::with_capacity(7);
        for p in vec!["rare", "ban", "beast", "undead", "tech", "magick"] {
            m.insert(p.to_string(), vec![]);
        }
        m
    };

    let undefined_sigil = Rc::new("UNDEFINDED SIGILS".to_string());

    let mut sigil_rc = HashMap::with_capacity(set.sigils.len());
    let mut sigils_description = HashMap::with_capacity(set.sigils.len());

    for s in set.sigils {
        // Convert the sigil to a rc
        let rc = Rc::new(s.0.clone());

        sigil_rc.insert(s.0, rc.clone());
        sigils_description.insert(rc.clone(), s.1);
    }

    for c in set.cards {
        let card = Rc::new(ImfCard {
            set: code.clone(),
            name: c.name,
            description: c.description,
            rarity: if c.rare { Rarity::RARE } else { Rarity::COMMON },
            temple: Temple::EMPTY
                .set_if(Temple::BEAST, c.blood_cost != 0)
                .set_if(Temple::UNDEAD, c.bone_cost != 0)
                .set_if(Temple::TECH, c.energy_cost != 0)
                .set_if(Temple::MAGICK, c.mox_cost.len() != 0),
            attak: c.attack,
            health: c.health,
            sigils: c
                .sigils
                .iter()
                .map(|s| sigil_rc.get(s).unwrap_or(&undefined_sigil).clone())
                .collect(),
            sp_atk: match c.atkspecial.as_str() {
                "" => SpAtk::NONE,
                atk => match atk {
                    "mox" => SpAtk::MOX,
                    "green_mox" => SpAtk::GREEN_MOX,
                    "mirror" => SpAtk::MIRROR,
                    "ant" => SpAtk::ANT,
                    _ => return Err(ImfError::InvalidSpAtk(c.atkspecial)),
                },
            },
            costs: Costs::Costs {
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
            },
            traits: Traits::Traits {
                traits: None,
                flags: TraitFlag::EMPTY
                    .set_if(TraitFlag::CONDUCTIVE, c.conduit)
                    .set_if(TraitFlag::BAN, c.banned)
                    .set_if(TraitFlag::TERRAIN, c.nosac)
                    .set_if(TraitFlag::HARD, c.nohammer)
                    | 0, // convert the flag into the correct type
            },
        });

        cards.push(card)
    }
    Ok(ImfSet {
        code,
        cards,
        name: set.ruleset,
        sigils_description,
        pools,
    })
}

/// Implementation of [`Card`] for IMF card
#[derive(Debug)]
pub struct ImfCard {
    set: SetCode,
    name: String,
    description: String,

    rarity: Rarity,
    temple: Temple,

    attak: isize,
    health: isize,

    sigils: Vec<Rc<String>>,

    sp_atk: SpAtk,

    costs: Costs,

    traits: Traits,
}

/// Implementation of [`Set`] for IMF Card
#[derive(Debug)]
pub struct ImfSet {
    code: SetCode,
    name: String,
    cards: Vec<Rc<dyn Card>>,
    sigils_description: HashMap<Rc<String>, String>,
    pools: HashMap<String, Vec<Rc<dyn Card>>>,
}

impl Card for ImfCard {
    fn set(&self) -> &SetCode {
        &self.set
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn rarity(&self) -> &Rarity {
        &self.rarity
    }

    fn temple(&self) -> u16 {
        self.temple.0
    }

    fn attack(&self) -> isize {
        self.attak
    }

    fn health(&self) -> isize {
        self.health
    }

    fn sigils(&self) -> &Vec<Rc<String>> {
        &self.sigils
    }

    fn sp_atk(&self) -> &SpAtk {
        &self.sp_atk
    }

    fn costs(&self) -> &Costs {
        &self.costs
    }

    fn traits(&self) -> &Traits {
        &self.traits
    }
}

impl Set for ImfSet {
    fn code(&self) -> &SetCode {
        &self.code
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn cards(&self) -> &Vec<Rc<dyn Card>> {
        &self.cards
    }

    fn sigils_description(&self) -> &HashMap<Rc<String>, String> {
        &self.sigils_description
    }

    fn pools(&self) -> &HashMap<String, Vec<Rc<dyn Card>>> {
        &self.pools
    }
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
