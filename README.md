# TCP Echo Server

This project demonstrates the evolution of a TCP echo server implementation in Rust, starting from a basic version and progressing to more advanced features.

## Project Structure

The project contains several versions of the TCP echo server, each building upon the previous implementation:

### Version 1 (`v1chat.rs`)
- Basic TCP echo server implementation
- Single-threaded synchronous I/O
- Simple echo functionality that sends back received messages
- Limited to one message per connection

### Version 2 (`v2(clientonly).rs`)
- Enhanced version with continuous message handling
- Added support for multiple messages per connection
- Implemented proper connection handling and error management
- Added "exit" command support
- Still synchronous but with better error handling

### Version 3 (`v3chat.rs`)
- Multi-threaded implementation
- Added bidirectional communication
- Server operator can send messages to clients
- Uses `Arc<Mutex<>>` for thread-safe stream sharing
- Implemented message size limits
- Better error handling and connection management

### Main Version (`main.rs`)
- Asynchronous implementation using Tokio
- Most advanced version with async/await pattern
- Improved error handling and resource management
- Uses broadcast channels for shutdown coordination
- Better separation of concerns between client and server handling
- More robust message handling with proper buffering

## Features

- TCP server listening on localhost:7878
- Echo functionality (sends back received messages)
- Bidirectional communication
- Message size limits
- Graceful connection handling
- Error handling and logging
- Support for exit commands
- Thread-safe operations

## Requirements

- Rust (latest stable version)
- Tokio runtime (for the main version)

## Running the Server

To run any version of the server:

```bash
cargo run --bin <version_name>
```

For example:
```bash
cargo run --bin main
```

## Client Connection

You can connect to the server using any TCP client, such as `netcat`:

```bash
nc localhost 7878
```

## Notes

- The server listens on `127.0.0.1:7878`
- Type "exit" to close the connection
- Messages are limited to 1024 bytes in the later versions
- The main version (main.rs) is the most feature-complete and recommended implementation 