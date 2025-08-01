use chrono::Local;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock, broadcast};

const MAX_MESSAGE_SIZE: usize = 1024;
static CLIENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

type ClientWriter = Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>;
type SharedClients = Arc<RwLock<Vec<(String, ClientWriter)>>>;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    println!("[{}] Chat server listening on 127.0.0.1:7878", timestamp());

    let clients: SharedClients = Arc::new(RwLock::new(Vec::new()));
    let (shutdown_tx, _) = broadcast::channel::<()>(32);

    {
        let clients = clients.clone();
        let shutdown_tx = shutdown_tx.clone();
        tokio::spawn(async move {
            handle_server_input(clients, shutdown_tx).await;
        });
    }

    loop {
        let (stream, addr) = listener.accept().await?;
        let id = CLIENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let client_name = format!("Client {}", id);

        println!(
            "[{}] New client connected: {} from {}",
            timestamp(),
            client_name,
            addr
        );

        let shutdown_rx = shutdown_tx.subscribe();
        let clients = clients.clone();
        tokio::spawn(handle_connection(stream, shutdown_rx, clients, client_name));
    }
}

async fn handle_connection(
    stream: TcpStream,
    mut shutdown_rx: broadcast::Receiver<()>,
    clients: SharedClients,
    client_name: String,
) {
    let (read_half, write_half) = stream.into_split();
    let writer = Arc::new(Mutex::new(write_half));
    {
        let mut all = clients.write().await;
        all.push((client_name.clone(), writer.clone()));
    }

    let mut reader = BufReader::new(read_half);
    let mut buffer = String::new();

    loop {
        tokio::select! {
            res = reader.read_line(&mut buffer) => {
                match res {
                    Ok(0) => break, // client disconnected
                    Ok(_) => {
                        let message = buffer.trim().to_string();
                        buffer.clear();

                        if message.len() > MAX_MESSAGE_SIZE {
                            eprintln!("[{}] Message too long from {}. Disconnecting.", timestamp(), client_name);
                            break;
                        }

                        println!("[{}] {} says: {}", timestamp(), client_name, message);

                        if message.eq_ignore_ascii_case("exit") {
                            println!("[{}] {} requested exit.", timestamp(), client_name);
                            break;
                        }

                        // Send back to self
                        let response = format!("You said: {}\n", message);
                        let mut writer_self = writer.lock().await;
                        if let Err(e) = writer_self.write_all(response.as_bytes()).await {
                            eprintln!("[{}] Failed to write to {}: {}", timestamp(), client_name, e);
                            break;
                        }

                        // Broadcast to others
                        let broadcast_message = format!("{} says: {}\n", client_name, message);
                        let all = clients.read().await;
                        for (name, client_writer) in all.iter() {
                            if name == &client_name {
                                continue;
                            }
                            let mut writer = client_writer.lock().await;
                            if let Err(e) = writer.write_all(broadcast_message.as_bytes()).await {
                                eprintln!("[{}] Failed to send to {}: {}", timestamp(), name, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[{}] Error reading from {}: {}", timestamp(), client_name, e);
                        break;
                    }
                }
            }

            _ = shutdown_rx.recv() => {
                println!("[{}] Shutdown received. Closing session for {}", timestamp(), client_name);
                break;
            }
        }
    }

    {
        let mut all = clients.write().await;
        all.retain(|(name, w)| name != &client_name || !Arc::ptr_eq(w, &writer));
    }

    println!("[{}] {} disconnected.", timestamp(), client_name);
}

async fn handle_server_input(clients: SharedClients, shutdown_tx: broadcast::Sender<()>) {
    let stdin = io::stdin();
    let mut lines = BufReader::new(stdin).lines();

    println!(
        "[{}] Type messages to broadcast, or 'exit' to shutdown server.",
        timestamp()
    );

    while let Ok(Some(line)) = lines.next_line().await {
        if line.eq_ignore_ascii_case("exit") {
            println!("[{}] Server shutting down...", timestamp());
            let _ = shutdown_tx.send(());
            break;
        }

        println!("[{}] Server broadcast: {}", timestamp(), line);
        let message = format!("Server says: {}\n", line);

        let mut failed_clients = vec![];

        {
            let all = clients.read().await;
            for (i, (_name, client)) in all.iter().enumerate() {
                let mut writer = client.lock().await;
                if let Err(_) = writer.write_all(message.as_bytes()).await {
                    failed_clients.push(i);
                }
            }
        }

        if !failed_clients.is_empty() {
            let mut all = clients.write().await;
            for &i in failed_clients.iter().rev() {
                all.remove(i);
            }
        }
    }
}

fn timestamp() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
