#![allow(dead_code)]

mod config;
mod parse;
mod rdb;
mod resp;
mod store;

use config::Config;
use parse::parse_command;
use resp::{parse_simple_string, receive_rdb_file};
use std::io::Error;
use std::sync::Arc;
use store::Database;
use tokio::sync::mpsc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    spawn,
};

use bytes::{Bytes, BytesMut};

pub enum Command {
    Ping,
    Echo(String),
    Set(String, String, Option<u64>),
    Get(String),
    Keys(String),
    ConfigGet(String),
    Info(Option<String>),
    Replconf(Vec<String>),
    Psync(String, Option<usize>),
    Unknown,
}

async fn execute_command(
    stream: &mut TcpStream,
    command: Command,
    db: &Database,
    tx: mpsc::UnboundedSender<String>,
    mut reply: bool,
) -> Result<(), Error> {
    let resp: Bytes = match command {
        Command::Ping => Bytes::from_static(b"+PONG\r\n"),
        Command::Echo(echo_arg) => Bytes::from(format!("+{}\r\n", echo_arg)),
        Command::Set(key, value, expiry_in_ms) => {
            let cmd_raw = resp::encoding_array(&["set", &key, &value]);
            db.spread(&cmd_raw).await;
            println!("set: {} {}", key, value);

            let resp = match expiry_in_ms {
                Some(expiry_in_ms) => {
                    db.set_with_expire(&key, &value, expiry_in_ms).await;
                    Bytes::from_static(b"+OK\r\n")
                }
                None => {
                    db.set(&key, &value).await;
                    Bytes::from_static(b"+OK\r\n")
                }
            };

            resp
        }
        Command::Get(key) => {
            println!("get: {}", key);
            reply = true;
            match db.get(&key).await {
                Some(value) => Bytes::from(format!("+{}\r\n", value)),
                None => Bytes::from_static(b"$-1\r\n"),
            }
        }
        Command::Keys(pattern) => {
            let mut keys = db.keys(&pattern).await;
            keys.sort();
            let mut resp = String::new();
            resp.push_str(&format!("*{}\r\n", keys.len()));
            for key in keys {
                resp.push_str(&format!("${}\r\n{}\r\n", key.len(), key));
            }
            Bytes::from(resp)
        }
        Command::ConfigGet(key) => match db.config_get(key.as_str()) {
            Some(value) => Bytes::from(format!(
                "*2\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
                key.len(),
                key,
                value.len(),
                value
            )),
            None => Bytes::from_static(b"$-1\r\n"),
        },
        Command::Info(parm) => match parm {
            Some(parm) => execute_info_command(parm, db.config()),
            None => Bytes::from_static(b"-Failed to fetch\r\n"),
        },
        Command::Replconf(args) => {
            match args[0].as_str() {
                "listening-port" => {
                    let ip: String = stream.peer_addr().unwrap().ip().to_string();
                    let port = args[1].parse::<u16>().unwrap();
                    println!("replication added: {}:{}", &ip, port);
                    db.add_replication(tx).await;
                }
                _ => {}
            }

            Bytes::from_static(b"+OK\r\n")
        }
        Command::Psync(_, _) => {
            let id = "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb";
            let offset = 0;
            let mut bytes = BytesMut::new();
            //format!("+FULLRESYNC {} {}\r\n", id, offset)
            bytes.extend_from_slice(b"+FULLRESYNC ");
            bytes.extend_from_slice(id.as_bytes());
            bytes.extend_from_slice(b" ");
            bytes.extend_from_slice(offset.to_string().as_bytes());
            bytes.extend_from_slice(b"\r\n");

            bytes.extend(resp::rdb_file(&rdb::empty_rdb()));

            bytes.freeze()
        }

        Command::Unknown => Bytes::from_static(b"-ERR unknown command\r\n"),
    };

    println!("Reply: {:?}", reply);
    if reply {
        stream.write_all(&resp).await?;
    }
    Ok(())
}

fn execute_info_command(parm: String, config: &Config) -> Bytes {
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
        _ => Bytes::from_static(b"-Failed to fetch\r\n"),
    }
}

async fn handle_stream(mut stream: TcpStream, db: &Database, reply: bool) -> Result<(), Error> {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let mut buf = [0; 1];
    loop {
        tokio::select! {
            n = stream.peek(&mut buf) => {
                let n = n.unwrap_or(0);
                if n == 0 {
                    break;
                }
                let command = parse_command(&mut stream).await;
                let _ = execute_command(&mut stream, command, db, tx.clone(), reply).await;
            }

            Some(msg) = rx.recv() => {
                stream.write_all(msg.as_bytes()).await?;
            }
        }
    }
    Ok(())
}

async fn connect_to_master(address: &str, config: &Config) -> Result<TcpStream, Error> {
    let mut stream = TcpStream::connect(address).await?;
    stream.write_all("*1\r\n$4\r\nping\r\n".as_bytes()).await?;

    println!("Send handshake 1");

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

    println!("Send handshake 2");

    let rec = parse_simple_string(&mut stream).await.unwrap();
    println!("PSYNC: {}", rec);
    let _ = receive_rdb_file(&mut stream).await.unwrap();
    println!("RDB received");
    Ok(stream)
}

#[tokio::main]
async fn main() {
    let db = Database::new();
    let address = format!("127.0.0.1:{}", &db.config().get("port").unwrap());
    let listener = TcpListener::bind(&address)
        .await
        .expect(format!("Failed to bind to {}", address).as_str());
    println!("Listening on {}", address);

    let db = Arc::new(db);

    if let Some(address) = db.config().get("replicaof") {
        println!("Connecting to master at {}", address);
        match connect_to_master(&address, db.config()).await {
            Ok(stream) => {
                let db = Arc::clone(&db);
                spawn(async move {
                    if let Err(e) = handle_stream(stream, &db, false).await {
                        println!("error: {}", e);
                    }
                });
            }
            Err(e) => {
                println!("Failed to connect to master: {}", e);
            }
        }
    }

    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((_stream, _)) => {
                let db = Arc::clone(&db);
                spawn(async move {
                    if let Err(e) = handle_stream(_stream, &db, true).await {
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
