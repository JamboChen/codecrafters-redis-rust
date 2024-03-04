use tokio::net::TcpStream;

use crate::resp::parse_array;
use crate::Command;

pub async fn parse_command(stream: &mut TcpStream) -> (Command, usize) {
    let (tokens, offset) = parse_array(stream).await.unwrap();

    let command = match tokens[0].to_lowercase().as_str() {
        "ping" => Command::Ping,
        "echo" if tokens.len() == 2 => Command::Echo(tokens[1].clone()),
        "set" => match tokens.len() {
            3 => Command::Set(tokens[1].clone(), tokens[2].clone(), None),
            5 if tokens[3].to_lowercase() == "px" => {
                let expiry_in_ms = tokens[4].parse::<u64>().unwrap();
                Command::Set(tokens[1].clone(), tokens[2].clone(), Some(expiry_in_ms))
            }
            _ => Command::Unknown,
        },
        "get" if tokens.len() == 2 => Command::Get(tokens[1].clone()),
        "keys" if tokens.len() == 2 => Command::Keys(tokens[1].clone()),
        "config" => match tokens[1].to_lowercase().as_str() {
            "get" => Command::ConfigGet(tokens[2].clone()),
            _ => Command::Unknown,
        },
        "info" => {
            if tokens.len() < 2 {
                Command::Info(None)
            } else {
                Command::Info(Some(tokens[1].clone()))
            }
        }
        "replconf" => Command::Replconf(
            tokens[1..]
                .iter()
                .map(|s| s.clone().to_lowercase())
                .collect(),
        ),
        "psync" if tokens.len() == 3 => {
            let offset = if tokens[2] == "-1" {
                None
            } else {
                Some(tokens[2].parse().unwrap())
            };
            Command::Psync(tokens[1].clone(), offset)
        }
        "wait" if tokens.len() == 3 => {
            let count = tokens[1].parse().unwrap();
            let timeout = tokens[2].parse().unwrap();
            Command::Wait(count, timeout)
        }
        _ => Command::Unknown,
    };

    (command, offset)
}
