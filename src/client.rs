// client.rs
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct Client {
    pub address: String,
    pub stream: Arc<Mutex<TcpStream>>,
    pub username: String,
}

// Состояние клиента: логин, текущая комната и прочие динамические параметры
#[derive(Debug)]
pub struct ClientState {
    pub current_room: Option<String>,
    pub is_logged_in: bool,
    pub mark_disconnected: bool,
}

impl Client {
    pub fn new(stream: TcpStream) -> (Self, Arc<Mutex<ClientState>>) {
        let address = stream.peer_addr().unwrap().to_string();
        let stream = Arc::new(Mutex::new(stream));
        let client = Self {
            address,
            stream,
            username: String::new(),
        };
        let state = Arc::new(Mutex::new(ClientState {
            current_room: None,
            is_logged_in: false,
            mark_disconnected: false,
        }));
        (client, state)
    }

    pub fn send_message(&self, message: &str) {
        if let Ok(mut stream) = self.stream.lock() {
            if let Err(e) = stream.write_all(message.as_bytes()) {
                eprintln!("Failed to send message to {}: {}", self.address, e);
            }
        } else {
            eprintln!("Failed to lock stream for {}", self.address);
        }
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.username == other.username
    }
}

impl Eq for Client {}

impl Hash for Client {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
        self.username.hash(state);
    }
}
