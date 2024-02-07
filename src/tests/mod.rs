use std::future::Future;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Channel, Endpoint, Server};
use tonic::Request;

use crate::api::grpc::message_storage::v1::{
    message_storage_client::MessageStorageClient, message_storage_server::MessageStorageServer,
    MessageRequest,
};
use crate::MessageStorageService;

async fn server_and_client() -> (impl Future<Output = ()>, MessageStorageClient<Channel>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let stream = TcpListenerStream::new(listener);

    let serve_future = async {
        let result = Server::builder()
            .add_service(MessageStorageServer::new(MessageStorageService::default()))
            .serve_with_incoming(stream)
            .await;
        assert!(result.is_ok());
    };

    let channel = Endpoint::try_from(format!("http://{addr}"))
        .unwrap()
        .connect_lazy();

    let client = MessageStorageClient::new(channel);

    (serve_future, client)
}

#[tokio::test]
async fn err_wrong_key() {
    let (serve_future, mut client) = server_and_client().await;

    let request_future = async {
        let response = client
            .send_message(Request::new(MessageRequest {
                key: "Wrongo!".to_string(),
                tenant: "tenant".to_string(),
            }))
            .await
            .unwrap_err();

        insta::assert_debug_snapshot!(
                response.message(),
                @r###""Key is wrong: Wrongo!""###);
    };

    // Wait for completion
    tokio::select! {
        () = serve_future => panic!("Server returned first"),
        () = request_future => (),
    }
}
