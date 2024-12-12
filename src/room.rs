// room.rs
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::client::Client;

#[derive(Debug)]
pub struct Room {
    name: String,
    members: Arc<Mutex<HashSet<Client>>>, // теперь храним Client
}

impl Room {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            members: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn add_member(&self, client: Client) {
        if let Ok(mut members) = self.members.lock() {
            members.insert(client);
        } else {
            eprintln!("Failed to lock members for room {}", self.name);
        }
    }

    pub fn remove_member(&self, client: &Client) {
        if let Ok(mut members) = self.members.lock() {
            members.remove(client);
        } else {
            eprintln!("Failed to lock members for room {}", self.name);
        }
    }

    pub fn broadcast(&self, message: &str) {
        if let Ok(members) = self.members.lock() {
            for member in members.iter() {
                member.send_message(&format!("{}\n> ", message));
            }
        } else {
            eprintln!("Failed to lock members for broadcasting in room {}", self.name);
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_members(&self) -> Vec<Client> {
        if let Ok(members) = self.members.lock() {
            members.iter().cloned().collect()
        } else {
            eprintln!("Failed to lock members for room {}", self.name);
            Vec::new()
        }
    }
}
