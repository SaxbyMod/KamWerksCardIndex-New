//! Contain implementation for generate card embed from card and a few other info
use magpie_engine::prelude::*;
use poise::serenity_prelude::{colours::roles, CreateEmbed, CreateEmbedFooter};

use crate::{
    emojis::{imf, number, ToEmoji},
    hash_card_url, Card, Set,
};

type EmbedRes = (CreateEmbed, String);

/// Generate card embed from a card data.
///
/// The name of the card is store in the embed title along with the set name and any trais flags
/// icon.
///
/// General info like cost, stats, desciption are store inside the embed description because they
/// will always be there
///
/// Sigils and other traits use the embed field because they are optional and not every card have
/// them.
pub fn gen_embed(rank: f32, card: &Card, set: &Set, compact: bool) -> CreateEmbed {
    // The specific gen embed function should return the embed and the footer that they would like
    // to add.

    let (embed, footer) = match card.set.code() {
        "aug" => gen_aug_embed(card, set, compact),
        "std" | "ete" | "egg" => gen_imf_embed(card, set, compact),
        _ => unimplemented!(),
    };
    embed.footer(CreateEmbedFooter::new(format!(
        "{footer}\nMatch {:.2}% with the search term",
        rank * 100.
    )))
}

fn gen_imf_embed(card: &Card, set: &Set, compact: bool) -> EmbedRes {
    let mut embed = CreateEmbed::new()
        .color(if card.rarity.eq(&Rarity::RARE) {
            roles::GREEN
        } else {
            roles::LIGHT_GREY
        })
        .title(format!(
            "{} ({}) {}",
            card.name,
            set.name,
            match &card.traits {
                Some(tr) => TraitsFlag::from(tr.flags).to_emoji(),
                None => String::new(),
            }
        ));

    let mut desc = if card.description.is_empty() || compact {
        String::new()
    } else {
        format!("*{}*\n\n", card.description)
    };

    desc.push_str(&cost_str(card)); // the card cost
    desc.push('\n'); // stat separator

    // imf shouldn't have any other thing
    #[allow(clippy::match_wildcard_for_single_variants)]
    desc.push_str(&format!(
        "**Stat:** {} / {}\n",
        match &card.attack {
            Attack::Num(a) => a.to_string(),
            Attack::SpAtk(sp) => sp.to_emoji(),
            _ => unreachable!(),
        },
        card.health
    ));

    if !card.sigils.is_empty() {
        if compact {
            desc.push_str(&format!("**Sigils:** {}\n", card.sigils.join(", ")));
        } else {
            let mut desc = String::with_capacity(card.sigils.iter().map(String::len).sum());

            for s in &card.sigils {
                let text = set.sigils_description.get(s).unwrap();
                desc.push_str(&format!("**{s}:** {text}\n"));
            }

            embed = embed.field("== SIGILS ==", desc, false);
        }
    }

    if !card.related.is_empty() {
        let value = format!("**Related:** {}\n", card.related.join(", "));
        if compact {
            desc.push_str(&value);
        } else {
            embed = embed.field("== EXTRA INFO ==", value, false);
        }
    }

    if compact {
        desc = desc.replace("\n\n", "\n");
    }

    (
        embed
            .description(desc)
            .thumbnail(format!("attachment://{}.png", hash_card_url(card))),
        String::new(), // empty footer
    )
}

fn gen_aug_embed(card: &Card, set: &Set, compact: bool) -> EmbedRes {
    let color = if let Some(t) = Temple::from(card.temple).flags().next() {
        match *t {
            Temple::BEAST => roles::DARK_GOLD,
            Temple::UNDEAD => roles::GREEN,
            Temple::TECH => roles::BLUE,
            Temple::MAGICK => roles::RED,
            Temple::FOOL => roles::MAGENTA,
            _ => unreachable!(),
        }
    } else {
        unreachable!()
    };

    let mut embed = CreateEmbed::new().color(color).title(format!(
        "{} ({}) {}",
        card.name,
        set.name,
        match &card.traits {
            Some(tr) => TraitsFlag::from(tr.flags).to_emoji(),
            None => String::new(),
        }
    ));

    let mut desc = if card.description.is_empty() || compact {
        String::new()
    } else {
        format!("*{}*\n\n", card.description)
    };

    desc.push_str(&format!(
        "**Tier:** {}\n",
        match &card.rarity {
            Rarity::UNIQUE => String::from("Talking"),
            a => a.to_string(),
        }
    ));
    if let Some(t) = &card.tribes {
        desc.push_str(&format!("**Tribes:** {t}\n"));
    }

    desc.push('\n'); // cost separator
    desc.push_str(&cost_str(card)); // the card cost
    desc.push('\n'); // stat separator

    // remove this once we actually parse aug spatk
    #[allow(clippy::match_wildcard_for_single_variants)]
    desc.push_str(&format!(
        "**Stat:** {} / {}",
        match &card.attack {
            Attack::Num(a) => a.to_string(),
            Attack::Str(s) => s.to_owned(),
            _ => unreachable!(),
        },
        card.health
    ));

    if !card.sigils.is_empty() {
        if compact {
            desc.push_str(&format!("**Sigils:** {}\n", card.sigils.join(", ")));
        } else {
            let mut desc = String::with_capacity(card.sigils.iter().map(String::len).sum());

            for s in &card.sigils {
                let text = set.sigils_description.get(s).unwrap();
                desc.push_str(&format!("**{s}:** {text}\n"));
            }

            embed = embed.field("== SIGILS ==", desc, false);
        }
    }

    if let Some(Traits {
        strings: Some(t), ..
    }) = &card.traits
    {
        if compact {
            desc.push_str(&format!("**Traits:** {}", t.join(", ")));
        } else {
            let mut desc = String::with_capacity(t.iter().map(String::len).sum());

            for s in t {
                let text = set.sigils_description.get(s).unwrap();
                desc.push_str(&format!("**{s}:** {text}\n"));
            }

            embed = embed.field("== TRAITS ==", desc, false);
        }
    }

    if !card.related.is_empty() {
        let value = format!("**Token:** {}", card.related.join(", "));
        if compact {
            desc.push_str(&value);
        } else {
            embed = embed.field("== EXTRA INFO ==", value, false);
        }
    }

    if compact {
        desc = desc.replace("\n\n", "\n");
    }

    (
        embed
            .description(desc)
            .thumbnail(format!("attachment://{}.png", hash_card_url(card))),
        if card.extra.artist.is_empty() {
            String::new()
        } else {
            format!("This card art was drawn by {}", card.extra.artist)
        },
    )
}

/// Missing embed when the card is not found
pub fn missing_embed(name: &str) -> CreateEmbed {
    CreateEmbed::new()
        .color(roles::RED)
        .title(format!("Card \"{name}\" not found"))
        .description(
            "No card found with sufficient similarity with the search term in the selected set(s).",
        )
}

fn cost_str(card: &Card) -> String {
    #[allow(clippy::inline_always)] // this is just a helper function so inline it
    #[inline(always)]
    fn append_cost(out: &mut String, count: isize, labe: &str, icon: &str) {
        #[rustfmt::skip] // it look nicer like this
        let t = format!( "**{} Cost:**{}{}{}\n", labe, icon, number::X, count.to_emoji());

        if count != 0 {
            out.push_str(&t);
        }
    }

    let mut out = String::new();

    if let Some(costs) = &card.costs {
        append_cost(&mut out, costs.blood, "Blood", imf::BLOOD);
        append_cost(&mut out, costs.bone, "Bone", imf::BONE);
        append_cost(&mut out, costs.energy, "Energy", imf::ENERGY);
        append_cost(&mut out, costs.extra.max, "Max", imf::MAX);

        if costs.mox != 0 {
            let mut mox_cost = String::from("**Mox cost:** ");
            let count = costs.mox_count.clone().unwrap_or_default();

            for m in Mox::from(costs.mox).flags() {
                match *m {
                    Mox::R => mox_cost.extend(vec![imf::RED; count.r]),
                    Mox::G => mox_cost.extend(vec![imf::GREEN; count.g]),
                    Mox::B => mox_cost.extend(vec![imf::BLUE; count.b]),
                    Mox::Y => mox_cost.extend(vec![imf::GRAY; count.y]),
                    _ => unreachable!(),
                }
            }
            out.push_str(&mox_cost);
            out.push('\n');
        }
    }

    if out.is_empty() {
        out.push_str("**Free**\n");
    }

    out
}
