// src/lib.rs

// Intention:
// Defines the library part of the crate. This allows modules like api_models
// and api_client to be shared between the main binary (src/main.rs) and
// integration tests (tests/).

// Declare and make modules public so they can be used by main.rs and tests.
pub mod api_client;
pub mod api_models;

// Re-export key items for easier use (optional but good practice)
pub use api_client::*;
pub use api_models::*;
