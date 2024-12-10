# Chatroom Server

A multi-threaded chatroom server written in Rust, allowing users to communicate in public rooms and send private messages to specific users.

## Features

### User Management
- **Sign Up**: Register with a unique username and password using the `1` command.
- **Sign In**: Log in with credentials using the `2` command.
- **Log Out**: Log out without disconnecting using the `/q` command.
- **Disconnect**: Log out and disconnect from the server using the `/disconnect` command.

### Rooms
- **Default Room**: All users join the `main` room upon logging in.
- **Create Room**: Create a new room with `/cr <room_name>` or `/create_room <room_name>`.
- **Switch Room**: Switch to an existing room with `/sr <room_name>` or `/switch_room <room_name>`.
- **Broadcast Messages**: Messages sent in a room are broadcast to all users in the same room.

### Private Messaging
- Send private messages to specific users using the `@username` prefix, followed by the message.
  - Example: `@JohnDoe Hello!`

### User and Room Listing
- Use `/list users` to see all registered users and their online/offline status.
- Use `/list rooms` to view all available chat rooms.

### Graceful Shutdown (Planned)
- The server can handle `Ctrl+C` signals for a clean shutdown.

### Robust Error Handling
- Prevents invalid operations (e.g., creating duplicate rooms or switching to non-existent rooms).
- Provides meaningful feedback for invalid commands.

## How to Run the Server

1. **Clone the Repository**:
   ```bash
   git clone <repository_url>
   cd chatroom
   ```

2. **Build the Project**:
   Ensure you have Rust installed. Then run:
   ```bash
   cargo build
   ```

3. **Run the Server**:
   Start the server with:
   ```bash
   cargo run
   ```
   The server will listen on `127.0.0.1:7878`.

4. **Connect Clients**:
   Use a tool like `telnet` or `nc` to connect:
   ```bash
   telnet 127.0.0.1 7878
   ```
   or
   ```bash
   nc 127.0.0.1 7878
   ```

## Usage Instructions

### Login Process
1. Upon connecting, you'll be prompted to:
   ```
   1. Sign up
   2. Sign in
   ```
2. Follow the instructions to register or log in.

### Commands
- `/list users`: View all users and their status.
- `/list rooms`: View available rooms.
- `/cr <room_name>`: Create a new room.
- `/sr <room_name>`: Switch to an existing room.
- `/q`: Log out without disconnecting.
- `/disconnect`: Log out and disconnect from the server.
- `@username <message>`: Send a private message to a user.

### General Messaging
- Type any message (not starting with `/` or `@`) to broadcast it to the current room.

## Implementation Details

- **Concurrency**: Uses threads and shared state management with `Arc` and `Mutex`.
- **Message Handling**:
  - Commands (starting with `/`) are parsed and executed.
  - Messages (not starting with `/` or `@`) are broadcasted to the current room.
  - Private messages (starting with `@`) are delivered to the specified user.
- **Persistence**: User data is stored in memory during the session.
- **Planned Enhancements**:
  - Persistent storage (e.g., database integration).
  - Graceful shutdown for all active threads.

## Example Interaction

**Client A** connects and signs up:
```
Welcome to the chat server!
1. Sign up
2. Sign in
Please choose (1|2): 1
Enter a username: Alice
Enter a password: ****
User 'Alice' registered successfully.
You have joined the 'main' room.
>
```

**Client B** connects and signs up:
```
Welcome to the chat server!
1. Sign up
2. Sign in
Please choose (1|2): 1
Enter a username: Bob
Enter a password: ****
User 'Bob' registered successfully.
You have joined the 'main' room.
>
```

**Client A** sends a private message:
```
> @Bob Hi Bob!
(Private) Message sent successfully.
>
```

**Client B** receives:
```
(Private) Alice: Hi Bob!
>
```

**Client A** switches to another room:
```
> /cr chatroom
Room 'chatroom' created.
> /sr chatroom
Switched to room 'chatroom'.
>
```

**Client B** lists rooms:
```
> /list rooms
Available Rooms:
main
chatroom
>
```

## Requirements

- **Rust**: Ensure Rust is installed. Visit [Rust's official website](https://www.rust-lang.org/) for installation instructions.
- **Telnet or Netcat**: Required for client connections.

## Contributing

Feel free to fork the repository, submit pull requests, or open issues for bug reports and feature suggestions.

## License

This project is open-source and available under the MIT License. See the `LICENSE` file for more details.