use::std::io::{Read, Write};
use::std::net::TcpListener;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Echo server listening to 127.0.0.1:7878");

    for stream in listener.incoming() {
        let mut stream = stream?;
        println!("Connection established");

        let mut buffer = [0; 512];
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            println!("Client closed the connection");
            continue;
        }
        
        // Print the received message
        let received_message = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received {}", received_message);

        // Echo it back
        stream.write_all(&buffer[..bytes_read])?;
        println!("Message echoed back to client: {}", received_message);
    }
    Ok(())
}