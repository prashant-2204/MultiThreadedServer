use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct CacheEntry {
    value: String,
    expires_at: u64,
}

pub struct Cache {
    store: RwLock<HashMap<String, CacheEntry>>,
    capacity: usize,
}

impl Cache {
    pub fn new(capacity: usize) -> Self {
        Cache {
            store: RwLock::new(HashMap::new()),
            capacity,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let store = self.store.read().unwrap();
        store.get(key).and_then(|entry| {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            if now < entry.expires_at {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    pub fn set(&self, key: &str, value: &str, ttl: u64) {
        let mut store = self.store.write().unwrap();
        if store.len() >= self.capacity {
            if let Some(oldest_key) = store.keys().next().cloned() {
                store.remove(&oldest_key);
            }
        }
        
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + ttl;
            
        store.insert(
            key.to_string(),
            CacheEntry {
                value: value.to_string(),
                expires_at,
            }
        );
    }
}