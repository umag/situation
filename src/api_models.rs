// src/api_models.rs

// Intention:
// Defines Rust data structures that correspond to the JSON schemas defined in the
// openapi.json specification for the Luminork API. These structs are used for
// deserializing API responses.

// Design Choices:
// - Uses `serde::Deserialize` for easy conversion from JSON.
// - Field names match the JSON properties defined in the OpenAPI schema.
// - Uses `Option` for fields that are not explicitly marked as required or might be nullable
//   (like `code` in `ApiError`).
// - Added basic documentation for each struct and its fields.
// - Verification (2025-04-21): Initial check suggested token was string, but runtime error shows it's an object.
//   Updated WhoamiResponse and re-added TokenDetails struct to match actual API behavior.

use serde::Deserialize;

/// Represents the nested token details within the WhoamiResponse.
/// This structure reflects the actual runtime response from the API.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // Assuming camelCase based on parent, adjust if needed
pub struct TokenDetails {
    /// Issued at timestamp.
    pub iat: i64, // Assuming timestamp fits in i64
    /// Subject (often user ID).
    pub sub: String,
    /// User primary key. Matches `user_pk` in the actual response object.
    #[serde(rename = "user_pk")] // Override rename_all for this field
    pub user_pk: String,
    /// Workspace primary key. Matches `workspace_pk` in the actual response object.
    #[serde(rename = "workspace_pk")] // Override rename_all for this field
    pub workspace_pk: String,
}

/// Represents the response from the `/whoami` endpoint.
/// Contains information about the authenticated user and their workspace.
/// Verification (2025-04-21): Updated based on runtime error. The `token` field is an object, not a string.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WhoamiResponse {
    /// The unique identifier for the user. Matches OpenAPI `userId`.
    pub user_id: String,
    /// The email address of the user. Matches OpenAPI `userEmail`.
    pub user_email: String,
    /// The identifier for the user's current workspace. Matches OpenAPI `workspaceId`.
    pub workspace_id: String,
    /// Detailed information extracted from the authentication token. Matches actual API response.
    pub token: TokenDetails, // Reverted: Changed back from String to TokenDetails based on runtime error.
}

/// Represents a standard error response from the v1 API.
/// Verification (2025-04-21): Confirmed structure matches OpenAPI spec `ApiError`.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    /// An optional error code.
    pub code: Option<i32>,
    /// A human-readable error message.
    pub message: String,
    /// The HTTP status code associated with the error.
    pub status_code: u16, // Using u16 for HTTP status codes
}

/// Represents a summary of a change set, typically used in lists.
/// Based on the example in openapi.json for ListChangeSetV1Response.
/// Fields assumed based on the example: {"id":"...", "name":"...", "status":"..."}
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetSummary {
    /// The unique identifier for the change set.
    pub id: String,
    /// The user-provided name for the change set.
    pub name: String,
    /// The current status of the change set (e.g., "Draft", "Applied").
    pub status: String,
    // Note: Add other fields if the actual API response includes more than the example.
}

/// Represents the response from the `GET /v1/w/{workspace_id}/change-sets` endpoint.
/// Contains a list of change set summaries.
/// Based on the schema `ListChangeSetV1Response` in openapi.json.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListChangeSetV1Response {
    /// A list containing summaries of the available change sets.
    pub change_sets: Vec<ChangeSetSummary>,
}

/// Represents the request body for the `POST /v1/w/{workspace_id}/change-sets` endpoint.
/// Based on the schema `CreateChangeSetV1Request` in openapi.json.
#[derive(Debug, serde::Serialize, Clone)] // Use Serialize for request bodies
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetV1Request {
    /// The desired name for the new change set.
    pub change_set_name: String,
}

/// Represents the response from the `POST /v1/w/{workspace_id}/change-sets` endpoint.
/// Based on the schema `CreateChangeSetV1Response` in openapi.json.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetV1Response {
    /// Contains details of the created change set. The exact structure is not fully
    /// specified in the OpenAPI schema (just `{"type": "object"}`), so we use `serde_json::Value`
    /// to capture whatever object is returned.
    #[serde(rename = "changeSet")] // Match the exact name from OpenAPI
    pub change_set: serde_json::Value,
}

/// Represents the response from the `GET /v1/w/{workspace_id}/change-sets/{change_set_id}` endpoint.
/// Based on the schema `GetChangeSetV1Response` in openapi.json.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetChangeSetV1Response {
    /// Contains details of the specific change set. The exact structure is not fully
    /// specified in the OpenAPI schema (just `{"type": "object"}`), so we use `serde_json::Value`
    /// to capture whatever object is returned.
    #[serde(rename = "changeSet")] // Match the exact name from OpenAPI
    pub change_set: serde_json::Value,
}

// TODO: Add more structs here as needed based on openapi.json schemas
// for other endpoints like Components, etc.
// Examples:
// pub struct GetComponentV1Response { ... }
