use std::net::TcpStream;
use std::io::Write;
use std::clone::Clone;

#[derive(Debug)]
pub struct Client {
    pub address: String,
    pub stream: TcpStream,
    pub is_logged_in: bool,
    pub username: Option<String>,
    pub current_room: String,
    pub mark_disconnected: bool,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        let address = stream.peer_addr().unwrap().to_string();
        Self {
            address,
            stream,
            is_logged_in: false,
            username: None,
            current_room: String::new(),
            mark_disconnected: false,
        }
    }

    pub fn send_to(&mut self, message: &str) {
        if let Err(e) = self.stream.write_all(message.as_bytes()) {
            eprintln!("Failed to send message to {}: {}", self.address, e);
        }
    }
}

// Implement Clone manually, ignoring the non-cloneable TcpStream by using peer_addr again.
// Actually, for simplicity in this example, we will just derive Clone for fields that can be cloned,
// and skip the stream in the clone. The cloned client won't be fully functional, but we only clone
// this struct to store minimal info in the clients map. Another approach would be to store just the username.
impl Clone for Client {
    fn clone(&self) -> Self {
        // Not fully functional clone (stream not cloned)
        // but good enough for username/address references.
        Self {
            address: self.address.clone(),
            stream: self.stream.try_clone().expect("Failed to clone stream"),
            is_logged_in: self.is_logged_in,
            username: self.username.clone(),
            current_room: self.current_room.clone(),
            mark_disconnected: self.mark_disconnected,
        }
    }
}
