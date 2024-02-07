use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
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

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct MessageId(u64);

impl KeyAndTenant {
    pub fn try_from_parts(key: &str, tenant: &str) -> Result<Self> {
        let valid_key_pattern = r"^K-[a-z0-9]{5}-[A-Z]$";
        let valid_key_regex = Regex::new(valid_key_pattern)
            .with_context(|| format!("Wrong key regex: {valid_key_pattern}"))?;

        if !valid_key_regex.is_match(key) {
            return Err(anyhow!("Wrong key format: {}", key));
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
    message_store: Arc<Mutex<HashMap<KeyAndTenant, MessageId>>>,
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

        let (id, is_new) = {
            let mut store = self
                .message_store
                .lock()
                .map_err(|e| Status::internal(format!("Error acquiring the lock: {e}")))?;

            if let Some(id) = store.get(&key_and_tenant) {
                (id.0, false)
            } else {
                let id = MessageId(store.len() as u64 + 1);
                let old_val = store.insert(key_and_tenant, id);
                assert!(old_val.is_none());
                (id.0, true)
            }
        };

        // .unwrap_or(&MessageId(store.len() as u64))
        let reply = MessageResponse {
            timestamp: Some(prost_types::Timestamp::from(now)),
            id,
            new: is_new,
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}
