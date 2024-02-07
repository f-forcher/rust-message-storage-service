use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use api::grpc::message_storage;
use message_storage::v1::message_storage_server::MessageStorage;
use message_storage::v1::{MessageRequest, MessageResponse};

pub mod api;
#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct KeyAndTenant {
    key: String,
    tenant: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct IdAndCount {
    pub id: u64,
    pub count: u64,
}

impl KeyAndTenant {
    pub fn try_from_parts(key: &str, tenant: &str) -> Result<Self> {
        let valid_key_pattern = r"^K-[a-z0-9]{5}-[A-Z]$";
        let valid_key_regex = Regex::new(valid_key_pattern)
            .with_context(|| format!("Wrong key regex: {valid_key_pattern}"))?;

        if !valid_key_regex.is_match(key) {
            return Err(anyhow!("Key is wrong: {}", key));
        }

        Ok(Self {
            key: key.to_owned(),
            tenant: tenant.to_owned(),
        })
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn tenant(&self) -> &str {
        &self.tenant
    }
}

#[derive(Debug, Default)]
pub struct MessageStorageService {
    message_store: RwLock<HashMap<KeyAndTenant, IdAndCount>>,
}

#[tonic::async_trait]
impl MessageStorage for MessageStorageService {
    async fn send_message(
        &self,
        request: Request<MessageRequest>,
    ) -> Result<Response<MessageResponse>, Status> {
        println!("Got a request: {request:?}");

        // Get the current time as duration from the UNIX epoch,
        // then add the current time to the UNIX epoch to get back a SystemTime
        let now = SystemTime::UNIX_EPOCH
            + SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .context("System time is less than unit epoch")
                .map_err(|e| Status::internal(format!("Internal error: {e}")))?;

        let request = request.into_inner();
        let key_and_tenant = KeyAndTenant::try_from_parts(&request.key, &request.tenant)
            .map_err(|e| Status::invalid_argument(format!("{e}")))?;

        let id_and_count = self
            .message_store
            .read()
            .await
            .get(&key_and_tenant)
            .unwrap_or(&IdAndCount {
                id: self.message_store.read().await.len() as u64 + 1,
                count: 0,
            });
        let is_new = id_and_count.count == 0;
        self.message_store
            .insert(key_and_tenant, how_many_already + 1);

        let reply = MessageResponse {
            timestamp: Some(prost_types::Timestamp::from(now)),
            id: 1,       //TODO: Implement a proper ID
            new: is_new, // TODO: Implement new flag
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}
