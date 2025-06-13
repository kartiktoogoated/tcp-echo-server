use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

const MAX_MESSAGE_SIZE: usize = 1024; // 1KB per message

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Chat server listening on 127.0.0.1:7878");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Client connected");

                // Wrap the stream in Arc<Mutex<>> so multiple threads can access it
                let stream = Arc::new(Mutex::new(stream.try_clone()?));

                // Client reading thread
                let client_stream = Arc::clone(&stream);
                let client_thread = thread::spawn(move || {
                    let mut reader = BufReader::new(client_stream.lock().unwrap().try_clone().unwrap());
                    let mut buffer = String::new();

                    loop {
                        buffer.clear(); // Clear the buffer before reading a new message

                        match reader.read_line(&mut buffer) {
                            Ok(0) => {
                                println!("Client disconnected");
                                break;
                            }
                            Ok(_) => {
                                if buffer.len() > MAX_MESSAGE_SIZE {
                                    eprintln!("Message too long! Disconnecting client.");
                                    break;
                                }

                                let message = buffer.trim();
                                if message.eq_ignore_ascii_case("exit") {
                                    println!("Client sent exit. Closing connection");
                                    break;
                                }

                                println!("Client sent: {}", message);

                                // Send message back to client
                                let mut locked_stream = client_stream.lock().unwrap();
                                let response = format!("You said: {}\n", message);
                                if let Err(e) = locked_stream.write_all(response.as_bytes()) {
                                    eprintln!("Error sending to client: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Error reading from client: {}", e);
                                break;
                            }
                        }
                    }
                });

                // Server operator thread
                let server_stream = Arc::clone(&stream);
                let server_thread = thread::spawn(move || {
                    let stdin = io::stdin();
                    for line in stdin.lock().lines() {
                        let line = line.unwrap();
                        if line.eq_ignore_ascii_case("exit") {
                            println!("Server exiting connection");
                            break;
                        }

                        println!("Server operator sent: {}", line);

                        let mut locked_stream = server_stream.lock().unwrap();
                        let response = format!("Server says: {}\n", line);
                        if let Err(e) = locked_stream.write_all(response.as_bytes()) {
                            eprintln!("Error sending to client: {}", e);
                            break;
                        }
                    }
                });

                // Wait for both threads to finish
                let _ = client_thread.join();
                let _ = server_thread.join();

                println!("Connection closed. Waiting for new client...");
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }

    Ok(())
}
