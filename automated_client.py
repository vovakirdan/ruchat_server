import socket
import time
import random
import string

def automated_client(host, port, username, password, message):
    try:
        # Create a socket and connect to the chat server
        client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        client_socket.connect((host, port))
        print(f"Connected to {host}:{port}")
        
        # Read initial welcome messages
        time.sleep(1)
        welcome = client_socket.recv(1024).decode('utf-8')
        print(welcome)
        
        # Choose Sign-Up option (1)
        client_socket.sendall(b"1\n")
        time.sleep(1)
        signup_prompt = client_socket.recv(1024).decode('utf-8')
        print(signup_prompt)
        
        # Send username
        client_socket.sendall(f"{username}\n".encode('utf-8'))
        time.sleep(1)
        username_prompt = client_socket.recv(1024).decode('utf-8')
        print(username_prompt)
        
        # Send password
        client_socket.sendall(f"{password}\n".encode('utf-8'))
        time.sleep(1)
        password_response = client_socket.recv(1024).decode('utf-8')
        print(password_response)
        
        # Wait for the user to enter the chat
        time.sleep(1)
        chat_welcome = client_socket.recv(1024).decode('utf-8')
        print(chat_welcome)
        
        # Send a message
        # client_socket.sendall(f"{message}\n".encode('utf-8'))
        # time.sleep(1)
        # message_response = client_socket.recv(1024).decode('utf-8')
        # print(message_response)
        while True:
            message = input()
            client_socket.sendall(f"{message}\n".encode('utf-8'))
            time.sleep(1)
            message_response = client_socket.recv(1024).decode('utf-8')
            print(message_response)
        # Close the connection
        # client_socket.close()
        print("Disconnected from the server.")

    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    # Configuration
    HOST = "127.0.0.1"  # Chat server IP
    PORT = 7878         # Chat server port
    USERNAME = ''.join([random.choice(string.ascii_letters) for _ in range(random.randint(3, 10))])  # "testuser"
    PASSWORD = ''.join([random.choice(string.ascii_letters) for _ in range(random.randint(3, 10))])  # "testpass"
    MESSAGE = "Hello, this is a test message!"
    
    automated_client(HOST, PORT, USERNAME, PASSWORD, MESSAGE)
