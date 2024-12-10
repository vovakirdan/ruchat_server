use crate::user::User;
use std::collections::HashMap;

pub struct Database {
    users: HashMap<String, User>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn register(&mut self, username: &str, password: &str) -> Result <(), &str> {
        if self.users.contains_key(username) {
            return Err("User already exists");
        }
        self.users.insert(username.to_string(), User::new(username, password));
        Ok(())
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<(), &str> {
        if let Some(user) = self.users.get_mut(username) {
            if user.password == password {
                user.is_online = true;
                return Ok(());
            } else {
                return Err("incorrect password");
            }
        }
        Err("User not found")
    }

    pub fn list_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }
}