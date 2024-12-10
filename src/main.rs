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

    // This loop handles the client's session until disconnected
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

        // If not logged in, handle registration / login
        if !client.is_logged_in {
            match input.as_str() {
                "1" => {
                    // Registration flow
                    client.send_to("Enter a username: ");
                    let username = match client.stream.read(&mut buffer) {
                        Ok(0) => {
                            println!("Client {} disconnected during registration username input.", client.address);
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
                            println!("Client {} disconnected during password input.", client.address);
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
                            client.send_to("1. Sign up\n2. Sign in\nPlease choose (1|2): ");
                            continue;
                        }
                    }
                }
                "2" => {
                    // Login flow
                    client.send_to("Enter your username: ");
                    let username = match client.stream.read(&mut buffer) {
                        Ok(0) => {
                            println!("Client {} disconnected during login username input.", client.address);
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
                            println!("Client {} disconnected during password input.", client.address);
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

            // Show main room info and enter command loop
            client.send_to("Welcome to the main room...\n");
            client.send_to("Instructions:\n- Use '/list' to see current users.\n- Use '@username message' to send private messages.\n");
            client.send_to("Commands:\n- '/q' to log out\n- '/disconnect' to log out and disconnect\n");
        } else {
            // The client is logged in, so 'input' here should be treated as a command
            // We should not break after handling a single command, we stay in command mode.
            // Let's implement a separate command loop after successful login.
            let mut cmd = input.to_string();

            // Since we currently read only once, the logic will be:
            // If the command is recognized, handle it,
            // else read again at the top of the loop.
            // But we can restructure to continuously prompt in a loop.

            loop {
                // If the previous command was handled outside, prompt again
                if cmd.is_empty() {
                    client.send_to("> ");
                    let bytes_read = match client.stream.read(&mut buffer) {
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
                    cmd = String::from_utf8_lossy(&buffer[..bytes_read]).trim().to_string();
                }

                if cmd == "/q" {
                    // Log out but keep connection
                    if let Some(ref username) = client.username {
                        let mut db = db.lock().unwrap();
                        db.logout(username);
                    }
                    client.is_logged_in = false;
                    client.username = None;
                    client.send_to("You have been logged out.\n");
                    client.send_to("1. Sign up\n2. Sign in\nPlease choose (1|2): ");
                    break; // break from command loop to handle sign up/sign in again
                } else if cmd == "/disconnect" {
                    // Log out and disconnect client
                    if let Some(ref username) = client.username {
                        let mut db = db.lock().unwrap();
                        db.logout(username);
                    }
                    client.send_to("You have been logged out and disconnected.\n");
                    println!("Client {} disconnected by user request.", client.address);
                    return; 
                } else if cmd.starts_with("/list") {
                    let db = db.lock().unwrap();
                    let users = db.list_users();
                    client.send_to("Current users:\n");
                    client.send_to(&format!("{}\n", users.join("\n")));
                } else if cmd.starts_with('@') {
                    // Private message logic placeholder
                    client.send_to("Private messaging not implemented yet.\n");
                } else if cmd.is_empty() {
                    // If empty, just prompt again
                } else {
                    client.send_to("Unknown command.\n");
                }

                // Prompt for next command
                client.send_to("> ");
                let bytes_read = match client.stream.read(&mut buffer) {
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
                cmd = String::from_utf8_lossy(&buffer[..bytes_read]).trim().to_string();
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
                let address = stream.peer_addr().unwrap().to_string();
                println!("Client {} connected", address);
                let client = Client::new(stream);
                let db = Arc::clone(&db);
                thread::spawn(|| handle_client(client, db));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
