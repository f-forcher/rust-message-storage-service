# rust-message-storage-service
Proof of concept of a small message storage service in rust.

# Run

### Server
To run the server:
```
cargo run --bin message-storage-server
```

The server will run at `localhost:50051`.

### Client
To run the client and send a standard message
```
cargo run --bin message-storage-client
```

### Tests
To run the tests:
```
cargo test
```
