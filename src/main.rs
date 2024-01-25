use std::io::Error;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

async fn execute_command(stream: TcpStream) -> Result<(), Error> {
    let response = "+PONG\r\n";
    let mut stream = stream;
    let mut buf = [0; 1024];
    while let Ok(n) = stream.read(&mut buf).await {
        if n == 0 {
            break;
        }
        stream.write_all(response.as_bytes()).await?;
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
                if let Err(e) = execute_command(_stream).await {
                    println!("error: {}", e);
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
