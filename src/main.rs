use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

fn execute_command(stream: TcpStream) {
    let resp = "+PONG\r\n";
    let mut stream = stream;
    stream.write(resp.as_bytes()).expect("write failed");
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                execute_command(_stream)
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
