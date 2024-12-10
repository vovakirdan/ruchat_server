use std::net::TcpStream;
use std::io::Write;

#[derive(Debug)]
pub struct Client {
    pub address: String,
    pub stream: TcpStream,
    pub is_logged_in: bool,
    pub username: Option<String>,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        let address = stream.peer_addr().unwrap().to_string();
        Self {
            address,
            stream,
            is_logged_in: false,
            username: None,
        }
    }

    // function to send a message to client
    pub fn send_to(&mut self, message: &str) {
        if let Err(e) = self.stream.write_all(message.as_bytes()) {
            eprintln!("Failed to send message to {}: {}", self.address, e);
        }
    }
}