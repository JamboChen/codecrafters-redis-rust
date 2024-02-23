mod args;
mod parse;
mod store;

use parse::parse_command;
use std::io::Error;
use std::sync::Arc;
use store::Database;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    spawn,
};

pub enum Command {
    Ping,
    Echo(String),
    Set(String, String, Option<u64>),
    Get(String),
    Keys(String),
    ConfigGet(String),
    Unknown,
}

async fn execute_command(
    stream: &mut TcpStream,
    command: Command,
    db: &Database,
) -> Result<(), Error> {
    let resp: String = match command {
        Command::Ping => "+PONG\r\n".to_string(),
        Command::Echo(echo_arg) => {
            format!("+{}\r\n", echo_arg)
        }
        Command::Set(key, value, expiry_in_ms) => match expiry_in_ms {
            Some(expiry_in_ms) => {
                db.set_with_expire(&key, &value, expiry_in_ms).await;
                "+OK\r\n".to_string()
            }
            None => {
                db.set(&key, &value).await;
                "+OK\r\n".to_string()
            }
        },
        Command::Get(key) => match db.get(&key).await {
            Some(value) => {
                format!("+{}\r\n", value)
            }
            None => "$-1\r\n".to_string(),
        },
        Command::Keys(pattern) => {
            let mut keys = db.keys(&pattern).await;
            keys.sort();
            let mut resp = String::new();
            resp.push_str(&format!("*{}\r\n", keys.len()));
            for key in keys {
                resp.push_str(&format!("${}\r\n{}\r\n", key.len(), key));
            }
            resp
        }
        Command::ConfigGet(key) => match db.config_get(key.as_str()).await {
            Some(value) => {
                format!(
                    "*2\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
                    key.len(),
                    key,
                    value.len(),
                    value
                )
            }
            None => "$-1\r\n".to_string(),
        },
        Command::Unknown => "-ERR unknown command\r\n".to_string(),
    };

    stream.write_all(resp.as_bytes()).await?;
    Ok(())
}

async fn handle_stream(stream: TcpStream, db: &Database) -> Result<(), Error> {
    let mut stream = stream;
    let mut buf = [0; 1024];
    while let Ok(n) = stream.read(&mut buf).await {
        if n == 0 {
            break;
        }

        match parse_command(&buf[..n]).await {
            Ok(cmd) => execute_command(&mut stream, cmd, db).await?,

            Err(e) => {
                println!("error: {}", e);
                break;
            }
        }
    }
    Ok(())
}

const DEFAULT_PORT: &str = "6379";

#[tokio::main]
async fn main() {
    let mut args = args::Args::new();
    args.load();

    let db_dir = args.get("dir");
    let db_name = args.get("dbfilename");
    let port = args.get("port").unwrap_or(DEFAULT_PORT.to_string());

    let db = Database::new(db_dir, db_name);
    let db = Arc::new(db);

    let address = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(address).await.expect("failed to bind");

    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((_stream, _)) => {
                println!("accepted new connection");
                let db = Arc::clone(&db); // Move this line outside of the loop
                spawn(async move {
                    if let Err(e) = handle_stream(_stream, &db).await {
                        println!("error: {}", e);
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
