#![allow(unused)] // shush im fixing them

use magpie_engine::prelude::*;
use poise::serenity_prelude::{colours::roles, CreateEmbed};

use crate::emojis::{cost, ToEmoji};
use crate::{Card, Set};

use super::{append_cost, EmbedRes};

pub fn gen_embed(card: &Card, set: &Set, compact: bool) -> EmbedRes {
    todo!()
}
