// src/api_client.rs

// Intention:
// Provides functions to interact with the Luminork/Systeminit API.
// Currently contains placeholder implementations until the actual API client
// Handles API interactions.

// Design Choices:
// - Functions are async using `reqwest` for network calls.
// - Returns `Result` to handle potential errors (env var loading, network, deserialization, API errors).
// - Uses the data structures defined in `api_models.rs`.
// - Loads API URL and JWT token from environment variables using `dotenvy`.
// - Creates a reusable `reqwest::Client` initialized lazily via `OnceLock`.
// - Returns results containing both the data and logs for TUI display.

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

use crate::api_models::{
    ApiError,
    CreateChangeSetV1Request,
    CreateChangeSetV1Response,
    // Component Models
    CreateComponentV1Request,
    CreateComponentV1Response,
    DeleteChangeSetV1Response,
    DeleteComponentV1Response,
    GetChangeSetV1Response,
    GetComponentV1Response,
    ListChangeSetV1Response,
    MergeStatusV1Response,
    UpdateComponentV1Request,
    UpdateComponentV1Response,
    WhoamiResponse,
}; // Use OnceLock for lazy static initialization

// Intention: Lazily initialize the reqwest client and load env vars once.
// Design Choice: Use OnceLock for thread-safe, one-time initialization.
// Stores the API base URL, JWT token, and the reqwest client.
struct ApiConfig {
    client: reqwest::Client,
    base_url: String,
    jwt_token: String,
}

static API_CONFIG: OnceLock<Result<ApiConfig, Box<dyn Error + Send + Sync>>> = OnceLock::new();

// Helper function to create a config instance. Used by get_api_config.
fn create_new_api_config() -> Result<ApiConfig, Box<dyn Error + Send + Sync>> {
    dotenv().ok(); // Load .env file, ignore errors if it doesn't exist

    let base_url = env::var("SI_API").map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
    let jwt_token =
        env::var("JWT_TOKEN").map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

    let mut headers = HeaderMap::new();
    let mut auth_value = HeaderValue::from_str(&format!("Bearer {}", jwt_token))
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
    }) // jwt_token stored for potential future use/refresh
}

// Use OnceLock in both test and non-test environments for efficiency and consistency.
// The OnceLock ensures thread-safe, one-time initialization.
fn get_api_config() -> Result<&'static ApiConfig, &'static (dyn Error + Send + Sync)> {
    API_CONFIG
        .get_or_init(create_new_api_config)
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
    // Get the static ApiConfig reference using the unified function
    let config = get_api_config()?;

    let url = format!("{}/whoami", config.base_url);
    logs.push(format!("Calling API: GET {}", url));

    let response = config.client.get(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        // Attempt to deserialize the successful response
        let response_text = response.text().await?; // Read body first for logging
        logs.push(format!("API Success Body: {}", response_text));
        let whoami_data: WhoamiResponse = serde_json::from_str(&response_text).map_err(|e| {
            format!(
                "Failed to deserialize success response: {} - Body: {}",
                e, response_text
            )
        })?;
        // logs.push(format!("API Response Data Parsed: {:?}", whoami_data)); // Maybe too verbose?
        Ok((whoami_data, logs))
    } else {
        // Verification (2025-04-21):
        // - OpenAPI spec lists 401/403 for /whoami but doesn't explicitly link ApiError schema.
        // - Luminork service code doesn't show explicit ApiError construction for these statuses.
        // - Current generic error handling (status + text body) is acceptable.
        // TODO: Consider attempting to parse the error body as `ApiError` in the future
        //       if the API guarantees that structure for 4xx/5xx errors.
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        logs.push(format!("API Error Body: {}", error_text));
        // Attempt to parse as ApiError for more structured logging, but fall back
        let error_message = match serde_json::from_str::<ApiError>(&error_text) {
            Ok(api_error) => format!(
                "API request failed with status {}: Code {:?}, Message: {}",
                status, api_error.code, api_error.message
            ),
            Err(_) => format!("API request failed with status {}: {}", status, error_text),
        };
        Err(error_message.into()) // Return the error message, logs are not returned on error path
    }
}

/// Fetches a list of change sets for a given workspace.
/// Intention: Calls the `GET /v1/w/{workspace_id}/change-sets` endpoint.
/// Design: Uses the initialized `reqwest::Client`, constructs the URL with the workspace ID,
///         sends a GET request, and deserializes the JSON response into `ListChangeSetV1Response`.
///         Includes logging similar to the `whoami` function.
/// Returns: A tuple containing the `ListChangeSetV1Response` on success and a `Vec<String>` of log messages.
pub async fn list_change_sets(
    workspace_id: &str,
) -> Result<(ListChangeSetV1Response, Vec<String>), Box<dyn Error + Send + Sync>> {
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!("{}/v1/w/{}/change-sets", config.base_url, workspace_id);
    logs.push(format!("Calling API: GET {}", url));

    let response = config.client.get(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        let list_response: ListChangeSetV1Response =
            serde_json::from_str(&response_text).map_err(|e| {
                format!(
                    "Failed to deserialize list change sets response: {} - Body: {}",
                    e, response_text
                )
            })?;
        Ok((list_response, logs))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        logs.push(format!("API Error Body: {}", error_text));
        let error_message = match serde_json::from_str::<ApiError>(&error_text) {
            Ok(api_error) => format!(
                "API request failed with status {}: Code {:?}, Message: {}",
                status, api_error.code, api_error.message
            ),
            Err(_) => format!("API request failed with status {}: {}", status, error_text),
        };
        Err(error_message.into())
    }
}

/// Creates a new change set in the specified workspace.
/// Intention: Calls the `POST /v1/w/{workspace_id}/change-sets` endpoint.
/// Design: Uses the initialized `reqwest::Client`, constructs the URL,
///         serializes the request body (`CreateChangeSetV1Request`), sends a POST request,
///         and deserializes the JSON response into `CreateChangeSetV1Response`.
///         Includes logging similar to other API functions.
/// Returns: A tuple containing the `CreateChangeSetV1Response` on success and a `Vec<String>` of log messages.
pub async fn create_change_set(
    workspace_id: &str,
    request_body: CreateChangeSetV1Request, // Use imported type directly
) -> Result<
    (CreateChangeSetV1Response, Vec<String>), // Use imported type directly
    Box<dyn Error + Send + Sync>,
> {
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!("{}/v1/w/{}/change-sets", config.base_url, workspace_id);
    logs.push(format!("Calling API: POST {}", url));
    logs.push(format!("Request Body: {:?}", request_body)); // Log the request body

    let response = config
        .client
        .post(&url)
        .json(&request_body) // Serialize the request body struct to JSON
        .send()
        .await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        let create_response: CreateChangeSetV1Response = serde_json::from_str(&response_text) // Use imported type directly
            .map_err(|e| {
                format!(
                    "Failed to deserialize create change set response: {} - Body: {}",
                    e, response_text
                )
            })?;
        Ok((create_response, logs))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        logs.push(format!("API Error Body: {}", error_text));
        let error_message = match serde_json::from_str::<ApiError>(&error_text) {
            Ok(api_error) => format!(
                "API request failed with status {}: Code {:?}, Message: {}",
                status, api_error.code, api_error.message
            ),
            Err(_) => format!("API request failed with status {}: {}", status, error_text),
        };
        Err(error_message.into())
    }
}

/// Fetches details for a specific change set.
/// Corresponds to `GET /v1/w/{workspace_id}/change-sets/{change_set_id}`.
///
/// # Arguments
/// * `workspace_id` - The ID of the workspace.
/// * `change_set_id` - The ID of the change set to fetch.
///
/// # Returns
/// A `Result` containing the `GetChangeSetV1Response` on success, or an error string.
/// Also returns a `Vec<String>` containing logs generated during the call.
pub async fn get_change_set(
    workspace_id: &str,
    change_set_id: &str,
) -> Result<(GetChangeSetV1Response, Vec<String>), Box<dyn Error + Send + Sync>> {
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!(
        "{}/v1/w/{}/change-sets/{}",
        config.base_url, workspace_id, change_set_id
    );
    logs.push(format!("Calling API: GET {}", url));

    let response = config.client.get(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        let get_response: GetChangeSetV1Response =
            serde_json::from_str(&response_text).map_err(|e| {
                format!(
                    "Failed to deserialize get change set response: {} - Body: {}",
                    e, response_text
                )
            })?;
        Ok((get_response, logs))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        logs.push(format!("API Error Body: {}", error_text));
        let error_message = match serde_json::from_str::<ApiError>(&error_text) {
            Ok(api_error) => format!(
                "API request failed with status {}: Code {:?}, Message: {}",
                status, api_error.code, api_error.message
            ),
            Err(_) => format!("API request failed with status {}: {}", status, error_text),
        };
        Err(error_message.into())
    }
}

/// Abandons a specific change set by its ID.
/// Corresponds to `DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}`.
/// Operation ID: `abandon_change_set` (Matches OpenAPI spec)
///
/// # Arguments
/// * `workspace_id` - The ID of the workspace containing the change set.
/// * `change_set_id` - The ID of the change set to delete.
///
/// # Returns
/// A `Result` containing the `DeleteChangeSetV1Response` (which includes a `success` boolean) on success, or an error string on failure.
/// Also returns a `Vec<String>` containing logs generated during the call.
///
/// # Intention
/// Provides the functionality to abandon a change set via the API.
///
/// # Design
/// - Constructs the specific URL for the change set deletion endpoint.
/// - Uses the shared `reqwest` client and configuration (via `get_api_config`).
/// - Sends an HTTP DELETE request.
/// - Handles success and error responses similarly to other API client functions.
/// - Deserializes the success response into `DeleteChangeSetV1Response` (which contains `{ "success": true }`).
/// - Logs relevant information about the request and response.
pub async fn abandon_change_set(
    // Renamed function
    workspace_id: &str,
    change_set_id: &str,
) -> Result<
    (DeleteChangeSetV1Response, Vec<String>), // Return type already matches plan
    Box<dyn Error + Send + Sync>,
> {
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!(
        "{}/v1/w/{}/change-sets/{}",
        config.base_url, workspace_id, change_set_id
    );
    logs.push(format!("Calling API: DELETE {}", url));

    let response = config.client.delete(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        // Deserialize the response which should contain `{"success": true}`
        let abandon_response: DeleteChangeSetV1Response = serde_json::from_str(&response_text)
            .map_err(|e| {
                format!(
                    "Failed to deserialize abandon change set response: {} - Body: {}", // Updated error message
                    e, response_text
                )
            })?;
        // TODO: Consider checking abandon_response.success here? Or let caller handle it.
        Ok((abandon_response, logs)) // Return the deserialized response
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        logs.push(format!("API Error Body: {}", error_text));
        let error_message = match serde_json::from_str::<ApiError>(&error_text) {
            Ok(api_error) => format!(
                "API request failed with status {}: Code {:?}, Message: {}",
                status, api_error.code, api_error.message
            ),
            Err(_) => format!("API request failed with status {}: {}", status, error_text),
        };
        Err(error_message.into())
    }
}

/// Fetches the merge status for a specific change set.
/// Corresponds to `GET /v1/w/{workspace_id}/change-sets/{change_set_id}/merge_status`.
/// Operation ID: `merge_status`
///
/// # Arguments
/// * `workspace_id` - The ID of the workspace containing the change set.
/// * `change_set_id` - The ID of the change set to get the status for.
///
/// # Returns
/// A `Result` containing the `MergeStatusV1Response` on success, or an error string on failure.
/// Also returns a `Vec<String>` containing logs generated during the call.
///
/// # Intention
/// Provides the functionality to retrieve the merge status (including actions) for a change set.
///
/// # Design
/// - Constructs the specific URL for the merge status endpoint.
/// - Uses the shared `reqwest` client and configuration (via `get_api_config`).
/// - Sends an HTTP GET request.
/// - Handles success and error responses similarly to other API client functions.
/// - Deserializes the success response into `MergeStatusV1Response`.
/// - Logs relevant information about the request and response.
pub async fn get_merge_status(
    workspace_id: &str,
    change_set_id: &str,
) -> Result<(MergeStatusV1Response, Vec<String>), Box<dyn Error + Send + Sync>> {
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!(
        "{}/v1/w/{}/change-sets/{}/merge_status", // Added /merge_status
        config.base_url, workspace_id, change_set_id
    );
    logs.push(format!("Calling API: GET {}", url));

    let response = config.client.get(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        let merge_status_response: MergeStatusV1Response = serde_json::from_str(&response_text)
            .map_err(|e| {
                format!(
                    "Failed to deserialize merge status response: {} - Body: {}",
                    e, response_text
                )
            })?;
        Ok((merge_status_response, logs))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        logs.push(format!("API Error Body: {}", error_text));
        let error_message = match serde_json::from_str::<ApiError>(&error_text) {
            Ok(api_error) => format!(
                "API request failed with status {}: Code {:?}, Message: {}",
                status, api_error.code, api_error.message
            ),
            Err(_) => format!("API request failed with status {}: {}", status, error_text),
        };
        Err(error_message.into())
    }
}

/// Force applies a specific change set.
/// Corresponds to `POST /v1/w/{workspace_id}/change-sets/{change_set_id}/force_apply`.
/// Operation ID: `force_apply` (Matches OpenAPI spec)
///
/// # Arguments
/// * `workspace_id` - The ID of the workspace containing the change set.
/// * `change_set_id` - The ID of the change set to force apply.
///
/// # Returns
/// A `Result` containing `()` on success (as the API returns no body), or an error string on failure.
/// Also returns a `Vec<String>` containing logs generated during the call.
///
/// # Intention
/// Provides the functionality to force apply a change set via the API.
///
/// # Design
/// - Constructs the specific URL for the force apply endpoint.
/// - Uses the shared `reqwest` client and configuration (via `get_api_config`).
/// - Sends an HTTP POST request (with no body).
/// - Handles success (200 OK, empty body according to OpenAPI spec) and error responses similarly to other API client functions.
/// - Logs relevant information about the request and response.
pub async fn force_apply(
    // Renamed function
    workspace_id: &str,
    change_set_id: &str,
) -> Result<((), Vec<String>), Box<dyn Error + Send + Sync>> {
    // Return type is correct (unit tuple)
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!(
        "{}/v1/w/{}/change-sets/{}/force_apply", // Added /force_apply
        config.base_url, workspace_id, change_set_id
    );
    logs.push(format!("Calling API: POST {}", url));

    // Send POST request with no body
    let response = config.client.post(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        // Success response has no body according to OpenAPI spec
        let response_text = response.text().await?; // Read body anyway for logging
        logs.push(format!(
            "API Success Body (expected empty): {}",
            response_text
        ));
        Ok(((), logs)) // Return unit tuple for success
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        logs.push(format!("API Error Body: {}", error_text));
        let error_message = match serde_json::from_str::<ApiError>(&error_text) {
            Ok(api_error) => format!(
                "API request failed with status {}: Code {:?}, Message: {}",
                status, api_error.code, api_error.message
            ),
            Err(_) => format!("API request failed with status {}: {}", status, error_text),
        };
        Err(error_message.into())
    }
}

// TODO: Add functions for other API endpoints as needed.
// Examples:
// pub async fn get_component(workspace_id: &str, change_set_id: &str, component_id: &str) -> Result<(GetComponentV1Response, Vec<String>), Box<dyn Error + Send + Sync>> { ... }

// Note: The error type `Box<dyn Error + Send + Sync>` is used for flexibility,
// allowing different error types (IO, HTTP, API errors) to be returned.
// Consider defining a custom error enum for more specific error handling later.

//=============================================================================
// Component API Client Functions (Added based on openapi.json)
//=============================================================================

/// Creates a new component within a specific change set.
/// Corresponds to `POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components`.
/// Operation ID: `create_component`
pub async fn create_component(
    workspace_id: &str,
    change_set_id: &str,
    request_body: CreateComponentV1Request,
) -> Result<(CreateComponentV1Response, Vec<String>), Box<dyn Error + Send + Sync>> {
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!(
        "{}/v1/w/{}/change-sets/{}/components",
        config.base_url, workspace_id, change_set_id
    );
    logs.push(format!("Calling API: POST {}", url));
    logs.push(format!("Request Body: {:?}", request_body));

    let response = config.client.post(&url).json(&request_body).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        let create_response: CreateComponentV1Response = serde_json::from_str(&response_text)
            .map_err(|e| {
                format!(
                    "Failed to deserialize create component response: {} - Body: {}",
                    e, response_text
                )
            })?;
        Ok((create_response, logs))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        logs.push(format!("API Error Body: {}", error_text));
        let error_message = match serde_json::from_str::<ApiError>(&error_text) {
            Ok(api_error) => format!(
                "API request failed with status {}: Code {:?}, Message: {}",
                status, api_error.code, api_error.message
            ),
            Err(_) => format!("API request failed with status {}: {}", status, error_text),
        };
        Err(error_message.into())
    }
}

/// Fetches details for a specific component within a change set.
/// Corresponds to `GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`.
/// Operation ID: `get_component`
pub async fn get_component(
    workspace_id: &str,
    change_set_id: &str,
    component_id: &str,
) -> Result<(GetComponentV1Response, Vec<String>), Box<dyn Error + Send + Sync>> {
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!(
        "{}/v1/w/{}/change-sets/{}/components/{}",
        config.base_url, workspace_id, change_set_id, component_id
    );
    logs.push(format!("Calling API: GET {}", url));

    let response = config.client.get(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        let get_response: GetComponentV1Response =
            serde_json::from_str(&response_text).map_err(|e| {
                format!(
                    "Failed to deserialize get component response: {} - Body: {}",
                    e, response_text
                )
            })?;
        Ok((get_response, logs))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        logs.push(format!("API Error Body: {}", error_text));
        let error_message = match serde_json::from_str::<ApiError>(&error_text) {
            Ok(api_error) => format!(
                "API request failed with status {}: Code {:?}, Message: {}",
                status, api_error.code, api_error.message
            ),
            Err(_) => format!("API request failed with status {}: {}", status, error_text),
        };
        Err(error_message.into())
    }
}

/// Updates a specific component within a change set.
/// Corresponds to `PUT /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`.
/// Operation ID: `update_component`
pub async fn update_component(
    workspace_id: &str,
    change_set_id: &str,
    component_id: &str,
    request_body: UpdateComponentV1Request,
) -> Result<(UpdateComponentV1Response, Vec<String>), Box<dyn Error + Send + Sync>> {
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!(
        "{}/v1/w/{}/change-sets/{}/components/{}",
        config.base_url, workspace_id, change_set_id, component_id
    );
    logs.push(format!("Calling API: PUT {}", url));
    logs.push(format!("Request Body: {:?}", request_body));

    let response = config.client.put(&url).json(&request_body).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        // Response body is empty `{}`, so deserialize into the empty struct
        let update_response: UpdateComponentV1Response = serde_json::from_str(&response_text)
            .map_err(|e| {
                format!(
                    "Failed to deserialize update component response: {} - Body: {}",
                    e, response_text
                )
            })?;
        Ok((update_response, logs))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        logs.push(format!("API Error Body: {}", error_text));
        let error_message = match serde_json::from_str::<ApiError>(&error_text) {
            Ok(api_error) => format!(
                "API request failed with status {}: Code {:?}, Message: {}",
                status, api_error.code, api_error.message
            ),
            Err(_) => format!("API request failed with status {}: {}", status, error_text),
        };
        Err(error_message.into())
    }
}

/// Deletes a specific component within a change set.
/// Corresponds to `DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`.
/// Operation ID: `delete_component`
pub async fn delete_component(
    workspace_id: &str,
    change_set_id: &str,
    component_id: &str,
) -> Result<(DeleteComponentV1Response, Vec<String>), Box<dyn Error + Send + Sync>> {
    let mut logs = Vec::new();
    // Get the static ApiConfig reference
    let config = get_api_config()?;

    let url = format!(
        "{}/v1/w/{}/change-sets/{}/components/{}",
        config.base_url, workspace_id, change_set_id, component_id
    );
    logs.push(format!("Calling API: DELETE {}", url));

    let response = config.client.delete(&url).send().await?;

    let status = response.status();
    logs.push(format!("API Response Status: {}", status));

    if status.is_success() {
        let response_text = response.text().await?;
        logs.push(format!("API Success Body: {}", response_text));
        let delete_response: DeleteComponentV1Response = serde_json::from_str(&response_text)
            .map_err(|e| {
                format!(
                    "Failed to deserialize delete component response: {} - Body: {}",
                    e, response_text
                )
            })?;
        Ok((delete_response, logs))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        logs.push(format!("API Error Body: {}", error_text));
        let error_message = match serde_json::from_str::<ApiError>(&error_text) {
            Ok(api_error) => format!(
                "API request failed with status {}: Code {:?}, Message: {}",
                status, api_error.code, api_error.message
            ),
            Err(_) => format!("API request failed with status {}: {}", status, error_text),
        };
        Err(error_message.into())
    }
}
