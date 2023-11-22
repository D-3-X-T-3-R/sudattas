mod grpc_services;
pub use grpc_services::*;

pub mod proto {
    pub mod core {
        include!("grpc_services.rs");
    }
}