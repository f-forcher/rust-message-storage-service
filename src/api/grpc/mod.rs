pub mod message_storage {
    pub mod v1 {
        #![allow(unknown_lints)]
        #![allow(clippy::all, clippy::pedantic, clippy::nursery)]
        tonic::include_proto!("message_storage.v1"); // The string specified here must match the proto package name
    }
}
