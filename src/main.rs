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

    // Send a welcome message to the new client
    client.send_to("Welcome to the chat server! Please log in or register.\n");

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
        println!("Received from {}: {}", client.address, input);

        let response = match input.split_once(' ') {
            Some(("register", args)) => {
                let (username, password) = match args.split_once(' ') {
                    Some((u, p)) => (u, p),
                    None => return client.send_to("Usage: register <username> <password>\n"),
                };

                let mut db = db.lock().unwrap();
                db.register(username, password).unwrap_or_else(|e| e)
            }
            Some(("login", args)) => {
                let (username, password) = match args.split_once(' ') {
                    Some((u, p)) => (u, p),
                    None => return client.send_to("Usage: login <username> <password>\n"),
                };

                let mut db = db.lock().unwrap();
                db.login(username, password).unwrap_or_else(|e| e)
            }
            Some(("list", _)) => {
                let db = db.lock().unwrap();
                db.list_users().join("\n")
            }
            _ => "Unknown command. Use 'register', 'login', or 'list'.".to_string(),
        };

        client.send_to(&format!("{}\n", response));
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server is running on 127.0.0.1:7878");

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
