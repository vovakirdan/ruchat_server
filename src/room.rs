use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::client::Client;

pub struct Room {
    name: String,
    members: Vec<String>, // Just store usernames
}

impl Room {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            members: Vec::new(),
        }
    }

    pub fn add_member(&mut self, username: String) {
        if !self.members.contains(&username) {
            self.members.push(username);
        }
    }

    pub fn remove_member(&mut self, username: &str) {
        self.members.retain(|u| u != username);
    }

    pub fn broadcast(
        room_name: &str,
        sender: &str,
        message: &str,
        clients: &Arc<Mutex<HashMap<String, Arc<Mutex<Client>>>>>,
    ) {
        let clients_lock = clients.lock().unwrap();
        for (_, client_arc) in clients_lock.iter() {
            let mut client = client_arc.lock().unwrap();
            if client.current_room == room_name && client.is_logged_in && client.username.as_deref() != Some(sender) {
                client.send_to(&format!("\n[{}] {}: {}\n> ", room_name, sender, message));
            }
        }
        // Print to the server's console for logging/debugging
        println!("[{}] {}: {}", room_name, sender, message);
    }      
}
