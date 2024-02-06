use tonic::{transport::Server, Request, Response, Status};

use message_storage::v1::message_storage_server::{MessageStorage, MessageStorageServer};
use message_storage::v1::{MessageRequest, MessageResponse};

pub mod message_storage {
    pub mod v1 {
        tonic::include_proto!("message_storage.v1"); // The string specified here must match the proto package name
    }
}
