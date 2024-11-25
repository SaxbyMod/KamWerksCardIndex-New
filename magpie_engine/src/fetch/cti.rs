use std::collections::HashMap;
use serde::Deserialize;
use crate::{fetch::{fetch_from_notion, FetchError}, Attack, Card, Costs, Mox, MoxCount, Rarity, Set, SetCode, Temple};

use super::{SetError, SetResult};

/// Struct to represent the wrapping of the response containing the `results` field.
#[derive(Deserialize, Debug)] // Derive Debug for printing
struct NotionResponse {
    results: Option<Vec<CtiCard>>, // Make the results field optional to handle missing results
}

/// Fetch Custom TCG Inscryption from the
/// [Notion Database](https://www.notion.so/inscryption-pvp-wiki/Custom-TCG-Inscryption-3f22fc55858d4cfab2061783b5120f87).
#[allow(clippy::too_many_lines)]
pub fn fetch_cti_set(code: SetCode) -> SetResult<(), ()> {
    
    let notion_api_key = std::env::var("NOTION_API_KEY")
        .map_err(|_| SetError::MissingApiKey("Notion API key not found".to_string()))?;
    
        match std::env::var("NOTION_API_KEY") {
            Ok(key) => println!("Retrieved API Key: {}", key),
            Err(err) => println!("Failed to retrieve API Key: {:?}", err),
        }
        

    let card_url = "https://api.notion.com/v1/databases/e19c88aa75b44bfe89321bcde8dc7d9f/query";
    let sigil_url = "https://api.notion.com/v1/databases/933d6166cb3f4ee89db51e4cf464f5bd/query";

    // Example payload (empty query for fetching all items)
    let payload = serde_json::json!({});

    let raw_response: NotionResponse =
        fetch_from_notion(card_url, Some(&notion_api_key), Some(payload))
            .map_err(|e| SetError::FetchError(e, card_url.to_string()))?;

    println!("{:?}", raw_response);

    let raw_card = raw_response.results.ok_or_else(|| SetError::DeserializeError(card_url.to_string()))?;

    // Fetch sigils
    let sigil: Vec<CtiSigil> = fetch_from_notion(sigil_url, Some(&notion_api_key), None)
        .map_err(|e| SetError::FetchError(e, sigil_url.to_string()))?;

    // Initialize containers for the cards and sigils descriptions
    let mut cards = Vec::with_capacity(raw_card.len());
    let mut sigils_description = HashMap::with_capacity(sigil.len());

    // Populate the sigils description map
    for s in sigil {
        sigils_description.insert(s.name, s.text.replace('\n', ""));
    }

    sigils_description.insert(
        String::from("UNDEFINDED SIGILS"),
        "THIS SIGIL IS NOT DEFINED BY THE SET".to_owned(),
    );

    // Process the raw card data
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
                        .ok_or_else(|| SetError::InvalidCostFormat(card.cost.clone()))?
                        .parse::<isize>()
                        .map_err(|_| SetError::InvalidCostFormat(card.cost.clone()))?;

                    (
                        first,
                        t.next()
                            .ok_or_else(|| SetError::InvalidCostFormat(card.cost.clone()))?,
                    )
                };

                match cost.as_str() {
                    "blood" => t.blood += count,
                    "bone" => t.bone += count,
                    "energy" => t.energy += count,
                    m @ ("ruby" | "sapphire" | "emerald" | "prism") => match m {
                        "ruby" => {
                            t.mox |= Mox::O;
                            mox_count.o += count as usize;
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
                    c => return Err(SetError::UnknownCost(c.to_string())),
                }
            }

            // Only include the moxes if they are not the default all 1
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
                _ => return Err(SetError::UnknownRarity(card.rarity)),
            },
            temple: match card.temple.as_str() {
                "Beast" => Temple::BEAST,
                "Undead" => Temple::UNDEAD,
                "Tech" => Temple::TECH,
                "Magicks" => Temple::MAGICK,
                "Terrain/Extras" => Temple::empty(),
                _ => return Err(SetError::UnknownTemple(card.temple))
            },
            tribes: None,
            attack: Attack::Num(card.attack.parse().unwrap_or(0)),
            health: card.health.parse().unwrap_or(0),
            sigils: [card.sigil_1, card.sigil_2, card.sigil_3, card.sigil_4]
                .into_iter()
                .filter(|s| !s.is_empty())
                .map(|s| if sigils_description.contains_key(&s) { s } else { String::from("UNDEFINED SIGIL") })
                .collect(),
            costs,
            traits: None,
            related: if card.token.is_empty() {
                vec![]
            } else {
                card.token.split(", ").map(ToOwned::to_owned).collect()
            },
            extra: (),
        });
    }

    // Return the assembled set
    Ok(Set {
        code,
        name: String::from("Custom TCG Inscryption"),
        cards,
        sigils_description,
    })
}

/// Json scheme for Cti card.
#[derive(Deserialize, Debug)] // Derive Debug for printing
struct CtiCard {
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

/// A Sigil in the set.
#[derive(Deserialize, Debug)] // Derive Debug for printing
struct CtiSigil {
    name: String,
    text: String,
}
