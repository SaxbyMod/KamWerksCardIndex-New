//! Just the lib for tutor.

use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::{DefaultHasher, Hash, Hasher},
    io::Cursor,
};

use image::GenericImageView;
use isahc::ReadResponseExt;
use lazy_static::lazy_static;
use magpie_engine::prelude::*;
use poise::serenity_prelude::{CreateAllowedMentions, CreateMessage, MessageReference};
use regex::Regex;

pub mod emojis;
pub mod magpie;
pub mod query;
pub mod search;

mod data;
pub use data::*;

mod handler;
pub use handler::*;

#[macro_use]
pub mod helper;

use self::magpie::FilterExt;

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
/// Filters type alias
pub type Filters = magpie_engine::prelude::Filters<AugExt, FilterExt>;

lazy_static! {
    /// The regex use to match for general search.
    pub static ref SEARCH_REGEX: Regex = Regex::new(r"(?:([^\s{}]+?)(\w{3}(?:\|\w{3})*)?)?\{\{(.*?)\}\}") .unwrap_or_die("Cannot compiling search regex fails");
    /// The regex use to match cache attachment link.
    pub static ref CACHE_REGEX: Regex = Regex::new(r"(\d+)\/(\d+)\/(\d+)\.png\?ex=(\w+)") .unwrap_or_die("Cannot compiling cache regex fails");
    /// The regex use to match message and tokenize them
    pub static ref QUERY_REGEX: Regex = Regex::new(r#"(?:"(.+)")|([-\w]+)|([^\s\w"-]*)"#) .unwrap_or_die("Cannot compile query regex");
    /// The regex use to match cost value in query
    pub static ref COST_REGEX: Regex = Regex::new(r"(-?\d+)?([a-zA-Z])").unwrap_or_die("Cannot compile query regex");

    /// Collection of all set magpie use
    pub static ref SETS: HashMap<&'static str, Set> = {
        set_map! {
            competitve (com) => "https://raw.githubusercontent.com/107zxz/inscr-onln-ruleset/main/competitive.json",
            eternal (ete) => "https://raw.githubusercontent.com/EternalHours/EternalFormat/main/IMF_Eternal.json",
            egg (egg) => "https://raw.githubusercontent.com/senor-huevo/Mr.Egg-s-Goofy/main/Mr.Egg's%20Goofy.json"
        }
    };

    /// Debug card use to test rendering
    pub static ref DEBUG_CARD: Card = Card {
        set: SetCode::new("com").unwrap(),
        name: "OLD_DATA".to_owned(),
        description: "If you gaze long into an abyss, the abyss also gazes into you.".to_owned(),
        portrait: "https://pbs.twimg.com/media/DUgfSnpU0AAA5Ky.jpg".to_owned(),

        rarity: Rarity::RARE,
        temple: Temple::TECH.into(),
        tribes: Some("Big Green Mother".to_string()),

        attack: 0,
        health: 10,
        sigils: Vec::new(),
        sp_atk: Some(SpAtk::CARD),
        costs: Some(Costs {
            blood: isize::MAX,
            bone: isize::MIN,
            energy: 100,
            mox: Mox::all(),
            mox_count: Some(MoxCount {
                r: 6,
                g: 9,
                b: 4,
                y: 2,
            }),
        }),
        traits: Some(Traits {
            string: None,
            flags: TraitsFlag::all(),
        }),
        related: Some(vec![
            "Phi".to_owned(),
            "NEW_DATA".to_owned(),
            "ANCIENT_DATA".to_owned(),
        ]),
        extra: AugExt {
            shattered_count: Some(MoxCount {
                r: 1,
                g: 9,
                b: 8,
                y: 4,
            }),
            max: 451,
            artist: "art".to_owned(),
        },
    };
}

/// Hash a card url. Just a wrapper around DefaultHasher.
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
    /// Set this message to reply and not ping the author
    fn reply(self, reference: impl Into<MessageReference>) -> Self;
}

impl MessageCreateExt for CreateMessage {
    fn reply(self, reference: impl Into<MessageReference>) -> Self {
        self.reference_message(reference)
            .allowed_mentions(CreateAllowedMentions::new())
    }
}

/// Exrension for Option and Result where it is critical that they don't fails and if they do
/// immediately stop terminate.
pub trait Death<T> {
    /// Unwrap the data inside or terminate the program.
    fn unwrap_or_die(self, message: &str) -> T;
}

impl<T> Death<T> for Option<T> {
    fn unwrap_or_die(self, message: &str) -> T {
        if let Some(it) = self {
            return it;
        }
        error!("{}", message.red());
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
                error!("{}", message.red());
                error!("{}", format!("{err:?}").red());
                error!("{}", "Critical Error awaiting death...".red());
                std::process::exit(0)
            }
        }
    }
}

macro_rules! color_fn {
    (
        $(
            $(#[$attr:meta])*
            fn $color:ident -> $ansi:literal;
        )*
    ) => {$(
        $(#[$attr])*
        fn $color(&self) -> String
        where
            Self: Display,
        {
            format!("\x1b[0;{}m{}\x1b[0m", $ansi, self)
        }
    )*};
}

/// Allow value to be convert to a string with ansi color code.
pub trait Color {
    color_fn! {
        /// Convert value to black text.
        fn black -> 30;
        /// Convert value to red text.
        fn red -> 31;
        /// Convert value to green text.
        fn green -> 32;
        /// Convert value to yellow text.
        fn yellow -> 33;
        /// Convert value to blue text.
        fn blue -> 34;
        /// Convert value to magenta text.
        fn magenta -> 35;
        /// Convert value to cyan text.
        fn cyan -> 36;
        /// Convert value to white text.
        fn white -> 37;
    }
}

impl<T: Display> Color for T {}
impl Color for str {}
