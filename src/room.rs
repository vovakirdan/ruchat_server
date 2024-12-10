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
        clients: &Arc<Mutex<HashMap<String, Arc<Mutex<Client>>>>>
    ) {
        // For each member in the room, except sender, send the message
        let map = clients.lock().unwrap();
        for (_, client_arc) in map.iter() {
            let mut c = client_arc.lock().unwrap();
            if let Some(u) = &c.username {
                if u != sender && c.current_room == room_name && c.is_logged_in {
                    c.send_to(&format!("{}: {}\n", sender, message));
                }
            }
        }
    }
}
