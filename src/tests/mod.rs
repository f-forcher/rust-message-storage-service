use std::future::Future;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Channel, Endpoint, Server};
use tonic::Request;
use futures::future::{FutureExt, Shared};

use crate::api::grpc::message_storage::v1::{
    message_storage_client::MessageStorageClient, message_storage_server::MessageStorageServer,
    MessageRequest,
};
use crate::MessageStorageService;


/// Create a server and a number of clients
async fn server_and_clients(num_clients: u32) -> (Shared<impl Future<Output = ()>>, Vec<MessageStorageClient<Channel>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let stream = TcpListenerStream::new(listener);

    let serve_future = async {
        let result = Server::builder()
            .add_service(MessageStorageServer::new(MessageStorageService::default()))
            .serve_with_incoming(stream)
            .await;
        assert!(result.is_ok());
    }.shared();

    let channel = Endpoint::try_from(format!("http://{addr}"))
        .unwrap()
        .connect_lazy();

    let clients: Vec<_> = (0..num_clients)
        .map(|_| MessageStorageClient::new(channel.clone()))
        .collect();

    (serve_future, clients)
}

#[tokio::test]
async fn get_message_simple() {
    let (serve_future, mut clients) = server_and_clients(1).await;
    let client = &mut clients.pop().unwrap();

    let request_future = async {
        let response = client
            .send_message(Request::new(MessageRequest {
                key: "K-4bbf1-P".to_string(),
                tenant: "tenant".to_string(),
            }))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(response.id, 1);
        assert_eq!(response.new, true);
    };

    // Wait for completion
    tokio::select! {
        () = serve_future => panic!("Server returned first"),
        () = request_future => (),
    }
}

#[tokio::test]
async fn err_wrong_key() {
    let (serve_future, mut clients) = server_and_clients(1).await;
    let client = &mut clients[0];

    let request_future = async {
        let response = client
            .send_message(Request::new(MessageRequest {
                key: "Wrong!".to_string(),
                tenant: "tenant".to_string(),
            }))
            .await
            .unwrap_err();

        insta::assert_debug_snapshot!(
                response.message(),
                @r###""Wrong key format: Wrong!""###);
    };

    // Wait for completion
    tokio::select! {
        () = serve_future => panic!("Server returned first"),
        () = request_future => (),
    }
}

#[tokio::test]
async fn get_multiple_messages() {
    let (serve_future, mut clients) = server_and_clients(2).await;
    let (first, second) = match &mut clients[0..2] {
        [first, second] => (first, second),
        _ => panic!("Expected 2 clients"),
    };

    let r1 = async {
        let response = first
            .send_message(Request::new(MessageRequest {
                key: "K-h53dk-A".to_string(),
                tenant: "3bd1c697".to_string(),
            }))
            .await
            .unwrap()
            .into_inner();

        // First message, new id, is a new message
        assert_eq!(response.id, 1);
        assert_eq!(response.new, true);
    };
    // Wait for completion
    tokio::select! {
        () = serve_future.clone() => panic!("Server returned first"),
        () = r1 => (),
    }

    let r2 = async {
        let response = second
            .send_message(Request::new(MessageRequest {
                key: "K-h53dk-A".to_string(),
                tenant: "75682017".to_string(),
            }))
            .await
            .unwrap()
            .into_inner();

        // New tenant and same key, get new id
        assert_eq!(response.id, 2);
        assert_eq!(response.new, true);
    };
    tokio::select! {
        () = serve_future.clone() => panic!("Server returned first"),
        () = r2 => (),
    }

    let r3 = async {
        let response = first
            .send_message(Request::new(MessageRequest {
                key: "K-867vc-C".to_string(),
                tenant: "3bd1c697".to_string(),
            }))
            .await
            .unwrap()
            .into_inner();

        // New key and same tenant, get new id
        assert_eq!(response.id, 3);
        assert_eq!(response.new, true);
    };
    tokio::select! {
        () = serve_future.clone() => panic!("Server returned first"),
        () = r3 => (),
    }

    let r4 = async {
        let response = second
            .send_message(Request::new(MessageRequest {
                key: "K-h53dk-A".to_string(),
                tenant: "75682017".to_string(),
            }))
            .await
            .unwrap()
            .into_inner();

        // Same tenant and key, get same id
        assert_eq!(response.id, 2);
        assert_eq!(response.new, false);
    };
    tokio::select! {
        () = serve_future.clone() => panic!("Server returned first"),
        () = r4 => (),
    }
}

