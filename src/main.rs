use std::io::prelude::*;
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for con in listener.incoming() {
        let mut stream = con.unwrap();
        let mut buffer = [0 as u8; 50];
        let mut buffersize;

        loop {
            buffersize = stream.read(&mut buffer).unwrap();

            println!(
                "Message received from {} => {}",
                stream.peer_addr().unwrap(),
                String::from_utf8_lossy(&buffer[..buffersize])
            );

            stream.write(&buffer[..buffersize]).unwrap();
        }
    }
}
