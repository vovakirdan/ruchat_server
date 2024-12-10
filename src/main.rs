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

        // If the client is not logged in yet, handle registration/login flow
        if !client.is_logged_in {
            match input.as_str() {
                "1" => {
                    // Registration
                    client.send_to("Enter a username: ");
                    let username = match client.stream.read(&mut buffer) {
                        Ok(0) => {
                            println!("Client {} disconnected during registration.", client.address);
                            return;
                        }
                        Ok(n) => String::from_utf8_lossy(&buffer[..n]).trim().to_string(),
                        Err(e) => {
                            eprintln!("Error reading username: {}", e);
                            return;
                        }
                    };

                    client.send_to("Enter a password: ");
                    let password = match client.stream.read(&mut buffer) {
                        Ok(0) => {
                            println!("Client {} disconnected during password entry.", client.address);
                            return;
                        }
                        Ok(p) => String::from_utf8_lossy(&buffer[..p]).trim().to_string(),
                        Err(e) => {
                            eprintln!("Error reading password: {}", e);
                            return;
                        }
                    };

                    let mut db = db.lock().unwrap();
                    match db.register(&username, &password) {
                        Ok(msg) => {
                            client.send_to(&format!("{}\n", msg));
                            client.username = Some(username.clone());
                            client.is_logged_in = true;
                            client.send_to("You are now logged in.\n");
                        }
                        Err(err) => {
                            client.send_to(&format!("{}\n", err));
                            // After an error, prompt again for login/sign up choice
                            client.send_to("1. Sign up\n2. Sign in\nPlease choose (1|2): ");
                            continue;
                        }
                    }
                }
                "2" => {
                    // Login
                    client.send_to("Enter your username: ");
                    let username = match client.stream.read(&mut buffer) {
                        Ok(0) => {
                            println!("Client {} disconnected during login username entry.", client.address);
                            return;
                        }
                        Ok(n) => String::from_utf8_lossy(&buffer[..n]).trim().to_string(),
                        Err(e) => {
                            eprintln!("Error reading username: {}", e);
                            return;
                        }
                    };

                    client.send_to("Enter your password: ");
                    let password = match client.stream.read(&mut buffer) {
                        Ok(0) => {
                            println!("Client {} disconnected during password entry.", client.address);
                            return;
                        }
                        Ok(p) => String::from_utf8_lossy(&buffer[..p]).trim().to_string(),
                        Err(e) => {
                            eprintln!("Error reading password: {}", e);
                            return;
                        }
                    };

                    let mut db = db.lock().unwrap();
                    match db.login(&username, &password) {
                        Ok(msg) => {
                            client.send_to(&format!("{}\n", msg));
                            client.username = Some(username);
                            client.is_logged_in = true;
                        }
                        Err(err) => {
                            client.send_to(&format!("{}\n", err));
                            // After an error, prompt again for login/sign up choice
                            client.send_to("1. Sign up\n2. Sign in\nPlease choose (1|2): ");
                            continue;
                        }
                    }
                }
                _ => {
                    client.send_to("Unknown command. Please choose (1|2): ");
                    continue;
                }
            }

            // Once logged in, show main room info and go into command loop
            client.send_to("Welcome to the main room...\n");
            client.send_to("Instructions:\n- Use '/list' to see current users.\n- Use '@username message' to send private messages.\n");
        }

        // Now the client is logged in, we enter a command loop
        loop {
            client.send_to("> "); // Prompt for next command
            let cmd_bytes = match client.stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Client {} disconnected.", client.address);
                    return;
                }
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Error reading command: {}", e);
                    return;
                }
            };

            let command = String::from_utf8_lossy(&buffer[..cmd_bytes]).trim().to_string();

            if command.starts_with("/list") {
                let db = db.lock().unwrap();
                let users = db.list_users();
                client.send_to("Current users:\n");
                client.send_to(&format!("{}\n", users.join("\n")));
            } else if command.starts_with('@') {
                // For now, just acknowledge private messages (not implemented yet)
                client.send_to("Private messaging not implemented yet.\n");
            } else if command.is_empty() {
                // If empty line, just continue
                continue;
            } else {
                client.send_to("Unknown command.\n");
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server is running on 127.0.0.1:7878");
    println!("Use 'nc 127.0.0.1 7878' or 'telnet 127.0.0.1 7878' to connect");

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
