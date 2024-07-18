use std::io::Read;
use std::{collections::HashMap, fs::File, sync::Mutex};

use bincode::deserialize;
use magpie_engine::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{hashmap, Card, Set};

pub type Cache = HashMap<u64, CacheData>;

/// The caches data
#[derive(Serialize, Deserialize)]
pub struct CacheData {
    pub channel_id: u64,
    pub attachment_id: u64,
    pub expire_date: u64,
}

pub const QUERY_REGEX: &str = r"(?:(.*?)(\w{3}(?:\|\w{3})*))?\[(.*?)\]";
pub const CACHE_REGEX: &str = r"(\d+)\/(\d+)\/(\d+)\.png\?ex=(\w+)";

/// Custom data carry between function.
pub struct Data {
    pub query_regex: Regex,
    pub cache_regex: Regex,
    pub sets: HashMap<String, Set>,
    pub debug_card: Card,
    pub portrait_cache: Mutex<HashMap<u64, CacheData>>,
}

impl Data {
    pub fn new() -> Self {
        Data {
            query_regex: Regex::new(r"(?:(.*?)(\w{3}(?:\|\w{3})*))?\[(.*?)\]")
                .expect("Compiling query regex fails"),
            cache_regex: Regex::new(r"(\d+)\/(\d+)\/(\d+)\.png\?ex=(\w+)")
                .expect("Compiling cache regex fails"),
            sets: setup_set(),
            debug_card: debug_card(),
            portrait_cache: Self::load_cache(),
        }
    }

    pub fn insert_cache(&self, key: u64, data: CacheData) {
        self.portrait_cache.lock().unwrap().insert(key, data);
    }

    pub fn remove_cache(&self, key: u64) {
        self.portrait_cache.lock().unwrap().remove(&key);
    }

    pub fn save_cache(&self) {
        bincode::serialize_into(
            File::create("test.bin").expect("Cannot create cache file"),
            &self.portrait_cache,
        )
        .unwrap();
    }

    fn load_cache() -> Mutex<Cache> {
        let bytes = {
            let mut f =
                File::open("test.bin").unwrap_or_else(|_| File::create_new("test.bin").unwrap());

            // fails on 32 bits system in which case we will reserve less but it not critical so we
            // will just allow it
            #[allow(clippy::cast_possible_truncation)]
            let mut buf = vec![
                0;
                f.metadata()
                    .expect("Unable to get cache file metadata")
                    .len() as usize
            ];

            f.read_exact(&mut buf).expect("Buffer overflow");

            buf
        };
        deserialize(&bytes[..]).unwrap()
    }
}

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}

/// set up all the set for magpie
fn setup_set() -> HashMap<String, Set> {
    let competitive = fetch_imf_set(
        "https://raw.githubusercontent.com/107zxz/inscr-onln-ruleset/main/competitive.json",
        SetCode::new("com").unwrap(),
    )
    .expect("Cannot process Competitive's set")
    .upgrade();

    hashmap! {
        "com".to_owned() => competitive
    }
}

/// The default Debug card
fn debug_card() -> Card {
    Card {
        set: SetCode::new("com").unwrap(),
        name: "OLD_DATA".to_owned(),
        description: "If you gaze long into an abyss, the abyss also gazes into you.".to_owned(),
        portrait: "https://pbs.twimg.com/media/DUgfSnpU0AAA5Ky.jpg".to_owned(),
        rarity: Rarity::RARE,
        temple: Temple::TECH.into(),
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
                r: -6,
                g: -9,
                b: 4,
                y: 2,
            }),
        }),
        traits: Some(Traits {
            traits: None,
            flags: TraitsFlag::all(),
        }),
        related: Some(vec![]),
        extra: AugExt {
            shattered_count: Some(MoxCount {
                r: 1,
                g: 9,
                b: 8,
                y: 4,
            }),
            max: 451,
            tribes: "Big Green Mother".to_owned(),
            artist: "art".to_owned(),
        },
    }
}
