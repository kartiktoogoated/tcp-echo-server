use::std::io::{Read, Write};
use::std::net::TcpListener;

fn main() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Echo server listening on 127.0.0.1:7878");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Connection established");

                let mut buffer = [0; 512];

                loop {
                    // Try reading from the stream
                    match stream.read(&mut buffer) {
                        Ok(0) => {
                            // 0 bytes means client disconnected
                            println!("Client disconnected");
                            break;
                        }
                        Ok(n) => {
                            let received_message = String::from_utf8_lossy(&mut buffer[..n]).trim().to_string();
                            println!("Received: {}", received_message);

                            if received_message.eq_ignore_ascii_case("exit") {
                                println!("Exit command received. Closing connection");
                                break;
                            }

                            if let Err(e) = stream.write_all(received_message.as_bytes()) {
                                eprintln!("failed reading from client: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading from client: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
    Ok(())
 }