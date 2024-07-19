use std::iter;

use crate::emojis::{imf, number, ToEmoji};
use crate::{hash_card_url, Card, Set};
use magpie_engine::prelude::*;
use poise::serenity_prelude::CreateEmbedFooter;
use poise::serenity_prelude::{colours::roles, CreateEmbed};

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
pub fn gen_embed(rank: f32, card: &Card, set: &Set) -> CreateEmbed {
    // The specific gen embed function should return the embed and the footer that they would like
    // to add.

    let (embed, footer) = match card.set.code() {
        "aug" => gen_aug_embed(card, set),
        "egg" | "ete" | "com" | "old" => gen_imf_embed(card, set),
        _ => unimplemented!(),
    };
    embed.footer(CreateEmbedFooter::new(format!(
        "{footer}\nMatch {:.2}% with the search term",
        rank * 100.
    )))
}

fn gen_imf_embed(card: &Card, set: &Set) -> EmbedRes {
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

    let mut desc = if card.description.is_empty() {
        String::new()
    } else {
        format!("*{}*\n\n", card.description)
    };

    desc.push_str(&cost_str(card)); // the card cost
    desc.push('\n'); // stat separator

    desc.push_str(&format!(
        "**Stat:** {} / {}",
        match &card.sp_atk {
            Some(sp) => sp.to_emoji(),
            None => card.attack.to_string(),
        },
        card.health
    ));

    if !card.sigils.is_empty() {
        let mut desc = String::with_capacity(card.sigils.iter().map(|s| s.len()).sum());

        for s in &card.sigils {
            let text = set.sigils_description.get(s).unwrap();
            desc.push_str(&format!("**{s}:** {text}\n"));
        }

        embed = embed.field("== SIGILS ==", desc, false);
    }

    (
        embed
            .description(desc)
            .thumbnail(format!("attachment://{}.png", hash_card_url(card))),
        String::new(), // empty footer
    )
}

fn gen_aug_embed(_: &Card, _: &Set) -> EmbedRes {
    (CreateEmbed::new(), String::new())
}

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

        if costs.mox != 0 {
            let mut mox_cost = String::from("**Mox cost:** ");
            let count = costs.mox_count.clone().unwrap_or_default();

            for m in Mox::from(costs.mox).flags() {
                match *m {
                    Mox::R => mox_cost.extend(iter::repeat(imf::RED).take(count.r)),
                    Mox::G => mox_cost.extend(iter::repeat(imf::GREEN).take(count.g)),
                    Mox::B => mox_cost.extend(iter::repeat(imf::BLUE).take(count.b)),
                    Mox::Y => mox_cost.extend(iter::repeat(imf::GRAY).take(count.y)),
                    _ => unreachable!(),
                }
            }
            out.push_str(&mox_cost);
            out.push('\n');
        }
    }

    append_cost(&mut out, card.extra.max, "Max", imf::MAX);

    if out.is_empty() {
        out.push_str("**Free**\n");
    }

    out
}
