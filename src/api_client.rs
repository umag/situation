// src/api_client.rs

// Intention:
// Provides functions to interact with the Luminork/Systeminit API.
// Currently contains placeholder implementations until the actual API client
// (e.g., using luminork crate or reqwest) is determined and implemented.

// Design Choices:
// - Functions are async using `reqwest` for network calls.
// - Returns `Result` to handle potential errors (env var loading, network, deserialization, API errors).
// - Uses the data structures defined in `api_models.rs`.
// - Loads API URL and JWT token from environment variables using `dotenvy`.
// - Creates a reusable `reqwest::Client`.

use crate::api_models::WhoamiResponse;
use dotenvy::dotenv;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use crate::api_models::ApiError; // Import ApiError for potential future parsing
use std::env;
use std::error::Error;
use std::sync::OnceLock; // Use OnceLock for lazy static initialization

// Intention: Lazily initialize the reqwest client and load env vars once.
// Design Choice: Use OnceLock for thread-safe, one-time initialization.
// Stores the API base URL, JWT token, and the reqwest client.
struct ApiConfig {
    client: reqwest::Client,
    base_url: String,
    jwt_token: String,
}

static API_CONFIG: OnceLock<Result<ApiConfig, Box<dyn Error + Send + Sync>>> = OnceLock::new();

fn get_api_config() -> Result<&'static ApiConfig, &'static (dyn Error + Send + Sync)> {
    API_CONFIG.get_or_init(|| {
        dotenv().ok(); // Load .env file, ignore errors if it doesn't exist

        let base_url = env::var("SI_API").map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        let jwt_token = env::var("JWT_TOKEN").map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let mut headers = HeaderMap::new();
        let mut auth_value = HeaderValue::from_str(&format!("Bearer {}", jwt_token))
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        auth_value.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth_value);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        Ok(ApiConfig { client, base_url, jwt_token }) // jwt_token stored for potential future use/refresh
    })
    .as_ref()
    .map_err(|e| &**e) // Convert Box<dyn Error> to &dyn Error
}


/// Fetches user information from the `/whoami` endpoint.
/// Intention: Calls the actual `/whoami` API endpoint using configuration from `.env`.
/// Design: Uses the initialized `reqwest::Client` and constructs the URL.
///         Sends a GET request and deserializes the JSON response into `WhoamiResponse`.
/// Verification (2025-04-21):
///   - Confirmed endpoint (`GET /whoami`) matches OpenAPI spec and luminork service impl.
///   - Confirmed Bearer token authentication matches expectations.
///   - Confirmed `WhoamiResponse` struct in `api_models.rs` matches the actual runtime response structure
///     (Note: `token` field is an object, differing from OpenAPI spec/service code which suggested string).
/// Returns: A tuple containing the `WhoamiResponse` on success and a `Vec<String>` of log messages.
pub async fn whoami() -> Result<(WhoamiResponse, Vec<String>), Box<dyn Error + Send + Sync>> {
    let mut logs = Vec::new();
    let config = get_api_config()?; // Get or initialize config

    let url = format!("{}/whoami", config.base_url);
    logs.push(format!("Calling API: GET {}", url));

    let response = config.client.get(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        // Attempt to deserialize the successful response
        let response_text = response.text().await?; // Read body first for logging
        logs.push(format!("API Success Body: {}", response_text));
        let whoami_data: WhoamiResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to deserialize success response: {} - Body: {}", e, response_text))?;
        // logs.push(format!("API Response Data Parsed: {:?}", whoami_data)); // Maybe too verbose?
        Ok((whoami_data, logs))
    } else {
        // Verification (2025-04-21):
        // - OpenAPI spec lists 401/403 for /whoami but doesn't explicitly link ApiError schema.
        // - Luminork service code doesn't show explicit ApiError construction for these statuses.
        // - Current generic error handling (status + text body) is acceptable.
        // TODO: Consider attempting to parse the error body as `ApiError` in the future
        //       if the API guarantees that structure for 4xx/5xx errors.
        let error_text = response.text().await.unwrap_or_else(|_| "Failed to read error body".to_string());
        logs.push(format!("API Error Body: {}", error_text));
        // Attempt to parse as ApiError for more structured logging, but fall back
        let error_message = match serde_json::from_str::<ApiError>(&error_text) {
             Ok(api_error) => format!("API request failed with status {}: Code {:?}, Message: {}", status, api_error.code, api_error.message),
             Err(_) => format!("API request failed with status {}: {}", status, error_text),
        };
        Err(error_message.into()) // Return the error message, logs are not returned on error path
    }
}

// TODO: Add placeholder functions for other API endpoints as needed.
// Examples:
// pub async fn list_change_sets(workspace_id: &str) -> Result<ListChangeSetV1Response, Box<dyn Error + Send + Sync>> { ... }
// pub async fn create_change_set(workspace_id: &str, name: &str) -> Result<CreateChangeSetV1Response, Box<dyn Error + Send + Sync>> { ... }

// Note: The error type `Box<dyn Error + Send + Sync>` is used for flexibility,
// allowing different error types (IO, HTTP, API errors) to be returned.
// Consider defining a custom error enum for more specific error handling later.
