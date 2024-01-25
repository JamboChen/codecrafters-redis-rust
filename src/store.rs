use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct Database {
    db: RwLock<HashMap<String, String>>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            db: RwLock::new(HashMap::new()),
        }
    }

    pub async fn set(&self, key: &str, value: &str) {
        let mut db = self.db.write().await;
        db.insert(key.to_owned(), value.to_owned());
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let db = self.db.read().await;
        db.get(key).cloned()
    }
}
