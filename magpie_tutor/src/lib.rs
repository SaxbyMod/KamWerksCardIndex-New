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
pub mod engine;
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

use self::{
    engine::{FilterExt, MagpieCosts, MagpieExt},
    fetch::AugBranch,
};

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
pub type Card = magpie_engine::Card<MagpieExt, MagpieCosts>;
/// Set type alias.
pub type Set = magpie_engine::Set<MagpieExt, MagpieCosts>;
/// Filters type alias
pub type Filters = magpie_engine::prelude::Filters<MagpieExt, MagpieCosts, FilterExt>;

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
pub const CACHE_FILE_PATH: &str = "./cache.bin";

lazy_static! {
    /// The regex use to match for general search.
    pub static ref SEARCH_REGEX: Regex = Regex::new(r"(\S*)\[\[(.*?)\]\]") .unwrap_or_die("Cannot compiling search regex fails");
    /// The regex use to match cache attachment link.
    pub static ref CACHE_REGEX: Regex = Regex::new(r"(\d+)\/(\d+)\/(\d+)\.png\?ex=(\w+)") .unwrap_or_die("Cannot compiling cache regex fails");
    /// The regex use to match message and tokenize them
    pub static ref QUERY_REGEX: Regex = Regex::new(r#"(?:"(.+)")|([-\w]+)|([^\s\w"-]*)"#) .unwrap_or_die("Cannot compile query regex");
    /// The regex use to match cost value in query
    pub static ref COST_REGEX: Regex = Regex::new(r"(-?\d+)?([a-zA-Z])").unwrap_or_die("Cannot compile query regex");
    /// The regex use to detech if a messagae asking for a game
    pub static ref FIGHT_REGEX: Regex = Regex::new(r"wants? to (?:play|fight)").unwrap_or_die("Cannot compile asking for fight regex");

    /// Collection of all set magpie use
    pub static ref SETS: Mutex<HashMap<&'static str, Set>> = Mutex::new(load_set());

    /// Debug card use to test rendering
    pub static ref DEBUG_CARD: Card = Card {
        set: SetCode::new("des").unwrap(),
        name: "OLD_DATA".to_owned(),
        description: "If you gaze long into an abyss, the abyss also gazes into you.".to_owned(),
        portrait: "https://pbs.twimg.com/media/DUgfSnpU0AAA5Ky.jpg".to_owned(),

        rarity: Rarity::RARE,
        temple: Temple::ARTISTRY,
        tribes: Some("Big Green Mother".to_string()),

        attack: Attack::Num(420),
        health: 10,
        sigils: Vec::new(),
        costs: Some(Costs {
            blood: isize::MAX,
            bone: isize::MIN,
            energy: 100,
            mox: Mox::all(),
            mox_count: Some(MoxCount {
                o:6,
                g:9,
                b:4,
                y:2,
                k:1,
                r: 1,
                e: 1,
                p: 1,
            }),
            extra: MagpieCosts {
                shattered_count: Some(MoxCount {
                    o: 1,
                    g: 9,
                    b: 8,
                    y: 4,
                    k: 1,
                    r: 1,
                    e: 1,
                    p: 1,
                }),
                max: 451,
                link: 6,
                gold: 24601
            }
        }),
        traits: Some(Traits {
            strings: Some(["Beastly", "Trait 13", "Prisoner 24601"].into_iter().map(std::string::ToString::to_string).collect()),
            flags: TraitsFlag::all(),
        }),
        related: vec![
            "Phi".to_owned(),
            "NEW_DATA".to_owned(),
            "ANCIENT_DATA".to_owned(),
        ],
        extra: MagpieExt {
            artist: String::from("artist")
        },
    };

    /// Portrait Caches to save times on image processing
    pub static ref CACHE: Mutex<HashMap<u64, CacheData>> = load_cache();

    /// List of response that ping will return
    pub static ref PING_RESPONSE: [&'static str;16] = [
        "o jan Mike. sina toki la sina lape suli lon luka tenpo sike. mi mute li lukin e sin nasin. o pini lape",
        "ijo ijo",
        "tenpo kama",
        "mi mute li lukin toki e ni. tomo tawa sina li tenpo suli awen.",
        "suwi olin mi a. ilo jan li lon aaa",
        "mi unpa mama sini lon tenpo ni",
        "mi sona semi",
        "sin ijo lon tenpo",
        "mi olin toki pona",
        "mi olin musi Insison",
        "sina sona ala sona toki pona",
        "o sewi sewi anpa anpa poka poka teje teje A B A B",
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        "https://www.youtube.com/watch?v=b7vWLz9iGsk",
        "I don't know who you are. I don't know what you want. If you are looking for ransom I can tell you I don't have money, but what I do have are a very particular set of skills. Skills I have acquired over a very long career. Skills that make me a nightmare for people like you. If you let my daughter go now that'll be the end of it. I will not look for you, I will not pursue you, but if you don't, I will look for you, I will find you and I will kill you.",
        "Crazy?\nI was crazy once\nThey lock me in a room\nA rubber room\nA rubber room with rats\nThe rats make me crazy\nCrazy?\nI was crazy once\nThey lock me in a room\nA rubber room\nA rubber room with rats\nThe rats make me crazy\nCrazy?\nI was crazy once\nThey lock me in a room\nA rubber room\nA rubber room with rats\nThe rats make me crazy\nCrazy?\nI was crazy once\nThey lock me in a room\nA rubber room\nA rubber room with rats\nThe rats make me crazy\n",
    ];
}

fn load_set() -> HashMap<&'static str, Set> {
    set_map! {
        standard (std) => "https://raw.githubusercontent.com/107zxz/inscr-onln-ruleset/main/standard.json",
        eternal (ete) => "https://raw.githubusercontent.com/EternalHours/EternalFormat/main/IMF_Eternal.json",
        egg (egg) => "https://raw.githubusercontent.com/senor-huevo/Mr.Egg-s-Goofy/main/Mr.Egg's%20Goofy.json",
        ---
        augmented (aug) => fetch_aug_set(AugBranch::Snapshot),
        aug_main (Aug) => fetch_aug_set(AugBranch::Main),
        descryption (des) => fetch_desc_set(),
        custom_tcg (cti) => fetch_cti_set(),
    }
}

fn load_cache() -> Mutex<HashMap<u64, CacheData>> {
    let bytes = {
        let mut f = File::open(CACHE_FILE_PATH)
            .unwrap_or_else(|_| File::create_new(CACHE_FILE_PATH).unwrap());

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
        File::create(CACHE_FILE_PATH).expect("Cannot create cache file"),
        &*CACHE,
    )
    .unwrap();
    done!("Caches save successfully to {}", CACHE_FILE_PATH.green());
}

/// Hash a card url. Just a wrapper around DefaultHasher.
fn hash_card_url(card: &Card) -> u64 {
    let mut hasher = DefaultHasher::new();
    card.portrait.hash(&mut hasher);
    hasher.finish()
}

/// Resize a image from it's bytes.
fn resize_img(img: &[u8], scale: u32) -> Vec<u8> {
    if img.is_empty() {
        return Vec::new();
    }
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
    match isahc::get(url) {
        Ok(t) if t.status().is_success() => t,
        _ => {
            error!("Cannot reach url: {url}");
            return Vec::new();
        }
    }
    .bytes()
    .unwrap_or_else(|_| {
        error!("Cannot decode card portrait from url: {url}");
        Vec::new()
    })
}

/// Return the current epoch
pub fn current_epoch() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Are you Marty McFly? Return to the correct timeline")
        .as_millis()
}
