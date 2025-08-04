// Re-export the generated protobuf code
pub mod sapphillon {
    pub mod v1 {
        include!("proto_generated/sapphillon.v1.rs");
    }
}

// Convenient re-exports for easier access
pub use sapphillon::v1::*;