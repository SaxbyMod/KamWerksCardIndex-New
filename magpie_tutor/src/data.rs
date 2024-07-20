use std::io::Read;
use std::{collections::HashMap, fs::File, sync::Mutex};

use magpie_engine::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{hashmap, set_map, Card, Color, Death, Set};

const CACHE_FILE: &str = "./cache.bin";

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

/// Custom data carry between function.
pub struct Data {
    /// The regex use to match for query.
    pub query_regex: Regex,
    /// The regex use to match cache attachment link.
    pub cache_regex: Regex,
    /// Collect of set that magpie support.
    pub sets: HashMap<String, Set>,
    /// Debug card use for test rendering.
    pub debug_card: Card,
    /// Cache for portrait.
    pub cache: Mutex<HashMap<u64, CacheData>>,
}

impl Data {
    /// Create a new instant of data
    pub fn new() -> Self {
        Data {
            query_regex: Regex::new(r"(?:([^\s{}]+?)(\w{3}(?:\|\w{3})*)?)?\{\{(.*?)\}\}")
                .expect("Compiling query regex fails"),
            cache_regex: Regex::new(r"(\d+)\/(\d+)\/(\d+)\.png\?ex=(\w+)")
                .expect("Compiling cache regex fails"),
            sets: setup_set(),
            debug_card: debug_card(),
            cache: Self::load_cache(),
        }
    }

    /// Save the cache to a file
    pub fn save_cache(&self) {
        bincode::serialize_into(
            File::create(CACHE_FILE).expect("Cannot create cache file"),
            &self.cache,
        )
        .unwrap();
        done!("Caches save successfully to {}", CACHE_FILE.green());
    }

    fn load_cache() -> Mutex<Cache> {
        info!("Loading caches from {}...", CACHE_FILE.green());
        let now = std::time::Instant::now();
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

        done!(
            "Loaded {} caches in {}",
            t.lock().unwrap().len().green(),
            format!("{:.2?}", now.elapsed()).green()
        );

        t
    }
}

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}

/// set up all the set for magpie.
fn setup_set() -> HashMap<String, Set> {
    info!("Fetching set...");
    let sets = set_map! {
        competitve (com) => "https://raw.githubusercontent.com/107zxz/inscr-onln-ruleset/main/competitive.json",
        eternal (ete) => "https://raw.githubusercontent.com/EternalHours/EternalFormat/main/IMF_Eternal.json",
        egg (egg) => "https://raw.githubusercontent.com/senor-huevo/Mr.Egg-s-Goofy/main/Mr.Egg's%20Goofy.json"
    };
    done!("Finish fetching {} sets", sets.len().green());
    sets
}

/// The default Debug card.
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
                r: 6,
                g: 9,
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
