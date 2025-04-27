// src/api_client/mod.rs

// Intention:
// Defines shared configuration and utilities for API client functions.
// Declares modules for individual API endpoint functions.

// Design Choices:
// - Centralizes API configuration (URL, token, client) using OnceLock for lazy initialization.
// - Provides a common `get_api_config` function for all endpoint modules.
// - Re-exports functions from submodules to maintain a consistent external API.

use std::{
    env,
    error::Error,
    sync::OnceLock,
};

use dotenvy::dotenv;
use reqwest::header::{
    AUTHORIZATION,
    HeaderMap,
    HeaderValue,
};

// Make ApiError accessible within this module and its children
pub(crate) use crate::api_models::ApiError;

// Declare modules for each API function
pub mod abandon_change_set;
pub mod create_change_set;
pub mod create_component;
pub mod delete_component;
pub mod force_apply;
pub mod get_change_set;
pub mod get_component;
pub mod get_merge_status;
pub mod list_change_sets;
pub mod list_components; // Added module declaration
pub mod list_schemas; // Added module declaration
pub mod update_component;
pub mod whoami;

// Re-export functions from submodules
pub use abandon_change_set::abandon_change_set;
pub use create_change_set::create_change_set;
pub use create_component::create_component;
pub use delete_component::delete_component;
pub use force_apply::force_apply;
pub use get_change_set::get_change_set;
pub use get_component::get_component;
pub use get_merge_status::get_merge_status;
pub use list_change_sets::list_change_sets;
pub use list_components::list_components; // Added function re-export
pub use list_schemas::list_schemas; // Added function re-export
pub use update_component::update_component;
pub use whoami::whoami;

// --- Shared Configuration Logic ---

// Intention: Lazily initialize the reqwest client and load env vars once.
// Design Choice: Use OnceLock for thread-safe, one-time initialization.
// Stores the API base URL, JWT token, and the reqwest client.
// Made fields pub(crate) so they are accessible within the api_client module.
pub(crate) struct ApiConfig {
    client: reqwest::Client,
    base_url: String,
    jwt_token: String, // Keep for potential future use/refresh
}

static API_CONFIG: OnceLock<Result<ApiConfig, Box<dyn Error + Send + Sync>>> =
    OnceLock::new();

// Helper function to create a config instance. Used by get_api_config.
// Kept private to this module.
fn create_new_api_config() -> Result<ApiConfig, Box<dyn Error + Send + Sync>> {
    dotenv().ok(); // Load .env file, ignore errors if it doesn't exist

    let base_url = env::var("SI_API")
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
    let jwt_token = env::var("JWT_TOKEN")
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

    let mut headers = HeaderMap::new();
    let mut auth_value =
        HeaderValue::from_str(&format!("Bearer {}", jwt_token))
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
    auth_value.set_sensitive(true);
    headers.insert(AUTHORIZATION, auth_value);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

    Ok(ApiConfig {
        client,
        base_url,
        jwt_token,
    })
}

// Provides access to the initialized ApiConfig.
// Made pub(crate) for use by submodule functions.
pub(crate) fn get_api_config()
-> Result<&'static ApiConfig, &'static (dyn Error + Send + Sync)> {
    API_CONFIG
        .get_or_init(create_new_api_config)
        .as_ref()
        .map_err(|e| &**e) // Convert Box<dyn Error> to &dyn Error
}
