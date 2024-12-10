mod user;
mod room;
mod database;
mod client;

use std::net::TcpListener;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;

use client::Client;
use database::Database;
use room::Room;

fn parse_command(
    cmd: &str,
    client: &mut Client,
    db: &Arc<Mutex<Database>>,
    rooms: &Arc<Mutex<std::collections::HashMap<String, Room>>>,
    clients: &Arc<Mutex<std::collections::HashMap<String, Arc<Mutex<Client>>>>>,
) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "/list" => {
            let db = db.lock().unwrap();
            let users = db.list_users();
            client.send_to("Current users:\n");
            client.send_to(&format!("{}\n", users.join("\n")));
        }
        "/q" => {
            // Log out but keep connection open
            if let Some(username) = &client.username {
                db.lock().unwrap().logout(username);
                // Remove from current room
                if !client.current_room.is_empty() {
                    let mut rooms_lock = rooms.lock().unwrap();
                    if let Some(room) = rooms_lock.get_mut(&client.current_room) {
                        room.remove_member(username);
                    }
                }
                // Remove from global clients map
                clients.lock().unwrap().remove(username);
            }
            client.is_logged_in = false;
            client.username = None;
            client.current_room.clear();
            client.send_to("You have been logged out.\n");
            client.send_to("1. Sign up\n2. Sign in\nPlease choose (1|2): ");
        }
        "/disconnect" => {
            // Log out and disconnect
            if let Some(username) = &client.username {
                db.lock().unwrap().logout(username);
                // Remove from room
                if !client.current_room.is_empty() {
                    let mut rooms_lock = rooms.lock().unwrap();
                    if let Some(room) = rooms_lock.get_mut(&client.current_room) {
                        room.remove_member(username);
                    }
                }
                // Remove from global clients map
                clients.lock().unwrap().remove(username);
            }
            client.send_to("You have been logged out and disconnected.\n");
            // The caller should handle the actual disconnect by returning from handle_client.
            client.mark_disconnected = true;
        }
        "/cr" | "/create_room" => {
            if parts.len() < 2 {
                client.send_to("Usage: /cr <room_name>\n");
                return;
            }
            let room_name = parts[1].trim();
            if room_name.is_empty() {
                client.send_to("Room name cannot be empty.\n");
                return;
            }
            if room_name == "main" {
                client.send_to("Cannot create a room named 'main'.\n");
                return;
            }
            let mut rooms_lock = rooms.lock().unwrap();
            if rooms_lock.contains_key(room_name) {
                client.send_to("Room already exists.\n");
            } else {
                rooms_lock.insert(room_name.to_string(), Room::new(room_name));
                client.send_to(&format!("Room '{}' created.\n", room_name));
            }
        }
        "/sr" | "/switch_room" => {
            if parts.len() < 2 {
                client.send_to("Usage: /sr <room_name>\n");
                return;
            }
            let room_name = parts[1].trim();
            if room_name.is_empty() {
                client.send_to("Room name cannot be empty.\n");
                return;
            }
            let mut rooms_lock = rooms.lock().unwrap();

            if !rooms_lock.contains_key(room_name) {
                client.send_to("Room does not exist.\n");
                return;
            }

            // Remove user from old room
            if let Some(username) = &client.username {
                if let Some(old_room) = rooms_lock.get_mut(&client.current_room) {
                    old_room.remove_member(username);
                }
                // Add user to new room
                if let Some(new_room) = rooms_lock.get_mut(room_name) {
                    new_room.add_member(username.to_string());
                }

                client.current_room = room_name.to_string();
                client.send_to(&format!("Switched to room '{}'.\n", room_name));
            }
        }
        _ => {
            // It's a command but not recognized
            client.send_to("Unknown command.\n");
        }
    }
}

fn handle_message(
    msg: &str,
    client: &mut Client,
    rooms: &Arc<Mutex<std::collections::HashMap<String, Room>>>,
    clients: &Arc<Mutex<std::collections::HashMap<String, Arc<Mutex<Client>>>>>
) {
    // Broadcast the message to the current room
    if client.current_room.is_empty() {
        client.send_to("You are not in a room.\n");
        return;
    }
    let username = match &client.username {
        Some(u) => u.clone(),
        None => return,
    };

    let rooms_lock = rooms.lock().unwrap();
    if let Some(room) = rooms_lock.get(&client.current_room) {
        // We'll call a broadcast function on the room
        drop(rooms_lock); // Drop before calling broadcast to avoid double locks
        Room::broadcast(&client.current_room, &username, msg, clients);
    } else {
        client.send_to("You are in a non-existent room.\n");
    }
}

fn handle_client(
    mut client: Client,
    db: Arc<Mutex<Database>>,
    rooms: Arc<Mutex<std::collections::HashMap<String, Room>>>,
    clients: Arc<Mutex<std::collections::HashMap<String, Arc<Mutex<Client>>>>>
) {
    let mut buffer = [0; 512];

    client.send_to("Welcome to the chat server!\n");
    client.send_to("1. Sign up\n2. Sign in\nPlease choose (1|2): ");

    loop {
        if client.mark_disconnected {
            println!("Client {} disconnected by command.", client.address);
            return;
        }

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
                    // Registration
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
                            // Add client to global clients map
                            clients.lock().unwrap().insert(username.clone(), Arc::new(Mutex::new(client.clone())));

                            // Join main room
                            {
                                let mut rooms_lock = rooms.lock().unwrap();
                                let main_room = rooms_lock.get_mut("main").unwrap();
                                main_room.add_member(username.clone());
                                client.current_room = "main".to_string();
                            }
                        }
                        Err(err) => {
                            client.send_to(&format!("{}\n", err));
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
                            client.username = Some(username.clone());
                            client.is_logged_in = true;
                            // Add client to global clients map
                            clients.lock().unwrap().insert(username.clone(), Arc::new(Mutex::new(client.clone())));

                            // Join main room by default
                            let mut rooms_lock = rooms.lock().unwrap();
                            let main_room = rooms_lock.get_mut("main").unwrap();
                            main_room.add_member(username.clone());
                            client.current_room = "main".to_string();
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
            client.send_to("Room commands:\n- '/cr <room_name>' to create a room\n- '/sr <room_name>' or '/switch_room <room_name>' to switch rooms\n");
            client.send_to("> ");
        } else {
            // Logged in: parse commands or treat as message
            if input.starts_with('/') {
                parse_command(&input, &mut client, &db, &rooms, &clients);
                if client.mark_disconnected {
                    return;
                }

                // If logged out by /q, wait for further login/signup commands
                if !client.is_logged_in {
                    continue;
                }
            } else if input.starts_with('@') {
                // Private messaging not implemented in detail, just a placeholder
                client.send_to("Private messaging not implemented yet.\n");
            } else if !input.is_empty() {
                // Treat as message to the room
                handle_message(&input, &mut client, &rooms, &clients);
            }

            if client.mark_disconnected {
                return;
            }

            if client.is_logged_in {
                client.send_to("> ");
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server is running on 127.0.0.1:7878");
    println!("Use 'nc 127.0.0.1 7878' or 'telnet 127.0.0.1 7878' to connect");

    let db = Arc::new(Mutex::new(Database::new()));

    // Initialize main room
    let mut initial_rooms = std::collections::HashMap::new();
    initial_rooms.insert("main".to_string(), Room::new("main"));
    let rooms = Arc::new(Mutex::new(initial_rooms));

    // Map of username -> Client
    let clients = Arc::new(Mutex::new(std::collections::HashMap::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let address = stream.peer_addr().unwrap().to_string();
                println!("Client {} connected", address);
                let client = Client::new(stream);
                let db = Arc::clone(&db);
                let rooms = Arc::clone(&rooms);
                let clients_map = Arc::clone(&clients);
                thread::spawn(move || handle_client(client, db, rooms, clients_map));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
