use core::panic;
use std::fmt::{Debug, Display};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Cursor;

use image::GenericImageView;
use isahc::ReadResponseExt;
use magpie_engine::prelude::AugExt;
use poise::serenity_prelude::{CreateAllowedMentions, CreateMessage, MessageReference};

pub mod embed;
pub mod emojis;
pub mod fuzzy;
#[macro_use]
pub mod helper;
pub mod magpie;
pub mod query;

mod data;
pub use data::*;

// Type definition for stuff

/// Discord bot error type alias.
pub type Error = Box<dyn std::error::Error + Send + Sync>;
/// Poise context type alias.
pub type CmdCtx<'a> = poise::Context<'a, Data, Error>;

/// Discord bot function return type.
pub type Res = Result<(), Error>;

/// Card type alias.
pub type Card = magpie_engine::Card<AugExt>;
/// Set type alias.
pub type Set = magpie_engine::Set<AugExt>;

/// Hash a card url. Just a wrapper around DefaultHasher
fn hash_card_url(card: &Card) -> u64 {
    let mut hasher = DefaultHasher::new();
    card.portrait.hash(&mut hasher);
    hasher.finish()
}

/// Resize a image from it's bytes.
fn resize_img(img: Vec<u8>, scale: u32) -> Vec<u8> {
    let t = image::io::Reader::new(Cursor::new(img))
        .with_guessed_format()
        .expect("Cursor IO fails")
        .decode()
        .expect("Decode image fails");
    let (w, h) = t.dimensions();
    let mut out = vec![];
    t.resize_exact(w * scale, h * scale, image::imageops::Nearest)
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("Resize fails");
    out
}

/// Generate card embed from a card data.
pub fn get_portrait(url: &str) -> Vec<u8> {
    isahc::get(url)
        .unwrap_or_else(|_| panic!("Cannot reach url: {url}"))
        .bytes()
        .unwrap_or_else(|_| panic!("Cannot decode image to byte from url: {url}"))
}

/// Custom message extension
pub trait MessageCreateExt
where
    Self: Sized,
{
    fn reply(self, reference: impl Into<MessageReference>) -> Self;
}

impl MessageCreateExt for CreateMessage {
    fn reply(self, reference: impl Into<MessageReference>) -> Self {
        self.reference_message(reference)
            .allowed_mentions(CreateAllowedMentions::new())
    }
}

/// Exrension for Option and Result where it is critical that they don't fails and if they do
/// immedietly stop terminate.
pub trait Death<T> {
    fn unwrap_or_die(self, message: &str) -> T;
}

impl<T> Death<T> for Option<T> {
    fn unwrap_or_die(self, message: &str) -> T {
        if let Some(it) = self {
            return it;
        }
        error!("{message}");
        error!("Critical Error awaiting death...");
        std::process::exit(0)
    }
}

impl<T, E> Death<T> for Result<T, E>
where
    E: Debug,
{
    fn unwrap_or_die(self, message: &str) -> T {
        match self {
            Ok(it) => it,
            Err(err) => {
                error!("{message}");
                error!("{err:?}");
                error!("Critical Error awaiting death...");
                std::process::exit(0)
            }
        }
    }
}

macro_rules! color_fn {
    ($color:ident, $ansi:literal) => {
        fn $color(&self) -> String
        where
            Self: Display,
        {
            format!("\x1b[{}m{}\x1b[0m", $ansi, self)
        }
    };
}

pub trait Color {
    color_fn!(black, "1;30");
    color_fn!(red, "1;31");
    color_fn!(green, "1;32");
    color_fn!(yellow, "1;33");
    color_fn!(blue, "1;34");
    color_fn!(magenta, "1;35");
    color_fn!(cyan, "1;36");
    color_fn!(white, "1;37");
}

macro_rules! impl_color {
    ($($type:ty)*) => {
        $(impl Color for $type {})*
    };
}

impl_color!(
    String str
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
);
