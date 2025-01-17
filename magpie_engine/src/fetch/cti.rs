use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::{fetch::{fetch_from_notion, FetchError}, Attack, Card, Costs, Mox, MoxCount, Rarity, Set, SetCode, Temple};

use super::{SetError, SetResult};

#[derive(Deserialize, Debug)]
struct NotionResponse {
    results: Option<Vec<NotionResult>>, // Wrap the results in an Option<Vec> to handle missing results
}

#[derive(Deserialize, Debug)]
struct NotionResult {
    properties: CtiCard, // The properties field contains a CtiCard
}

#[derive(Deserialize, Debug)]
struct NotionResponseSigils {
    results: Option<Vec<NotionResultSigils>>, // Wrap the results in an Option<Vec> to handle missing results
}

#[derive(Deserialize, Debug)]
struct NotionResultSigils {
    properties: CtiSigil, // The properties field contains a CtiCard
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
    let payload2 = serde_json::json!({});

    let raw_response: NotionResponse =
        fetch_from_notion(card_url, Some(&notion_api_key), Some(payload))
            .map_err(|e| SetError::FetchError(e, card_url.to_string()))?;

    println!("{:?}", raw_response);

    let raw_card = raw_response.results.ok_or_else(|| SetError::DeserializeError(card_url.to_string()))?;

    // Fetch sigils
    let sigil: NotionResponseSigils =
        fetch_from_notion(sigil_url, Some(&notion_api_key), Some(payload2))
            .map_err(|e| SetError::FetchError(e, sigil_url.to_string()))?;
    
    println!("{:?}", sigil);

    let raw_sigil = sigil.results.ok_or_else(|| SetError::DeserializeError(sigil_url.to_string()))?;

    // Initialize containers for the cards and sigils descriptions
    let mut cards = Vec::with_capacity(raw_card.len());
    let mut sigils_description = HashMap::with_capacity(raw_sigil.len());

    // Populate the sigils description map
    for s in raw_sigil {
        sigils_description.insert(
            s.properties.name.rich_text[0].plain_text.clone(), 
            s.properties.description.rich_text[0].plain_text.clone().replace('\n', "")
        );
    }

    // Process the raw card data
    for card in raw_card {
        let costs;
        if card.properties.cost.rich_text[0].plain_text != "Free" && !card.properties.cost.rich_text[0].plain_text.is_empty() {
            let mut t: Costs<()> = Costs::default();
            let mut mox_count = MoxCount::default();

            for c in card
                .properties.cost.rich_text[0].plain_text
                .to_lowercase()
                .replace("bones", "bone")
                .split(", ")
            {
                let (count, cost) = {
                    let s = c.to_lowercase().trim().to_string();
                    let mut t = s.split_whitespace().map(ToOwned::to_owned);

                    let first = t
                        .next()
                        .ok_or_else(|| SetError::InvalidCostFormat(card.properties.cost.rich_text[0].plain_text.clone()))?
                        .parse::<isize>()
                        .map_err(|_| SetError::InvalidCostFormat(card.properties.cost.rich_text[0].plain_text.clone()))?;

                    (
                        first,
                        t.next()
                            .ok_or_else(|| SetError::InvalidCostFormat(card.properties.cost.rich_text[0].plain_text.clone()))?,
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
            portrait: card.properties.image.url.clone(), // Using the image URL directly
            set: code,
            name: card.properties.name.rich_text[0].plain_text.clone(),
            description: card.properties.flavor.rich_text[0].plain_text.clone(),
            rarity: match card.properties.rarity.select.name.as_str() {
                "Common" | "Common (Joke Card)" | "" => Rarity::COMMON,
                "Uncommon" => Rarity::UNCOMMON,
                "Rare" => Rarity::RARE,
                "Talking" | "Deathcard" => Rarity::UNIQUE,
                "Side-Deck" => Rarity::SIDE,
                _ => return Err(SetError::UnknownRarity(card.properties.rarity.select.name)),
            },
            temple: match card.properties.temple.select.name.as_str() {
                "Beast" => Temple::BEAST,
                "Undead" => Temple::UNDEAD,
                "Tech" => Temple::TECH,
                "Magicks" => Temple::MAGICK,
                "Terrain/Extras" => Temple::empty(),
                _ => return Err(SetError::UnknownTemple(card.properties.temple.select.name))
            },
            tribes: None,
            attack: Attack::Num(card.properties.power.rich_text[0].plain_text.parse().unwrap_or(0)),
            health: card.properties.health.rich_text[0].plain_text.parse().unwrap_or(0),
            sigils: card.properties.sigil_1
            .iter()
            .chain(card.properties.sigil_2.iter())
            .chain(card.properties.sigil_3.iter())
            .chain(card.properties.sigil_4.iter())
            .filter_map(|sigil| {
                let sigil_name = sigil.rich_text.get(0)?.plain_text.clone();
                if sigil_name.is_empty() {
                    None
                } else {
                    Some(
                        sigils_description
                            .get(&sigil_name)
                            .cloned()
                            .unwrap_or_else(|| "UNDEFINED SIGIL".to_string()),
                    )
                }
            })
            .collect(),
            costs,
            traits: None,
            related: card.properties.token
            .as_ref()
            .and_then(|token| token.rich_text.get(0))
            .map(|token_text| vec![token_text.plain_text.clone()])
            .unwrap_or_else(Vec::new),      
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

#[derive(Serialize, Deserialize, Debug)]
pub struct CtiCard {
    #[serde(rename = "Name")]
    name: RichTextContent,

    #[serde(rename = "Sigil 4")]
    sigil_4: Option<RichTextContent>,

    #[serde(rename = "Health")]
    health: RichTextContent,

    #[serde(rename = "Sigil 3")]
    sigil_3: Option<RichTextContent>,

    #[serde(rename = "Image")]
    image: Link, // URL as String

    #[serde(rename = "Sigil 1")]
    sigil_1: Option<RichTextContent>,

    #[serde(rename = "Token")]
    token: Option<RichTextContent>,

    #[serde(rename = "Cost")]
    cost: RichTextContent,

    #[serde(rename = "Rarity")]
    rarity: SelectPrerequisite,

    #[serde(rename = "Flavor")]
    flavor: RichTextContent,

    #[serde(rename = "Power")]
    power: RichTextContent,

    #[serde(rename = "Wiki-Page")]
    wiki_page: Link, // URL as String

    #[serde(rename = "From")]
    from: RichTextContent,

    #[serde(rename = "Sigil 2")]
    sigil_2: Option<RichTextContent>,

    #[serde(rename = "Temple")]
    temple: SelectPrerequisite,

    #[serde(rename = "Internal Name")]
    internal_name: InternalName,
}

// RichText type that represents rich_text structure
#[derive(Serialize, Deserialize, Debug)]
pub struct RichTextContent {
    #[serde(rename = "rich_text")]
    pub rich_text: Vec<PlainText>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlainText {
    #[serde(rename = "plain_text")]
    pub plain_text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Link {
    #[serde(rename = "url")]
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SelectPrerequisite {
    #[serde(rename = "select")]
    pub select: SelectOption
}

// Select option for Rarity and Temple (with the name)
#[derive(Serialize, Deserialize, Debug)]
pub struct SelectOption {
    #[serde(rename = "name")]
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InternalName {
    #[serde(rename = "title")]
    pub title: Vec<InternalNamePlainText>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InternalNamePlainText {
    #[serde(rename = "plain_text")]
    pub plain_text: String,
}

/// A Sigil in the set.
#[derive(Deserialize, Debug)] // Derive Debug for printing
struct CtiSigil {
    #[serde(rename = "Description")]
    description: RichTextContent,
    #[serde(rename = "Category")]
    category: SelectPrerequisite,
    #[serde(rename = "Name")]
    name: RichTextContent,
    #[serde(rename = "Internal Name")]
    internam_name: InternalName,
}