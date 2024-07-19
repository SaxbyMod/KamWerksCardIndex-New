use core::panic;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Cursor;

use image::GenericImageView;
use isahc::ReadResponseExt;
use magpie_engine::prelude::AugExt;
use poise::serenity_prelude::{CreateAllowedMentions, CreateMessage, MessageReference};

pub mod embed;
pub mod emojis;
pub mod fuzzy;
pub mod helper;
pub mod magpie;
pub mod query;

mod data;
pub use data::*;

// Type definition for stuff

/// Discord bot error type alias.
pub type Error = Box<dyn std::error::Error + Send + Sync>;
/// Poise context type alias.
pub type Context<'a> = poise::Context<'a, Data, Error>;

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
        println!("\x1b[1;31m{message}\x1b[0m");
        println!("Critical Error awaiting death...");
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
                println!("{message}");
                println!("{err:?}");
                println!("Critical Error awaiting death...");
                std::process::exit(0)
            }
        }
    }
}
