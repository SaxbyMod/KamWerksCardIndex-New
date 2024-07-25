use std::{collections::HashMap, fs::File, io::Read, sync::Mutex};

use serde::{Deserialize, Serialize};

use crate::{done, info, Color};

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
    /// Cache for portrait.
    pub cache: Mutex<HashMap<u64, CacheData>>,
}

impl Data {
    /// Create a new instant of data
    pub fn new() -> Self {
        Data {
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
