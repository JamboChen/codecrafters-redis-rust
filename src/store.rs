use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
struct ExpiringValue {
    value: String,
    expires_at: Option<Instant>,
}

pub struct Database {
    db: RwLock<HashMap<String, ExpiringValue>>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            db: RwLock::new(HashMap::new()),
        }
    }

    pub async fn set(&self, key: &str, value: &str) {
        let value = ExpiringValue {
            value: value.to_owned(),
            expires_at: None,
        };
        let mut db = self.db.write().await;
        db.insert(key.to_owned(), value);
    }

    pub async fn set_with_expire(&self, key: &str, value: &str, expiry_in_ms: u64) {
        let now = Instant::now();
        let duration = Duration::from_millis(expiry_in_ms);
        let value = ExpiringValue {
            value: value.to_owned(),
            expires_at: Some(now + duration),
        };
        let mut db = self.db.write().await;
        db.insert(key.to_owned(), value);
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let now = Instant::now();

        let value = {
            let db = self.db.read().await;
            db.get(key).cloned()
        };
        match value {
            Some(v) => match v.expires_at {
                Some(expires_at) if expires_at < now => {
                    println!("now: {:?}, expires_at: {:?}", now, expires_at);
                    let mut db = self.db.write().await;
                    db.remove(key);
                    None
                }
                _ => Some(v.value),
            },
            None => None,
        }
    }
}
