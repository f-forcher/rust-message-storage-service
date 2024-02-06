use anyhow::{Context, Result};
use std::time::SystemTime;
use tonic::{transport::Server, Request, Response, Status};

use proto::message_storage::v1::message_storage_server::{MessageStorage, MessageStorageServer};
use proto::message_storage::v1::{MessageRequest, MessageResponse};

pub mod proto;

#[derive(Debug, Default)]
pub struct MessageStorageService {}

#[tonic::async_trait]
impl MessageStorage for MessageStorageService {
    async fn send_message(
        &self,
        request: Request<MessageRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<MessageResponse>, Status> {
        // Return an instance of type HelloReply
        println!("Got a request: {:?}", request);

        let now = SystemTime::UNIX_EPOCH
            + SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH) // Get the current time
                .context("System time is less than unit epoch")
                .map_err(|e| Status::internal(format!("Internal error: {}", e)))?; // Add the current time to the UNIX epoch

        let reply = MessageResponse {
            timestamp: Some(prost_types::Timestamp::from(now)),
            id: 0, //TODO: Implement a proper ID
            new: true,// TODO: Implement new flag
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "[::1]:50051".parse()?;
    let message_service = MessageStorageService::default();

    Server::builder()
        .add_service(MessageStorageServer::new(message_service))
        .serve(addr)
        .await?;

    Ok(())
}
