use std::io::Error;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    spawn,
};

enum Command {
    Ping,
    Echo(String),
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
            let mut pos = pos + string_lemgth + 2;
            let mut string_lemgth = 0;
            if input[pos] != RESPDataType::BULK_STRING {
                return Err(Error::new(std::io::ErrorKind::InvalidData, "invalid data"));
            }
            pos += 1;

            pos += parse_lenght(&input[pos..], &mut string_lemgth).await;
            let echo_arg = String::from_utf8_lossy(&input[pos..pos + string_lemgth]).to_string();
            Ok(Command::Echo(echo_arg))
        }
        _ => Err(Error::new(std::io::ErrorKind::InvalidData, "invalid data")),
    };
}

async fn execute_command(stream: &mut TcpStream, command: Command) -> Result<(), Error> {
    let resp: String;
    match command {
        Command::Ping => {
            resp = "+PONG\r\n".to_string();
        }
        Command::Echo(echo_arg) => {
            resp = format!("+{}\r\n", echo_arg);
        }
    }

    stream.write_all(resp.as_bytes()).await?;
    Ok(())
}

async fn handle_stream(stream: TcpStream) -> Result<(), Error> {
    let mut stream = stream;
    let mut buf = [0; 1024];
    while let Ok(n) = stream.read(&mut buf).await {
        if n == 0 {
            break;
        }
        match parse_command(&buf[..n]).await {
            Ok(cmd) => execute_command(&mut stream, cmd).await?,

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
    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .expect("failed to bind");

    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((_stream, _)) => {
                println!("accepted new connection");
                spawn(async move {
                    if let Err(e) = handle_stream(_stream).await {
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
