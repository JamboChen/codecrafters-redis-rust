use std::env::args;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub dir: Option<String>,
    pub dbfilename: Option<String>,
    pub replicaof: Option<(String, u16)>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            port: 6379,
            dir: None,
            dbfilename: None,
            replicaof: None,
        }
    }

    pub fn from_args() -> Self {
        let mut config = Config::new();
        let args: Vec<String> = args().collect();
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            match arg.to_lowercase().as_str() {
                "--port" => {
                    config.port = iter.next().map(|s| s.parse().unwrap()).unwrap_or(6379);
                }
                "--dir" => {
                    config.dir = iter.next().map(|s| s.to_owned());
                }
                "--dbfilename" => {
                    config.dbfilename = iter.next().map(|s| s.to_owned());
                }
                "--replicaof" => {
                    config.replicaof = iter.next().map(|s| {
                        (
                            s.to_owned(),
                            iter.next().map(|s| s.parse().unwrap()).unwrap_or(6379),
                        )
                    });
                }
                _ => {}
            }
        }

        config
    }

    pub fn get_file_path(&self) -> Option<String> {
        match (&self.dir, &self.dbfilename) {
            (Some(dir), Some(dbfilename)) => Some(format!("{}/{}", dir, dbfilename)),
            _ => None,
        }
    }

    pub fn get_info(&self, key: &str) -> Option<String> {
        match key {
            "dir" => self.dir.clone(),
            "dbfilename" => self.dbfilename.clone(),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "dir" => self.dir.clone(),
            "dbfilename" => self.dbfilename.clone(),
            "port" => Some(self.port.to_string()),
            "replicaof" => match &self.replicaof {
                Some((host, port)) => Some(format!("{}:{}", host, port)),
                None => None,
            },
            _ => None,
        }
    }
}
