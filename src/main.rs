use std::{
    io::{Error, Read, Write},
    net::{TcpListener, TcpStream},
};

fn execute_command(stream: TcpStream) -> Result<(), Error> {
    let response = "+PONG\r\n";
    let mut stream = stream;
    let mut buf = [0; 1024];
    while let Ok(_) = stream.read(&mut buf) {
        stream.write(response.as_bytes())?;
    }
    Ok(())
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                if let Err(e) = execute_command(_stream) {
                    println!("error: {}", e);
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
