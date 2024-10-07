use magpie_engine::prelude::*;
use poise::serenity_prelude::{colours::roles, CreateEmbed};

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
            Temple::MAGICK => roles::RED,
            Temple::FOOL => roles::MAGENTA,
            _ if t.is_empty() => roles::LIGHT_GREY,
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
    let mut out = String::new();

    if let Some(costs) = &card.costs {
        append_cost(&mut out, costs.blood, "Blood", cost::BLOOD);
        append_cost(&mut out, costs.bone, "Bone", cost::BONE);
        append_cost(&mut out, costs.energy, "Energy", cost::ENERGY);
        append_cost(&mut out, costs.extra.max, "Max", cost::MAX);

        if !costs.mox.is_empty() {
            let mut mox_cost = String::new();
            let count = costs.mox_count.clone().unwrap_or_default();

            for m in costs.mox.iter() {
                match m {
                    Mox::O => mox_cost.extend(vec![cost::ORANGE; count.o]),
                    Mox::G => mox_cost.extend(vec![cost::GREEN; count.g]),
                    Mox::B => mox_cost.extend(vec![cost::BLUE; count.b]),
                    Mox::Y => mox_cost.extend(vec![cost::GRAY; count.y]),
                    _ => unreachable!(),
                }
            }

            if !mox_cost.is_empty() {
                out.push_str("**Mox Cost:**");
                out.push_str(&mox_cost);
                out.push('\n');
            }
        }

        if let Some(shattered) = &costs.extra.shattered_count {
            let mut mox_cost = String::from("**Shattered cost:** ");

            mox_cost.extend(vec![cost::SHATTERED_ORANGE; shattered.o]);
            mox_cost.extend(vec![cost::SHATTERED_GREEN; shattered.g]);
            mox_cost.extend(vec![cost::SHATTERED_BLUE; shattered.b]);
            mox_cost.extend(vec![cost::SHATTERED_GRAY; shattered.y]);

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
