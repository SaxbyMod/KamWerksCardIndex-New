use magpie_engine::prelude::*;
use poise::serenity_prelude::{colours::roles, CreateEmbed};

use crate::{
    emojis::{cost, ToEmoji},
    Card, Set,
};

use super::{append_cost, EmbedRes};

pub fn gen_embed(card: &Card, set: &Set, compact: bool) -> EmbedRes {
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
                Some(tr) => tr.flags.to_emoji(),
                None => String::new(),
            }
        ));

    let mut desc = if card.description.is_empty() || compact {
        String::new()
    } else {
        format!("*{}*\n\n", card.description)
    };

    let mut out = String::new();

    if let Some(costs) = &card.costs {
        append_cost(&mut out, costs.blood, "Blood", cost::BLOOD);
        append_cost(&mut out, costs.bone, "Bone", cost::BONE);
        append_cost(&mut out, costs.energy, "Energy", cost::ENERGY);
        append_cost(&mut out, costs.extra.max, "Max", cost::MAX);

        if !costs.mox.is_empty() {
            let mut mox_cost = String::from("**Mox cost:** ");
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
            out.push_str(&mox_cost);
            out.push('\n');
        }
    }

    if out.is_empty() {
        out.push_str("**Free**\n");
    }

    desc.push_str(&out); // the card cost
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
        embed.description(desc),
        String::new(), // empty footer
    )
}
