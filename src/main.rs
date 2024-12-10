mod user;
mod room;
mod database;
mod client;

use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;

use client::Client;
use database::Database;

fn handle_client(mut client: Client, db: Arc<Mutex<Database>>) {
    let mut buffer = [0; 512];

    client.send_to("Welcome to the chat server!\n");
    client.send_to("1. Sign up\n2. Sign in\nPlease choose (1|2): ");

    loop {
        let bytes_read = match client.stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client {} disconnected.", client.address);
                return;
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("Error reading from {}: {}", client.address, e);
                return;
            }
        };

        let input = String::from_utf8_lossy(&buffer[..bytes_read]).trim().to_string();

        if !client.is_logged_in {
            match input.as_str() {
                "1" => {
                    client.send_to("Enter a username: ");
                    if let Ok(n) = client.stream.read(&mut buffer) {
                        let username = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                        client.send_to("Enter a password: ");
                        if let Ok(p) = client.stream.read(&mut buffer) {
                            let password = String::from_utf8_lossy(&buffer[..p]).trim().to_string();
                            let mut db = db.lock().unwrap();
                            match db.register(&username, &password) {
                                Ok(msg) => {
                                    client.send_to(&format!("{}\n", msg));
                                    client.username = Some(username);
                                    client.is_logged_in = true;
                                    client.send_to("You are now logged in.\n");
                                }
                                Err(err) => client.send_to(&format!("{}\n", err)),
                            }
                        }
                    }
                }
                "2" => {
                    client.send_to("Enter your username: ");
                    if let Ok(n) = client.stream.read(&mut buffer) {
                        let username = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                        client.send_to("Enter your password: ");
                        if let Ok(p) = client.stream.read(&mut buffer) {
                            let password = String::from_utf8_lossy(&buffer[..p]).trim().to_string();
                            let mut db = db.lock().unwrap();
                            match db.login(&username, &password) {
                                Ok(msg) => {
                                    client.send_to(&format!("{}\n", msg));
                                    client.username = Some(username);
                                    client.is_logged_in = true;
                                }
                                Err(err) => client.send_to(&format!("{}\n", err)),
                            }
                        }
                    }
                }
                _ => client.send_to("Unknown command. Please choose (1|2): "),
            }
        }

        if client.is_logged_in {
            client.send_to("Welcome to the main room...\n");
            client.send_to("Instructions:\n- Use 'list' to see online users.\n- Use '@username message' to send private messages.\n");

            let db = db.lock().unwrap();
            let users = db.list_users();
            client.send_to("Current users:\n");
            client.send_to(&format!("{}\n", users.join("\n")));

            return; // Exit loop after login and setup
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server is running on 127.0.0.1:7878");
    println!("run nc 127.0.0.1 787 to connect");

    let db = Arc::new(Mutex::new(Database::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let client = Client::new(stream);
                let db = Arc::clone(&db);
                thread::spawn(|| handle_client(client, db));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
