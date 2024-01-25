mod store;

use std::io::Error;
use store::Database;

use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    spawn,
};

enum Command {
    Ping,
    Echo(String),
    Set(String, String),
    Get(String),
}

struct RESPDataType {}
impl RESPDataType {
    const BULK_STRING: u8 = b'$'; // 0x24
    const ARRAY: u8 = b'*'; // 0x2a
}

// return the offset to skip the parsed data
async fn parse_lenght(input: &[u8], len: &mut usize) -> usize {
    println!("input: {:?}", std::str::from_utf8(input));
    let mut pos: usize = 0;
    *len = 0;
    while input[pos] != b'\r' {
        *len = *len * 10 + (input[pos] - b'0') as usize;
        pos += 1;
    }
    pos + 2
}

async fn parse_bulk_string(input: &[u8], result: &mut String) -> usize {
    if input[0] != RESPDataType::BULK_STRING {
        return 0;
    }

    let mut pos: usize = 1;
    let mut string_lemgth = 0;
    pos += parse_lenght(&input[pos..], &mut string_lemgth).await;

    *result = String::from_utf8_lossy(&input[pos..pos + string_lemgth]).to_string();
    pos + string_lemgth + 2
}

async fn parse_echo_arg(input: &[u8]) -> Result<String, Error> {
    let mut echo = String::new();
    let _ = parse_bulk_string(input, &mut echo).await;
    Ok(echo)
}

async fn parse_set_arg(input: &[u8]) -> Result<(String, String), Error> {
    let mut key = String::new();
    let pos = parse_bulk_string(input, &mut key).await;

    let mut value = String::new();
    let _ = parse_bulk_string(&input[pos..], &mut value).await;

    Ok((key, value))
}

async fn parse_get_arg(input: &[u8]) -> Result<String, Error> {
    let mut result = String::new();
    let _ = parse_bulk_string(input, &mut result).await;
    Ok(result)
}

async fn parse_command(input: &[u8]) -> Result<Command, Error> {
    if input[0] != RESPDataType::ARRAY {
        return Err(Error::new(std::io::ErrorKind::InvalidData, "invalid data"));
    }

    let mut pos: usize = 1;
    let mut args_count = 0;
    pos += parse_lenght(&input[pos..], &mut args_count).await;
    println!("args_count: {}", args_count);

    if input[pos] != RESPDataType::BULK_STRING {
        return Err(Error::new(std::io::ErrorKind::InvalidData, "invalid data"));
    }
    pos += 1;

    let mut string_lemgth = 0;
    pos += parse_lenght(&input[pos..], &mut string_lemgth).await;
    println!("string_lemgth: {}", string_lemgth);

    let command = String::from_utf8_lossy(&input[pos..pos + string_lemgth]).to_ascii_uppercase();
    return match command.as_str() {
        "PING" => Ok(Command::Ping),
        "ECHO" => {
            if args_count != 2 {
                return Err(Error::new(std::io::ErrorKind::InvalidData, "invalid data"));
            }
            pos = pos + string_lemgth + 2;
            let echo_arg = parse_echo_arg(&input[pos..]).await?;
            Ok(Command::Echo(echo_arg))
        }
        "SET" => {
            if args_count != 3 {
                return Err(Error::new(std::io::ErrorKind::InvalidData, "invalid data"));
            }
            pos = pos + string_lemgth + 2;
            let (key, value) = parse_set_arg(&input[pos..]).await?;
            Ok(Command::Set(key, value))
        }
        "GET" => {
            if args_count != 2 {
                return Err(Error::new(std::io::ErrorKind::InvalidData, "invalid data"));
            }
            pos = pos + string_lemgth + 2;
            let key = parse_get_arg(&input[pos..]).await?;
            Ok(Command::Get(key))
        }
        _ => Err(Error::new(std::io::ErrorKind::InvalidData, "invalid data")),
    };
}

async fn execute_command(
    stream: &mut TcpStream,
    command: Command,
    db: &Database,
) -> Result<(), Error> {
    let resp: String;
    match command {
        Command::Ping => {
            resp = "+PONG\r\n".to_string();
        }
        Command::Echo(echo_arg) => {
            resp = format!("+{}\r\n", echo_arg);
        }
        Command::Set(key, value) => {
            db.set(&key, &value).await;
            resp = "+OK\r\n".to_string();
        }
        Command::Get(key) => match db.get(&key).await {
            Some(value) => {
                resp = format!("+{}\r\n", value);
            }
            None => {
                resp = "$-1\r\n".to_string();
            }
        },
    }

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

#[tokio::main]
async fn main() {
    let db = Arc::new(Database::new());

    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .expect("failed to bind");

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

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_parse_length() {
        let input = b"123\r\n";
        let mut len = 0;
        let pos = parse_lenght(input, &mut len).await;
        assert_eq!(pos, 5);
        assert_eq!(len, 123);
    }

    #[tokio::test]
    async fn test_parse_bulk_string() {
        let input = b"$6\r\nfoobar\r\n";
        let mut result = String::new();
        let pos = parse_bulk_string(input, &mut result).await;
        assert_eq!(pos, 12);
        assert_eq!(result, "foobar");
    }
}
