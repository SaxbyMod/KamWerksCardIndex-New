#![allow(unused)] // shush im fixing them

use magpie_engine::prelude::*;
use poise::serenity_prelude::{colours::roles, Colour, CreateEmbed};

use crate::{
    emojis::{cost, ToEmoji},
    hash_card_url, Card, Set,
};

use super::{append_cost, EmbedRes};

pub fn gen_embed(card: &Card, set: &Set, compact: bool) -> EmbedRes {
    let color = if let Some(t) = card.temple.iter().next() {
        match t {
            Temple::BEAST => roles::DARK_GOLD,
            Temple::UNDEAD => roles::GREEN,
            Temple::TECH => roles::BLUE,
            Temple::MAGICK => roles::MAGENTA,
            Temple::ARTISTRY => Colour::new(u32::from_str_radix("3c3f4a", 16).unwrap()),
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
            Some(tr) => tr.flags.to_emoji(),
            None => String::new(),
        }
    ));

    let mut desc = if card.description.is_empty() || compact {
        String::new()
    } else {
        format!("*{}*\n\n", card.description)
    };

    desc.push_str(&format!("**Rarity:** {}\n", &card.rarity.to_string()));
    if let Some(t) = &card.tribes {
        desc.push_str(&format!("**Tribes:** {t}\n"));
    }

    desc.push('\n'); // cost separator
    let mut out = String::new();

    if let Some(costs) = &card.costs {
        append_cost(&mut out, costs.blood, "Blood", cost::BLOOD);
        append_cost(&mut out, costs.bone, "Bone", cost::BONE);
        append_cost(&mut out, costs.energy, "Energy", cost::ENERGY);
        append_cost(&mut out, costs.extra.link, "Link", cost::LINK);
        append_cost(&mut out, costs.extra.gold, "Gold", cost::GOLD);

        if !costs.mox.is_empty() {
            let mut mox_cost = String::from("**Mox cost:** ");

            for m in costs.mox.iter() {
                match m {
                    Mox::O => mox_cost.push_str(cost::ORANGE),
                    Mox::G => mox_cost.push_str(cost::GREEN),
                    Mox::B => mox_cost.push_str(cost::BLUE),
                    Mox::K => mox_cost.push_str(cost::BLACK),
                    Mox::P => mox_cost.push_str(cost::PLUS1),
                    Mox::Y => mox_cost.push_str(cost::GRAY),
                    _ => todo!(),
                }
            }
            out.push_str(&mox_cost);
            out.push('\n');
        }
    }

    if out.is_empty() {
        out.push_str("**Free**\n");
    }

    desc.push_str(&out); // the card cost
    desc.push('\n'); // stat separator

    // remove this once we actually parse aug spatk
    #[allow(clippy::match_wildcard_for_single_variants)]
    desc.push_str(&format!(
        "**Stat:** {} / {}",
        match &card.attack {
            Attack::Num(a) => a.to_string(),
            Attack::Str(s) => s.to_owned(),
            _ => unimplemented!(),
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
        embed = embed.field(
            "== TRAITS ==",
            format!("**Traits:** {}", t.join(", ")),
            false,
        );
    }

    if compact {
        desc = desc.replace("\n\n", "\n");
    }

    (
        embed
            .description(desc)
            .thumbnail(format!("attachment://{}.png", hash_card_url(card))),
        String::new(),
    )
}
