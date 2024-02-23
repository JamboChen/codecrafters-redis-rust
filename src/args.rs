use std::env::args;


#[derive(Clone)]
pub struct Args {
    pub port: u16,
    pub dir: Option<String>,
    pub dbfilename: Option<String>,
    pub replicaof: Option<(String, u16)>,
}

impl Args {
    pub fn new() -> Self {
        Args {
            port: 6379,
            dir: None,
            dbfilename: None,
            replicaof: None,
        }
    }

    pub fn load() -> Self {
        let mut args_self = Args::new();
        let args: Vec<String> = args().collect();
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            match arg.to_lowercase().as_str() {
                "--port" => {
                    args_self.port = iter.next().map(|s| s.parse().unwrap()).unwrap_or(6379);
                }
                "--dir" => {
                    args_self.dir = iter.next().map(|s| s.to_owned());
                }
                "--dbfilename" => {
                    args_self.dbfilename = iter.next().map(|s| s.to_owned());
                }
                "--replicaof" => {
                    args_self.replicaof = iter.next().map(|s| {
                        (
                            s.to_owned(),
                            iter.next().map(|s| s.parse().unwrap()).unwrap_or(6379),
                        )
                    });
                }
                _ => {}
            }
        }

        args_self
    }
}
