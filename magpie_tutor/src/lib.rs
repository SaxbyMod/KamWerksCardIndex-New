//! Just the lib for tutor.

use std::{
    collections::HashMap,
    fmt::Debug,
    fs::File,
    hash::{DefaultHasher, Hash, Hasher},
    io::{Cursor, Read},
    sync::Mutex,
};

use image::GenericImageView;
use isahc::ReadResponseExt;
use lazy_static::lazy_static;
use magpie_engine::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub mod emojis;
pub mod magpie;
pub mod query;
pub mod search;

mod message;
pub use message::*;

mod handler;
pub use handler::*;

mod traits;
pub use traits::*;

mod fuzzy;
pub use fuzzy::*;

#[macro_use]
pub mod r#macro;

use self::magpie::FilterExt;

// Type definition for stuff

/// Custom data carry between commands.
pub struct Data {}

impl Data {
    /// Make a new instance of [`Data`]
    pub fn new() -> Self {
        Data {}
    }
}

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}

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

/// Type alias for caches
pub type Cache = HashMap<u64, CacheData>;

/// The caches data.
#[derive(Serialize, Deserialize, Debug)]
pub struct CacheData {
    /// The channel id of the portrait cache.
    pub channel_id: u64,
    /// The attachment id of the potrait cache.
    pub attachment_id: u64,
    /// The expire date of the portrait cache.
    pub expire_date: u64,
}

/// Location of the cache file.
pub const CACHE_FILE: &str = "./cache.bin";

lazy_static! {
    /// The regex use to match for general search.
    pub static ref SEARCH_REGEX: Regex = Regex::new(r"(\S*)\[\[(.*?)\]\]") .unwrap_or_die("Cannot compiling search regex fails");
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
            egg (egg) => "https://raw.githubusercontent.com/senor-huevo/Mr.Egg-s-Goofy/main/Mr.Egg's%20Goofy.json",
            ---
            augmented (aug) => fetch_aug_set,
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
            strings: None,
            flags: TraitsFlag::all(),
        }),
        related: vec![
            "Phi".to_owned(),
            "NEW_DATA".to_owned(),
            "ANCIENT_DATA".to_owned(),
        ],
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

    /// Portrait Caches to save times on image processing
    pub static ref CACHE: Mutex<HashMap<u64, CacheData>> = load_cache();
}

fn load_cache() -> Mutex<HashMap<u64, CacheData>> {
    let bytes = {
        let mut f =
            File::open(CACHE_FILE).unwrap_or_else(|_| File::create_new(CACHE_FILE).unwrap());

        let mut buf = vec![
            0;
            f.metadata()
                .expect("Unable to get cache file metadata")
                .len()
                .try_into()
                .expect("File len data been truncated")
        ];

        f.read_exact(&mut buf).expect("Buffer overflow");

        buf
    };

    if bytes.is_empty() {
        return Mutex::new(HashMap::new());
    }

    let t: Mutex<Cache> = bincode::deserialize(&bytes).unwrap();
    t
}

/// Save the cache to the cache file.
pub fn save_cache() {
    bincode::serialize_into(
        File::create(CACHE_FILE).expect("Cannot create cache file"),
        &*CACHE,
    )
    .unwrap();
    done!("Caches save successfully to {}", CACHE_FILE.green());
}

/// Hash a card url. Just a wrapper around DefaultHasher.
fn hash_card_url(card: &Card) -> u64 {
    let mut hasher = DefaultHasher::new();
    card.portrait.hash(&mut hasher);
    hasher.finish()
}

/// Resize a image from it's bytes.
fn resize_img(img: &[u8], scale: u32) -> Vec<u8> {
    let t = image::load_from_memory(img).expect("Decode image fails");
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

/// Return the current epoch
pub fn current_epoch() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Are you Marty McFly? Return to the correct timeline")
        .as_millis()
}
