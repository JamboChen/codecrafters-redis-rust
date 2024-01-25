mod config;
mod parse;
mod store;
use std::io::Error;
use store::Database;

use parse::parse_command;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    spawn,
};

enum Command {
    Ping,
    Echo(String),
    Set(String, String, Option<u64>),
    Get(String),
    ConfigGet(String),
    Unknown,
}

async fn execute_command(
    stream: &mut TcpStream,
    command: Command,
    db: &Database,
    config: &config::Config,
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
        Command::ConfigGet(key) => match config.get(key.as_str()) {
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

async fn handle_stream(
    stream: TcpStream,
    db: &Database,
    config: &config::Config,
) -> Result<(), Error> {
    let mut stream = stream;
    let mut buf = [0; 1024];
    while let Ok(n) = stream.read(&mut buf).await {
        if n == 0 {
            break;
        }

        match parse_command(&buf[..n]).await {
            Ok(cmd) => execute_command(&mut stream, cmd, db, config).await?,

            Err(e) => {
                println!("error: {}", e);
                break;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let mut config = config::Config::new();
    config.from_args();
    println!("{:?}", config);

    let config = Arc::new(config);
    let db = Arc::new(Database::new());

    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .expect("failed to bind");

    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((_stream, _)) => {
                println!("accepted new connection");
                let config = Arc::clone(&config); // Move this line outside of the loop
                let db = Arc::clone(&db); // Move this line outside of the loop
                spawn(async move {
                    if let Err(e) = handle_stream(_stream, &db, &config).await {
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
