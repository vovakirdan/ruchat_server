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

    pub fn register(&mut self, username: &str, password: &str) -> Result <String, String> {
        if self.users.contains_key(username) {
            return Err("User already exists".to_string());
        }
        self.users.insert(username.to_string(), User::new(username, password));
        Ok(format!("User '{}' registered successfully.", username))
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<String, String> {
        if let Some(user) = self.users.get_mut(username) {
            if user.password == password {
                user.is_online = true;
                return Ok(format!("Welcome back, {}!", username));
            } else {
                return Err("Incorrect password.".to_string());
            }
        }
        Err("User not found.".to_string())
    }

    pub fn list_users(&self) -> Vec<String> {
        self.users
            .values()
            .map(|user| format!("{} (online: {})", user.username, user.is_online))
            .collect()
    }
}