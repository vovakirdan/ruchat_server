#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub password: String,
    pub is_online: bool,
}

impl User {
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
            is_online: false,
        }
    }
}