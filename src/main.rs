mod user;
mod room;
mod database;
mod client;

use std::net::TcpListener;
// use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::io::Read;
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
            if parts.len() < 2 {
                client.send_to("Usage: /list <users|rooms>\n");
                return;
            }

            match parts[1] {
                "users" => {
                    let db = db.lock().unwrap();
                    let users = db.list_users();
                    client.send_to("Online/Offline Users:\n");
                    client.send_to(&format!("{}\n", users.join("\n")));
                }
                "rooms" => {
                    let rooms_lock = rooms.lock().unwrap();
                    let room_names: Vec<String> = rooms_lock.keys().cloned().collect();
                    client.send_to("Available Rooms:\n");
                    client.send_to(&format!("{}\n", room_names.join("\n")));
                }
                _ => {
                    client.send_to("Invalid argument. Use '/list users' or '/list rooms'.\n");
                }
            }
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
                    new_room.add_member(username.clone());
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
    clients: &Arc<Mutex<std::collections::HashMap<String, Arc<Mutex<Client>>>>>,
) {
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
    clients: Arc<Mutex<std::collections::HashMap<String, Arc<Mutex<Client>>>>>,
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

        // If not logged in, prompt for login or registration
        if !client.is_logged_in {
            match input.as_str() {
                "1" => {
                    client.send_to("Enter a username: ");
                    let username = read_client_input(&mut client);
                    client.send_to("Enter a password: ");
                    let password = read_client_input(&mut client);

                    let mut db = db.lock().unwrap();
                    match db.register(&username, &password) {
                        Ok(msg) => {
                            client.send_to(&format!("{}\n", msg));
                            client.is_logged_in = true;
                            client.username = Some(username.clone());
                            client.current_room = "main".to_string();

                            // Add user to global map and room
                            clients.lock().unwrap().insert(username.clone(), Arc::new(Mutex::new(client.clone())));
                            let mut rooms_lock = rooms.lock().unwrap();
                            rooms_lock.get_mut("main").unwrap().add_member(username);
                            client.send_to("You have joined the 'main' room.\n");
                        }
                        Err(err) => {
                            client.send_to(&format!("{}\n", err));
                        }
                    }
                }
                "2" => {
                    client.send_to("Enter your username: ");
                    let username = read_client_input(&mut client);
                    client.send_to("Enter your password: ");
                    let password = read_client_input(&mut client);

                    let mut db = db.lock().unwrap();
                    match db.login(&username, &password) {
                        Ok(msg) => {
                            client.send_to(&format!("{}\n", msg));
                            client.is_logged_in = true;
                            client.username = Some(username.clone());
                            client.current_room = "main".to_string();

                            // Add user to global map and room
                            clients.lock().unwrap().insert(username.clone(), Arc::new(Mutex::new(client.clone())));
                            let mut rooms_lock = rooms.lock().unwrap();
                            rooms_lock.get_mut("main").unwrap().add_member(username);
                            client.send_to("You have joined the 'main' room.\n");
                        }
                        Err(err) => {
                            client.send_to(&format!("{}\n", err));
                        }
                    }
                }
                _ => {
                    client.send_to("Invalid choice. Please choose 1 (Sign up) or 2 (Sign in).\n");
                }
            }
            continue; // Skip further processing until logged in
        }

        // If logged in, check for commands or broadcast message
        if input.starts_with('/') {
            parse_command(&input, &mut client, &db, &rooms, &clients);
            if client.mark_disconnected {
                return;
            }
        } else if !input.is_empty() {
            // Broadcast message to the current room
            handle_message(&input, &mut client, &rooms, &clients);
        }

        if client.is_logged_in {
            client.send_to("> ");
        }
    }
}

fn read_client_input(client: &mut Client) -> String {
    let mut buffer = [0; 512];
    match client.stream.read(&mut buffer) {
        Ok(n) => String::from_utf8_lossy(&buffer[..n]).trim().to_string(),
        Err(_) => {
            client.send_to("Error reading input. Please try again.\n");
            String::new()
        }
    }
}

fn main() {
    // let is_running = Arc::new(AtomicBool::new(true));
    // let is_running_clone = Arc::clone(&is_running);

    // ctrlc::set_handler(move || {
    //     println!("\nReceived shutdown signal. Closing server...");
    //     is_running_clone.store(false, Ordering::SeqCst);
    // })
    // .expect("Error setting Ctrl-C handler");

    let listener = TcpListener::bind("127.0.0.1:7878").expect("Failed to bind to address");
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
        // if !is_running.load(Ordering::SeqCst) {
        //     break;
        // }
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


    // println!("Waiting for threads to finish...");
    // thread::sleep(std::time::Duration::from_secs(1)); // Give threads time to finish
    // println!("Server shut down gracefully.");
}
