mod user;
mod database;
mod client;

use std::net::TcpListener;
// use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::io::Read;
use std::thread;

use client::{Client, ClientState};
use database::Database;

const COMMANDS: &[&str] = &[
    "/list - List all users",
    "/cr <room_name> - Create a room",
    "/sr <room_name> - Switch to a room",
    "/q - Logout",
    "/disconnect - Disconnect",
    "@username message - Send a private message",
    "/list_rooms - List all rooms",
];

pub fn display_help(client: &Client) {
    client.send_message("Available commands:\n");
    for cmd in COMMANDS {
        client.send_message(&format!("{}\n", cmd));
    }
}

fn parse_command(
    cmd: &str,
    client: &Client,
    state: &Arc<Mutex<ClientState>>,
    db: &Arc<Mutex<Database>>,
    clients: &Arc<Mutex<std::collections::HashMap<String,(Client, Arc<Mutex<ClientState>>)>>>,
) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "/help" => {
            display_help(client);
        }
        "/list" => {
            if parts.len() < 2 {
                client.send_message("Usage: /list <users|rooms>\n");
                return;
            }

            match parts[1] {
                "users" => {
                    let db = db.lock().unwrap();
                    let users = db.list_users();
                    client.send_message("Online/Offline Users:\n");
                    client.send_message(&format!("{}\n", users.join("\n")));
                }
                "rooms" => {
                    let db_lock = db.lock().unwrap();
                    let room_names = db_lock.list_rooms();
                    client.send_message("Available Rooms:\n");
                    client.send_message(&format!("{}\n", room_names.join("\n")));
                }
                _ => {
                    client.send_message("Invalid argument. Use '/list users' or '/list rooms'.\n");
                }
            }
        }
        "/q" | "/quit" => {
            // Log out but keep connection open
            if client.username.is_empty() {
                return;
            }
            let mut db_lock = db.lock().unwrap();
            db_lock.logout(&client.username);
            clients.lock().unwrap().remove(&client.username);
            
            let mut st = state.lock().unwrap();
            st.is_logged_in = false;
            st.current_room = None;
            client.send_message("You have been logged out.\n");
            client.send_message("1. Sign up\n2. Sign in\nPlease choose (1|2): ");
        }
        "/disconnect" => {
            // Log out and disconnect
            if client.username.is_empty() {
                return;
            }
            db.lock().unwrap().logout(&client.username);
            clients.lock().unwrap().remove(&client.username);
            
            let mut st = state.lock().unwrap();
            st.is_logged_in = false;
            st.current_room = None;
            
            client.send_message("You have been logged out and disconnected.\n");
            st.mark_disconnected = true;
        }
        "/cr" | "/create_room" => {
            if parts.len() < 2 {
                client.send_message("Usage: /cr <room_name>\n");
                return;
            }
            let room_name = parts[1].trim();
            if room_name.is_empty() {
                client.send_message("Room name cannot be empty.\n");
                return;
            }
            let mut db_lock = db.lock().unwrap();
            match db_lock.add_room(room_name) {
                Ok(msg) => {
                    client.send_message(&format!("{}\n", msg));
                    let mut st = state.lock().unwrap();
                    st.current_room = Some(room_name.to_string());
                }
                Err(e) => {
                    client.send_message(&format!("{}\n", e));
                }
            }
        }

        "/sr" | "/switch_room" => {
            if parts.len() < 2 {
                client.send_message("Usage: /sr <room_name>\n");
                return;
            }
            let room_name = parts[1].trim();
            if room_name.is_empty() {
                client.send_message("Room name cannot be empty.\n");
                return;
            }
            let db_lock = db.lock().unwrap();
            if db_lock.room_exists(room_name) {
                let mut st = state.lock().unwrap();
                st.current_room = Some(room_name.to_string());
                client.send_message(&format!("Switched to room '{}'.\n", room_name));
            } else {
                client.send_message("Room does not exist.\n");
            }
        }
        _ => {
            // It's a command but not recognized
            client.send_message("Unknown command.\n");
        }
    }
}

fn handle_message(
    msg: &str,
    client: &Client,
    state: &Arc<Mutex<ClientState>>,
    clients: &Arc<Mutex<std::collections::HashMap<String,(Client, Arc<Mutex<ClientState>>)>>>,
) {
    let sender_state = state.lock().unwrap();
    let sender_room = match &sender_state.current_room {
        Some(r) => r.clone(),
        None => {
            client.send_message("You are not in a room.\n");
            return;
        }
    };
    drop(sender_state);

    let clients_map = clients.lock().unwrap();
    for (_, (other_client, other_state)) in clients_map.iter() {
        let other_st = other_state.lock().unwrap();
        if other_st.is_logged_in && other_st.current_room.as_ref() == Some(&sender_room) && other_client.username != client.username {
            other_client.send_message(&format!("[{}] {}: {}\n> ", sender_room, client.username, msg));
        }
    }
}

fn handle_client(
    client: Client,
    state: Arc<Mutex<ClientState>>,
    db: Arc<Mutex<Database>>,
    clients: Arc<Mutex<std::collections::HashMap<String,(Client, Arc<Mutex<ClientState>>)>>>,
) {
    client.send_message("Welcome to the chat server!\n");
    client.send_message("1. Sign up\n2. Sign in\nPlease choose (1|2): ");
    let mut client = client;

    loop {
        let input = read_client_input(&client);
        // if input.is_empty() {
        //     println!("Client {} disconnected.", client.address);
        //     return;
        // }

        {
            // Проверяем статус логина
            let st = state.lock().unwrap();
            if !st.is_logged_in {
                drop(st); // отпускаем лок
                // Если не залогинен, ждем выбора: 1 - регистрация, 2 - вход
                match input.as_str() {
                    "1" => {
                        client.send_message("Enter a username: ");
                        let username = read_client_input(&client);
                        if username.is_empty() {
                            return;
                        }
                        client.send_message("Enter a password: ");
                        let password = read_client_input(&client);
                        if password.is_empty() {
                            return;
                        }

                        let mut db_lock = db.lock().unwrap();
                        match db_lock.register(&username, &password) {
                            Ok(msg) => {
                                client.send_message(&format!("{}\n", msg));
                                client.username = username.clone(); // Присваиваем username сразу клиенту
                                {
                                    let mut st = state.lock().unwrap();
                                    st.is_logged_in = true;
                                    st.current_room = Some("main".to_string());
                                }
                                
                                let mut clients_map = clients.lock().unwrap();
                                clients_map.insert(username.clone(), (client.clone(), state.clone()));
                                client.send_message("You have joined the 'main' room.\n> ");
                            }
                            Err(err) => {
                                client.send_message(&format!("{}\n", err));
                            }
                        }
                    }
                    "2" => {
                        client.send_message("Enter your username: ");
                        let username = read_client_input(&client);
                        if username.is_empty() {
                            return;
                        }
                        client.send_message("Enter your password: ");
                        let password = read_client_input(&client);
                        if password.is_empty() {
                            return;
                        }

                        let mut db_lock = db.lock().unwrap();
                        match db_lock.login(&username, &password) {
                            Ok(msg) => {
                                client.send_message(&format!("{}\n", msg));
                                let mut st = state.lock().unwrap();
                                st.is_logged_in = true;
                                st.current_room = Some("main".to_string());
                                drop(st);

                                let mut clients_map = clients.lock().unwrap();
                                clients_map.insert(username.clone(), (client.clone(), state.clone()));
                                client.send_message("You have joined the 'main' room.\n> ");
                                
                                let mut client_mut = client.clone();
                                client_mut.username = username;
                            }
                            Err(err) => {
                                client.send_message(&format!("{}\n", err));
                            }
                        }
                    }
                    _ => {
                        client.send_message("Invalid choice. Please choose 1 (Sign up) or 2 (Sign in).\n");
                    }
                }
                continue;
            }
        }

        // Клиент залогинен
        if input.starts_with('/') {
            // Выполняем команду
            parse_command(&input, &client, &state, &db, &clients);

            let st = state.lock().unwrap();
            if st.mark_disconnected {
                println!("Client {} disconnected by command.", client.address);
                return;
            }
            drop(st);
        } else if !input.is_empty() {
            handle_message(&input, &client, &state, &clients);
        }

        // Если всё ещё залогинен, отправим промпт
        {
            let st = state.lock().unwrap();
            if st.is_logged_in {
                client.send_message("> ");
            }
        }
    }
}

fn read_client_input(client: &Client) -> String {
    let mut buffer = [0; 512];
    let stream_arc = Arc::clone(&client.stream);
    let mut stream = stream_arc.lock().unwrap();
    match stream.read(&mut buffer) {
        Ok(0) => return String::new(), // клиент отключился
        Ok(n) => String::from_utf8_lossy(&buffer[..n]).trim().to_string(),
        Err(_) => {
            client.send_message("Error reading input. Please try again.\n");
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
                
                // Распаковываем кортеж
                let (client, state) = Client::new(stream);
        
                let db = Arc::clone(&db);
                let clients_map = Arc::clone(&clients);
                
                // Передаем и client, и state
                thread::spawn(move || handle_client(client, state, db, clients_map));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }


    // println!("Waiting for threads to finish...");
    // thread::sleep(std::time::Duration::from_secs(1)); // Give threads time to finish
    // println!("Server shut down gracefully.");
}
