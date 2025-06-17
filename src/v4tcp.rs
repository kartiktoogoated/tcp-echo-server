use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, tcp::{OwnedReadHalf, OwnedWriteHalf}};
use tokio::sync::{Mutex, broadcast};
use std::sync::Arc;

const MAX_MESSAGE_SIZE: usize = 1024;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    println!("Chat server listening on 127.0.0.1:7878");

    let (shutdown_tx, _) = broadcast::channel(1);

    loop {
        let (stream, _) = listener.accept().await?;
        println!("Client connected");

        let (read_half, write_half) = stream.into_split();
        let write_half = Arc::new(Mutex::new(write_half));

        let shutdown_rx = shutdown_tx.subscribe();
        let write_half_clone = Arc::clone(&write_half);

        let client_task = tokio::spawn(handle_client(read_half, write_half_clone, shutdown_rx));
        let shutdown_tx_clone = shutdown_tx.clone();
        let server_task = tokio::spawn(handle_server(write_half, shutdown_tx_clone));

        let _ = client_task.await;
        let _ = server_task.await;

        println!("Connection closed. Waiting for new client...");
    }
}

async fn handle_client(
    read_half: OwnedReadHalf,
    write_half: Arc<Mutex<OwnedWriteHalf>>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let mut reader = BufReader::new(read_half);
    let mut buffer = String::new();

    loop {
        tokio::select! {
            res = reader.read_line(&mut buffer) => {
                match res {
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
                            println!("Client sent exit. Closing connection.");
                            break;
                        }

                        println!("Client sent: {}", message);

                        let mut writer = write_half.lock().await;
                        let response = format!("You said: {}\n", message);
                        if let Err(e) = writer.write_all(response.as_bytes()).await {
                            eprintln!("Error sending to client: {}", e);
                            break;
                        }
                        buffer.clear();
                    }
                    Err(e) => {
                        eprintln!("Error reading from client: {}", e);
                        break;
                    }
                }
            }

            _ = shutdown_rx.recv() => {
                println!("Client handler received shutdown signal.");
                break;
            }
        }
    }
}

async fn handle_server(
    write_half: Arc<Mutex<OwnedWriteHalf>>,
    shutdown_tx: broadcast::Sender<()>,
) {
    let stdin = io::stdin();
    let mut lines = BufReader::new(stdin).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        if line.eq_ignore_ascii_case("exit") {
            println!("Server exiting connection.");
            let _ = shutdown_tx.send(()); // Signal shutdown to all listeners
            break;
        }

        println!("Server operator sent: {}", line);

        let mut writer = write_half.lock().await;
        let response = format!("Server says: {}\n", line);
        if let Err(e) = writer.write_all(response.as_bytes()).await {
            eprintln!("Error sending to client: {}", e);
            break;
        }
    }
}
