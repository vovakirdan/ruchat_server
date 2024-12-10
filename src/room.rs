use crate::user::User;
use std::collections::HashMap;

pub struct Room {
    name: String,
    members: HashMap<String, User>,
}

impl Room {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            members: HashMap::new(),
        }
    }

    pub fn add_member(&mut self, user: User) {
        self.members.insert(user.username.clone(), user);
    }

    pub fn remove_member(&mut self, username: &str) {
        self.members.remove(username);
    }

    pub fn broadcast(&self, message: &str) {
        for (_, user) in &self.members {
            println!("[{}]: {}", self.name, message);  // todo replace with send to fn
        }
    }
}