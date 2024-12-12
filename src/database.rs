use crate::user::User;
use std::collections::HashMap;

pub struct Database {
    users: HashMap<String, User>,
    rooms: Vec<String>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            rooms: vec!["main".to_string()],
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

    pub fn logout(&mut self, username: &str) {
        if let Some(user) = self.users.get_mut(username) {
            user.is_online = false;
        }
    }

    pub fn list_users(&self) -> Vec<String> {
        self.users
            .values()
            .map(|user| {
                if user.is_online {
                    format!("{} online", user.username)
                } else {
                    format!("{} offline", user.username)
                }
            })
            .collect()
    }

    pub fn add_room(&mut self, room_name: &str) -> Result<String, String> {
        if self.rooms.contains(&room_name.to_string()) {
            return Err("Room already exists.".to_string());
        }
        self.rooms.push(room_name.to_string());
        Ok(format!("Room '{}' created.", room_name))
    }

    pub fn room_exists(&self, room_name: &str) -> bool {
        self.rooms.contains(&room_name.to_string())
    }

    pub fn list_rooms(&self) -> Vec<String> {
        self.rooms.clone()
    }
}