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
use serde_json; // Added import for serde_json::Value

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

/// Represents the detailed structure of a change set.
/// Based on ChangeSetSummary and common fields expected in detailed views.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSet {
    /// The unique identifier for the change set.
    pub id: String,
    /// The user-provided name for the change set.
    pub name: String,
    /// The current status of the change set (e.g., "Draft", "Applied").
    pub status: String,
    // TODO: Add more fields here if the API provides them in detailed responses
    // (e.g., description, created_at, updated_at).
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
    /// Contains details of the created change set.
    #[serde(rename = "changeSet")] // Match the exact name from OpenAPI
    pub change_set: ChangeSet, // Use the defined ChangeSet struct
}

/// Represents the response from the `GET /v1/w/{workspace_id}/change-sets/{change_set_id}` endpoint.
/// Based on the schema `GetChangeSetV1Response` in openapi.json.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetChangeSetV1Response {
    /// Contains details of the specific change set.
    #[serde(rename = "changeSet")] // Match the exact name from OpenAPI
    pub change_set: ChangeSet, // Use the defined ChangeSet struct
}

/// Response for DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}
/// Intention: Represents the successful deletion confirmation from the API.
/// Design: Simple struct with a boolean flag as defined in the OpenAPI schema `DeleteChangeSetV1Response`.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeleteChangeSetV1Response {
    /// Indicates whether the deletion was successful.
    pub success: bool,
}

/// Represents component details within a merge status action.
/// Based on `MergeStatusV1ResponseActionComponent` in openapi.json.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MergeStatusV1ResponseActionComponent {
    /// The unique identifier for the component.
    pub id: String,
    /// The name of the component.
    pub name: String,
}

/// Represents a single action within the merge status response.
/// Based on `MergeStatusV1ResponseAction` in openapi.json.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MergeStatusV1ResponseAction {
    /// The unique identifier for the action.
    pub id: String,
    /// The current state of the action (e.g., "Added", "Modified", "Deleted").
    pub state: String,
    /// The kind of action (e.g., "Create", "Update", "Delete").
    pub kind: String,
    /// The name associated with the action.
    pub name: String,
    /// Optional component details related to the action.
    pub component: Option<MergeStatusV1ResponseActionComponent>,
}

/// Represents the response from the `GET /v1/w/{workspace_id}/change-sets/{change_set_id}/merge_status` endpoint.
/// Based on the schema `MergeStatusV1Response` in openapi.json.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MergeStatusV1Response {
    /// Contains details of the change set itself.
    #[serde(rename = "changeSet")] // Match the exact name from OpenAPI
    pub change_set: ChangeSet, // Use the defined ChangeSet struct
    /// A list of actions associated with the change set's merge status.
    pub actions: Vec<MergeStatusV1ResponseAction>,
}

//=============================================================================
// Component API Models (Added based on openapi.json)
//=============================================================================

// --- Shared Component Sub-Structs ---

/// Represents a reference to a component, typically by its ID.
/// Based on `ComponentReference` in openapi.json (prioritizing `componentId`).
#[derive(serde::Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComponentReference {
    pub component_id: String,
}

/// Represents a connection point on a component (component + socket).
/// Based on `ConnectionPoint` in openapi.json.
#[derive(serde::Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionPoint {
    pub component_id: String, // Assuming component_id is used based on ComponentReference
    pub socket_name: String,
}

/// Represents a connection between two component sockets.
/// Based on `Connection` in openapi.json (using untagged enum for the two directions).
#[derive(serde::Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Connection {
    /// Connection from another component's output to this component's input.
    OutputToInput {
        from: ConnectionPoint, // Source component and output socket
        to: String,            // Target input socket name (on this component)
    },
    /// Connection from this component's output to another component's input.
    InputFromOutput {
        from: String,        // Source output socket name (on this component)
        to: ConnectionPoint, // Target component and input socket
    },
}

// --- Create Component ---

/// Request body for `POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components`.
/// Based on `CreateComponentV1Request` in openapi.json.
#[derive(serde::Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentV1Request {
    /// The domain properties for the component (arbitrary JSON object).
    pub domain: serde_json::Value,
    /// The name for the new component.
    pub name: String,
    /// The schema name for the component (e.g., "AWS::EC2::Instance").
    pub schema_name: String,
    /// List of connections for the component.
    pub connections: Vec<Connection>,
    /// Optional view name associated with the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_name: Option<String>,
}

/// Response for `POST /v1/w/{workspace_id}/change-sets/{change_set_id}/components`.
/// Based on `CreateComponentV1Response` in openapi.json.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentV1Response {
    /// The ID of the newly created component.
    pub component_id: String,
}

// --- Get Component ---

/// Represents geometry, view, and name information, likely for UI layout.
/// Based on `GeometryAndViewAndName` in openapi.json (schema is vague, assuming 'name').
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeometryAndViewAndName {
    pub name: String,
    // Note: Add other fields like x, y, width, height if known/needed from actual responses.
}

/// Represents a management function available for a component.
/// Based on `GetComponentV1ResponseManagementFunction` in openapi.json.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentV1ResponseManagementFunction {
    pub management_prototype_id: String,
    pub name: String,
}

/// Response for `GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`.
/// Based on `GetComponentV1Response` in openapi.json.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentV1Response {
    /// The component's data (arbitrary JSON object).
    pub component: serde_json::Value,
    /// The component's domain properties (arbitrary JSON object).
    pub domain: serde_json::Value,
    /// List of available management functions for the component.
    pub management_functions: Vec<GetComponentV1ResponseManagementFunction>,
    /// List of view-related data for the component.
    pub view_data: Vec<GeometryAndViewAndName>,
}

// --- Update Component ---

/// Request body for `PUT /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`.
/// Based on `UpdateComponentV1Request` in openapi.json.
#[derive(serde::Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponentV1Request {
    /// The updated domain properties for the component (arbitrary JSON object).
    pub domain: serde_json::Value,
    /// Optional updated name for the component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Response for `PUT /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`.
/// Based on `UpdateComponentV1Response` in openapi.json (empty object {}).
#[derive(Deserialize, Debug, Clone)]
pub struct UpdateComponentV1Response {
    // Empty struct represents the empty JSON object response `{}`.
}

// --- Delete Component ---

/// Response for `DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`.
/// Based on `DeleteComponentV1Response` in openapi.json.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeleteComponentV1Response {
    /// The status after deletion (e.g., "MarkedForDeletion").
    pub status: String,
}

// --- List Components ---

/// Represents the direction of a socket (input or output).
/// Based on `SocketDirection` enum in openapi.json.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SocketDirection {
    Input,
    Output,
}

/// Represents a socket on a component.
/// Based on `SocketViewV1` in openapi.json.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SocketViewV1 {
    pub id: String,
    pub name: String,
    pub direction: SocketDirection,
    pub arity: String,            // e.g., "one", "many"
    pub value: serde_json::Value, // Arbitrary JSON value
}

/// Represents a view associated with a component.
/// Based on `ViewV1` in openapi.json.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ViewV1 {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

/// Represents a property view for a component (domain or resource).
/// Based on `ComponentPropViewV1` in openapi.json.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComponentPropViewV1 {
    pub id: String,
    pub prop_id: String,
    pub value: serde_json::Value, // Arbitrary JSON value
    pub path: String,
}

// --- Connection View Sub-Structs ---

/// Represents an incoming connection view.
/// Based on `IncomingConnectionViewV1` in openapi.json.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IncomingConnectionViewV1 {
    pub from_component_id: String,
    pub from_component_name: String,
    pub from: String, // Socket name on the source component
    pub to: String,   // Socket name on the destination component (this one)
}

/// Represents an outgoing connection view.
/// Based on `OutgoingConnectionViewV1` in openapi.json.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OutgoingConnectionViewV1 {
    pub to_component_id: String,
    pub to_component_name: String,
    pub from: String, // Socket name on the source component (this one)
                      // Note: 'to' socket name is missing in OpenAPI spec, might be implied or an omission.
}

/// Represents a managing connection view.
/// Based on `ManagingConnectionViewV1` in openapi.json.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManagingConnectionViewV1 {
    pub component_id: String,
    pub component_name: String,
}

/// Represents a managed-by connection view.
/// Based on `ManagedByConnectionViewV1` in openapi.json.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManagedByConnectionViewV1 {
    pub component_id: String,
    pub component_name: String,
}

/// Represents different types of connection views.
/// Based on `ConnectionViewV1` (oneOf) in openapi.json.
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)] // Using untagged because the structure differs based on the single key
pub enum ConnectionViewV1 {
    Incoming {
        incoming: IncomingConnectionViewV1,
    },
    Outgoing {
        outgoing: OutgoingConnectionViewV1,
    },
    Managing {
        managing: ManagingConnectionViewV1,
    },
    ManagedBy {
        managed_by: ManagedByConnectionViewV1,
    },
}

/// Represents a detailed view of a component.
/// Based on `ComponentViewV1` in openapi.json.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComponentViewV1 {
    pub id: String,
    pub schema_id: String,
    pub schema_variant_id: String,
    pub sockets: Vec<SocketViewV1>,
    pub domain_props: Vec<ComponentPropViewV1>,
    pub resource_props: Vec<ComponentPropViewV1>,
    pub name: String,
    pub resource_id: String,
    pub to_delete: bool,
    pub can_be_upgraded: bool,
    pub connections: Vec<ConnectionViewV1>,
    pub views: Vec<ViewV1>,
}

/// Response for `GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components`.
/// Based on `ListComponentsV1Response` in openapi.json, but corrected to match actual API response.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsV1Response {
    /// A list of component IDs in the change set.
    pub components: Vec<String>,
}

// --- List Schemas ---

/// Represents a summary of a schema as returned by the list_schemas endpoint.
/// Based on the example in `ListSchemaV1Response` in openapi.json.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchemaSummary {
    /// The unique identifier for the schema.
    pub schema_id: String,
    /// The name of the schema (e.g., "AWS::EC2::Instance").
    pub schema_name: String,
    /// The category the schema belongs to.
    pub category: String,
    /// Indicates if the schema is installed.
    /// Design Choice: Changed from String to bool based on decoding error. Assumes API returns true/false.
    pub installed: bool,
}

/// Response for `GET /v1/w/{workspace_id}/change-sets/{change_set_id}/schema`.
/// Based on `ListSchemaV1Response` in openapi.json.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListSchemaV1Response {
    /// A list containing summaries of the available schemas.
    pub schemas: Vec<SchemaSummary>,
}

// TODO: Add more structs here as needed based on openapi.json schemas
// for other endpoints like Management Prototypes, etc.
