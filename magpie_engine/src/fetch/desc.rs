use std::collections::HashMap;

use serde::Deserialize;

use crate::data::cards;
use crate::{Set, SetCode};

use super::{fetch_json, FetchError};

pub fn fetch_desc(code: SetCode) -> Result<Set<()>, DescError> {
    todo!();

    //let card_raw: Vec<DescCard> =
    //    fetch_json("https://opensheet.elk.sh/1EjOtqUrjsMRl7wiVMN7tMuvAHvkw7snv1dNyFJIFbaE/Cards")
    //        .map_err(DescError::CardFetchError)?;
    //
    //let sigils: Vec<DescSigil> = fetch_json(
    //    "https://opensheet.elk.sh/1EjOtqUrjsMRl7wiVMN7tMuvAHvkw7snv1dNyFJIFbaE/[Sigils]",
    //)
    //.map_err(DescError::SigilFetchError)?;
    //
    //let mut cards = Vec::with_capacity(card_raw.len());
    //let sigils_description = {
    //    let mut h = HashMap::with_capacity(sigils.len());
    //    for s in sigils {
    //        h.insert(s.name, s.text);
    //    }
    //    h
    //};
    //
    //for card in card_raw {
    //    cards.push(card)
    //}
    //
    //Ok(Set {
    //    code,
    //    name: String::from("Descryption"),
    //    cards,
    //    sigils_description,
    //})
}

pub enum DescError {
    CardFetchError(FetchError),
    SigilFetchError(FetchError),
}

/// Json scheme for desc card
#[derive(Deserialize)]
struct DescCard {
    #[serde(rename = "Name")]
    name: String,

    #[serde(rename = "Scrybes")]
    temple: String,
    #[serde(rename = "Rarity")]
    rarity: String,

    #[serde(rename = "Cost")]
    cost: String,

    #[serde(rename = "Power")]
    attack: String,
    #[serde(rename = "Heakth")]
    health: String,

    #[serde(rename = "Sigils")]
    sigils: String,

    #[serde(rename = "Traits")]
    traits: String,
}

#[derive(Deserialize)]
struct DescSigil {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Text")]
    text: String,
}
