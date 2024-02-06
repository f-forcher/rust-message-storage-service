use tonic::{transport::Server, Request, Response, Status};

use v1::message_storage::message_storage_server::{MessageStorage, MessageStorageServer};
use v1::message_storage::{MessageResponse, MessageRequest};

pub mod v1 {
    pub mod message_storage {
        tonic::include_proto!("v1.message_storage"); // The string specified here must match the proto package name
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}
