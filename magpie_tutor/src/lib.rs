use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Cursor;

use image::GenericImageView;
use isahc::ReadResponseExt;
use magpie_engine::prelude::AugExt;
use poise::serenity_prelude::{
    CreateAllowedMentions, CreateMessage, MessageReference, UserUpdateEvent,
};

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

/// Hash a str to u64. Just a wrapper around DefaultHasher
fn hash_str(name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
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

pub trait UnsignExt {
    fn for_each<F>(self, f: F)
    where
        F: FnMut();
}

impl UnsignExt for usize {
    fn for_each<F>(self, mut f: F)
    where
        F: FnMut(),
    {
        for _ in 0..self {
            f();
        }
    }
}
