mod args;
mod parse;
mod store;

use args::Args;
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
    Info(Option<String>),
    Unknown,
}

async fn execute_command(
    stream: &mut TcpStream,
    command: Command,
    db: &Database,
    args: &Args,
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
        Command::Info(parm) => match parm {
            Some(parm) => execute_info_command(parm, args),
            None => "-Failed to fetch\r\n".to_string(),
        },
        Command::Unknown => "-ERR unknown command\r\n".to_string(),
    };

    stream.write_all(resp.as_bytes()).await?;
    Ok(())
}

fn execute_info_command(parm: String, args: &Args) -> String {
    match parm.as_str() {
        "replication" => match &args.replicaof {
            Some(_) => "$10\r\nrole:slave\r\n".to_string(),
            None => "$11\r\nrole:master\r\n".to_string(),
        },
        _ => "-Failed to fetch\r\n".to_string(),
    }
}

async fn handle_stream(stream: TcpStream, db: &Database, args: &Args) -> Result<(), Error> {
    let mut stream = stream;
    let mut buf = [0; 1024];
    while let Ok(n) = stream.read(&mut buf).await {
        if n == 0 {
            break;
        }

        match parse_command(&buf[..n]).await {
            Ok(cmd) => execute_command(&mut stream, cmd, db, args).await?,

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
    let args = args::Args::load();

    let address = format!("127.0.0.1:{}", &args.port);
    println!("Listening on {}", address);
    let listener = TcpListener::bind(address).await.expect("failed to bind");

    let db = Database::new(args.dir.clone(), args.dbfilename.clone());
    let db = Arc::new(db);
    let args = Arc::new(args);

    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((_stream, _)) => {
                println!("accepted new connection");
                let db = Arc::clone(&db);
                let args = Arc::clone(&args);
                spawn(async move {
                    if let Err(e) = handle_stream(_stream, &db, &args).await {
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
