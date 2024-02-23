use std::collections::HashMap;
use std::env::args;

#[derive(Debug)]
pub struct Args {
    args: HashMap<String, String>,
    flags: Vec<String>,
}

impl Args {
    pub fn new() -> Self {
        Args {
            args: HashMap::new(),
            flags: Vec::new(),
        }
    }

    pub fn load(&mut self) {
        let args: Vec<String> = args().collect();
        let mut iter = args.iter().peekable();
        while let Some(arg) = iter.next() {
            if !arg.starts_with("--") {
                continue;
            }
            let key = arg[2..].to_string();
            let value = if iter.peek().map(|s| s.starts_with("--")).unwrap_or(false) {
                None
            } else {
                iter.next().map(|s| s.to_owned())
            };

            if let Some(value) = value {
                self.args.insert(key, value);
            } else {
                self.flags.push(key);
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.args.get(key).map(|s| s.to_owned())
    }

    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains(&flag.to_string())
    }
}
