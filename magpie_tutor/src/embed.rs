use crate::emojis::{imf, number, ToEmoji};
use crate::{hash_str, Card, Set};
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
            .thumbnail(format!("attachment://{}.png", hash_str(&card.name))),
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
    let mut out = String::new();
    if let Some(costs) = &card.costs {
        if costs.blood != 0 {
            out.push_str(
                format!(
                    "**Blood Cost:**{}{}{}\n",
                    imf::BLOOD,
                    number::X,
                    costs.blood.to_emoji()
                )
                .as_str(),
            );
        }

        if costs.bone != 0 {
            out.push_str(
                format!(
                    "**Bone Cost:**{}{}{}\n",
                    imf::BONE,
                    number::X,
                    costs.bone.to_emoji()
                )
                .as_str(),
            );
        }

        if costs.energy != 0 {
            out.push_str(
                format!(
                    "**Energy Cost:**{}{}{}\n",
                    imf::ENERGY,
                    number::X,
                    costs.energy.to_emoji()
                )
                .as_str(),
            );
        }

        if costs.mox != 0 {
            let mut mox_cost = String::from("**Mox cost:** ");
            for m in Mox::from(costs.mox).flags() {
                match *m {
                    Mox::R => mox_cost.push_str(imf::RED),
                    Mox::G => mox_cost.push_str(imf::GREEN),
                    Mox::B => mox_cost.push_str(imf::BLUE),
                    Mox::Y => mox_cost.push_str(imf::GRAY),
                    _ => unreachable!(),
                }
            }
            out.push_str(&mox_cost);
            out.push('\n');
        }
    } else {
        out.push_str("**Free**\n");
    }

    out
}
