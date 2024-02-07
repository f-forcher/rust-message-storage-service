use message_storage::v1::message_storage_client::MessageStorageClient;
use message_storage::v1::MessageRequest;
use rust_message_storage_service::api::grpc::message_storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MessageStorageClient::connect("http://127.0.0.1:50051").await?;

    let request = tonic::Request::new(MessageRequest {
        key: "Wrongo!".into(),
        tenant: "tenant".into(),
    });

    let response = client.send_message(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
