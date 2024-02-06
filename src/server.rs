use anyhow::{Context, Result};
use std::time::SystemTime;
use tonic::{transport::Server, Request, Response, Status};

pub mod api;

use api::grpc::message_storage::v1::message_storage_server::{MessageStorage, MessageStorageServer};
use api::grpc::message_storage::v1::{MessageRequest, MessageResponse};

#[derive(Debug, Default)]
pub struct MessageStorageService {}

#[tonic::async_trait]
impl MessageStorage for MessageStorageService {
    async fn send_message(
        &self,
        request: Request<MessageRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<MessageResponse>, Status> {
        println!("Got a request: {:?}", request);

        // Get the current time as duration from the UNIX epoch, 
        // then add the current time to the UNIX epoch to get back a SystemTime
        let now = SystemTime::UNIX_EPOCH
            + SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH) 
                .context("System time is less than unit epoch")
                .map_err(|e| Status::internal(format!("Internal error: {}", e)))?; 

        let reply = MessageResponse {
            timestamp: Some(prost_types::Timestamp::from(now)),
            id: 1, //TODO: Implement a proper ID
            new: true,// TODO: Implement new flag
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:50051".parse()?;
    let message_service = MessageStorageService::default();

    Server::builder()
        .add_service(MessageStorageServer::new(message_service))
        .serve(addr)
        .await?;

    Ok(())
}
