mod config;
mod parse;
mod rdb;
mod resp;
mod store;

use config::Config;
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
    Replconf,
    Psync(String, Option<usize>),
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
        Command::ConfigGet(key) => match db.config_get(key.as_str()) {
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
            Some(parm) => execute_info_command(parm, db.config()),
            None => "-Failed to fetch\r\n".to_string(),
        },
        Command::Replconf => "+OK\r\n".to_string(),
        Command::Psync(_, _) => {
            let id = "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb";
            let offset = 0;
            stream
                .write_all(format!("+FULLRESYNC {} {}\r\n", id, offset).as_bytes())
                .await?;

            resp::rdb_file(rdb::EMPTY)
        }
        Command::Unknown => "-ERR unknown command\r\n".to_string(),
    };

    stream.write_all(resp.as_bytes()).await?;
    Ok(())
}

fn execute_info_command(parm: String, config: &Config) -> String {
    match parm.as_str() {
        "replication" => {
            let master_replid = "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb";
            let master_repl_offset = 0;
            let mut info = String::new();
            if let Some(_) = &config.replicaof {
                info.push_str(&format!("role:slave\r\n"));
            } else {
                info.push_str(&format!("role:master\r\n"));
            }
            info.push_str(&format!("master_replid:{}\r\n", master_replid));
            info.push_str(&format!("master_repl_offset:{}\r\n", master_repl_offset));

            resp::bulk_string(&info)
        }
        _ => "-Failed to fetch\r\n".to_string(),
    }
}

async fn handle_stream(stream: TcpStream, db: &Database) -> Result<(), Error> {
    let mut stream = stream;
    let mut buf = [0; 1024];
    while let Ok(n) = stream.read(&mut buf).await {
        if n == 0 {
            break;
        }

        match parse_command(&buf[..n]) {
            Ok(cmd) => execute_command(&mut stream, cmd, db).await?,

            Err(e) => {
                println!("error: {}", e);
                break;
            }
        }
    }
    Ok(())
}

async fn connect_to_master(address: &str, config: &Config) -> Result<(), Error> {
    let mut stream = TcpStream::connect(address).await?;
    stream.write_all("*1\r\n$4\r\nping\r\n".as_bytes()).await?;

    let mut buf = [0; 8];
    stream.read(&mut buf).await?;
    assert_eq!(b"+PONG\r\n", &buf[..7]);

    // REPLCONF listening-port <PORT>
    stream
        .write_all(
            format!(
                "*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n$4\r\n{}\r\n",
                config.port
            )
            .as_bytes(),
        )
        .await?;

    stream.read(&mut buf).await?;
    assert_eq!(b"+OK\r\n", &buf[..5]);

    // REPLCONF capa psync2
    stream
        .write_all("*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n".as_bytes())
        .await?;

    stream.read(&mut buf).await?;
    assert_eq!(b"+OK\r\n", &buf[..5]);

    // PSYNC ? -1
    stream
        .write_all("*3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n".as_bytes())
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let db = Database::new();
    let address = format!("127.0.0.1:{}", &db.config().get("port").unwrap());
    let listener = TcpListener::bind(&address).await.expect("failed to bind");
    println!("Listening on {}", address);

    if let Some(address) = db.config().get("replicaof") {
        println!("Connecting to master at {}", address);
        if let Err(e) = connect_to_master(&address, db.config()).await {
            println!("error: {}", e);
        }
    }

    let db = Arc::new(db);

    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((_stream, _)) => {
                println!("accepted new connection");
                let db = Arc::clone(&db);
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
