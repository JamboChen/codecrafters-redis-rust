use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

use crate::config::Config;
use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Clone)]
struct ExpiringValue {
    value: String,
    expires_at: Option<SystemTime>,
}

pub struct Database {
    db: Mutex<HashMap<String, ExpiringValue>>,
    config: Config,
    replications: Mutex<Vec<UnboundedSender<String>>>,
    wait: Mutex<usize>,
}

impl Database {
    pub fn new() -> Self {
        let config = Config::from_args();

        let db = match config.get_file_path() {
            Some(file_path) => {
                if let Some(file) = File::open(file_path).ok() {
                    println!("reading from file");
                    serialize(file)
                } else {
                    HashMap::new()
                }
            }
            None => HashMap::new(),
        };

        Database {
            db: Mutex::new(db),
            config,
            replications: Mutex::new(Vec::new()),
            wait: Mutex::new(0),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn set(&self, key: &str, value: &str) {
        let value = ExpiringValue {
            value: value.to_owned(),
            expires_at: None,
        };
        let mut db = self.db.lock().await;
        db.insert(key.to_owned(), value);
    }

    pub async fn set_with_expire(&self, key: &str, value: &str, expiry_in_ms: u64) {
        let now = SystemTime::now();
        let duration = Duration::from_millis(expiry_in_ms);
        let value = ExpiringValue {
            value: value.to_owned(),
            expires_at: Some(now + duration),
        };
        let mut db = self.db.lock().await;
        db.insert(key.to_owned(), value);
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let now = SystemTime::now();

        let value = {
            let db = self.db.lock().await;
            db.get(key).cloned()
        };
        match value {
            Some(v) => match v.expires_at {
                Some(expires_at) if expires_at < now => {
                    println!("now: {:?}, expires_at: {:?}", now, expires_at);
                    let mut db = self.db.lock().await;
                    db.remove(key);
                    None
                }
                _ => Some(v.value),
            },
            None => None,
        }
    }

    pub async fn keys(&self, _pattern: &str) -> Vec<String> {
        let now = SystemTime::now();
        let mut expired_keys = Vec::new();
        let mut valid_keys = Vec::new();

        {
            let db = self.db.lock().await;
            for (key, value) in db.iter() {
                match value.expires_at {
                    Some(expires_at) if expires_at < now => {
                        expired_keys.push(key.to_owned());
                    }
                    _ => {
                        valid_keys.push(key.to_owned());
                    }
                }
            }
        }

        {
            let mut db = self.db.lock().await;
            for key in expired_keys {
                db.remove(&key);
            }
        }

        valid_keys
    }

    pub fn config_get(&self, key: &str) -> Option<String> {
        self.config.get_info(key)
    }

    pub async fn add_replication(&self, tx: UnboundedSender<String>) {
        let mut replications = self.replications.lock().await;
        replications.push(tx);
    }

    pub async fn spread(&self, cmd: &[u8]) {
        let replications = self.replications.lock().await;
        for tx in replications.iter() {
            let _ = tx.send(String::from_utf8_lossy(cmd).to_string());
            {
                let mut wait = self.wait.lock().await;
                *wait += 1;
            }
        }
        let mut wait = self.wait.lock().await;
        *wait = 0;
    }

    pub async fn replication_count(&self) -> usize {
        let replications = self.replications.lock().await;
        replications.len()
    }

    pub async fn wait(&self, count: usize, timeout: u64) -> bool {
        let mut wait = self.wait.lock().await;
        if *wait >= count {
            *wait = 0;
            true
        } else {
            false
        }
    }
}

fn length_encode(buf: &[u8]) -> Option<(usize, usize)> {
    let mask = 3u8 << 6; // 1100 0000
    let num = match buf[0] & mask {
        0 => (u32::from_be_bytes([0, 0, 0, buf[0]]), 1),
        64u8 => (u32::from_be_bytes([0, 0, buf[0] & (64u8 - 1), buf[1]]), 2),
        128u8 => (u32::from_be_bytes(buf[1..5].try_into().unwrap()), 5),
        192u8 => return None,
        _ => unreachable!(),
    };
    let num = (num.0 as usize, num.1);
    Some(num)
}

fn serialize_kv(buf: &[u8]) -> Option<(String, ExpiringValue, usize)> {
    let is_expired = buf[0] == 0xfc;
    let expires_at = if is_expired {
        let expires_at = u64::from_le_bytes(buf[1..9].try_into().unwrap());
        Some(UNIX_EPOCH + Duration::from_millis(expires_at as u64))
    } else {
        None
    };
    let mut pos = if is_expired { 10 } else { 1 };

    let (key_len, offset) = length_encode(&buf[pos..]).unwrap();
    pos += offset;
    let key = String::from_utf8(buf[pos..pos + key_len].to_vec()).unwrap();
    pos += key_len;

    let (value_len, offset) = length_encode(&buf[pos..]).unwrap();
    pos += offset;
    let value = String::from_utf8(buf[pos..pos + value_len].to_vec()).unwrap();

    let value = ExpiringValue {
        value,
        expires_at: expires_at,
    };
    Some((key, value, pos + value_len))
}

fn serialize(file: File) -> HashMap<String, ExpiringValue> {
    let now = SystemTime::now();
    println!("now: {:?}", now);
    let mut reader = BufReader::new(file);
    let mut buf = [0u8; 1024];
    let _bytes_read = reader.read(&mut buf).unwrap();

    let fb_pos = buf.iter().position(|&b| b == 0xfb).unwrap();
    let mut pos = fb_pos + 1;
    let (hashtable_size, offset) = length_encode(&buf[pos..]).unwrap();
    pos += offset;
    let (_exprie_hashtable_size, offset) = length_encode(&buf[pos..]).unwrap();
    pos += offset;

    let mut db = HashMap::new();
    for _ in 0..hashtable_size {
        let (key, value, offset) = serialize_kv(&buf[pos..]).unwrap();
        match value.expires_at {
            Some(expires_at) if expires_at < now => {
                println!("key: {}, expires_at: {:?}", key, expires_at);
            }
            _ => {
                println!("key: {}, expires_at: {:?}", key, value.expires_at);
                db.insert(key, value);
            }
        }
        pos += offset;
    }

    db
}
