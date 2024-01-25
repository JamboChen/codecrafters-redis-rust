use std::env::args;

#[derive(Debug)]
pub struct Config {
    dir: Option<String>,
    dbfilename: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            dir: None,
            dbfilename: None,
        }
    }

    pub fn from_args(&mut self) {
        let args: Vec<String> = args().collect();
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            match arg.to_lowercase().as_str() {
                "--dir" => {
                    self.dir = iter.next().map(|s| s.to_owned());
                }
                "--dbfilename" => {
                    self.dbfilename = iter.next().map(|s| s.to_owned());
                }
                _ => {}
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match key.to_lowercase().as_str() {
            "dir" => self.dir.clone(),
            "dbfilename" => self.dbfilename.clone(),
            _ => None,
        }
    }
}
