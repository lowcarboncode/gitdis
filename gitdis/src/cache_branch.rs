use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

use log::debug;
use quickleaf::{Cache, Event};

use crate::cache::ArcCache;

#[derive(Clone)]
pub struct CacheBranch {
    pub cache: ArcCache,
    pub create_at: u128,
}

impl CacheBranch {
    pub fn new(total_cache_items: usize, sender: Sender<Event>) -> Self {
        let create_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        debug!("Creating new cache with {} items", total_cache_items);

        CacheBranch {
            cache: Arc::new(RwLock::new(Cache::with_sender(total_cache_items, sender))),
            create_at,
        }
    }

    pub fn get_data(&self) -> &ArcCache {
        &self.cache
    }

    pub fn get_create_at(&self) -> u128 {
        self.create_at
    }
}
