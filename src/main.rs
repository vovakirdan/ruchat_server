mod user;
mod room;
mod database;

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use database::Database;
use user::User;

fn handle_client(stream: TcpStream, db: Arc<Mutex<Database>>) {
    let mut buffer = [0; 512];
    if let Ok(mut stream) = stream.try_clone() {
        loop {
            let bytes_read = stream.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;  // connection closed
            }
            // process client input
            let input = String::from_utf8_lossy(&buffer[..bytes_read]);
            println!("Received: {}", input);
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server listening on port 7878");

    let db = Arc::new(Mutex::new(Database::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let db = Arc::clone(&db);
                thread::spawn(|| handle_client(stream, db));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
