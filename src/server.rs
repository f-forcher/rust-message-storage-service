use anyhow::Result;
use rust_message_storage_service::MessageStorageService;
use tonic::transport::Server;

use message_storage::v1::message_storage_server::MessageStorageServer;
use rust_message_storage_service::api::grpc::message_storage;


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
