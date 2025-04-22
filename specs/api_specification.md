# API Call Specification (SDF Server)

This document outlines the API calls implemented in the `sdf-server` service and
its associated `sdf-v1-routes-*` crates.

## Common Details

- **Base Path:** Most routes are nested under `/api`.
- **Authentication:** Typically requires a `Bearer <JWT_TOKEN>` in the
  `Authorization` header, often validated via middleware
  (`WorkspaceAuthorization`, `AdminUserContext`, etc.). Specific permissions
  might be required for certain endpoints.
- **Error Handling:** Errors generally return a JSON object conforming to
  `sdf_core::api_error::ApiError` with a relevant HTTP status code. Specific
  error types (e.g., `ChangeSetError`, `ComponentError`, `AdminAPIError`) define
  status codes for particular error conditions.

## Root API Endpoints (`/api`)

### 1. System Status

- **Method:** `GET`
- **Path:** `/api/`
- **Handler:** `sdf-server/src/routes.rs::system_status_route`
- **Success Response (200):** JSON `{"ok": true}`
- **Description:** Basic health check endpoint.

## Whoami Endpoint (`/api/whoami`)

### 1. Get User Information

- **Method:** `GET`
- **Path:** `/api/whoami/`
- **Handler:** `sdf-server/src/service/whoami.rs::whoami`
- **Success Response (200):** JSON object containing `userId`, `userEmail`,
  `workspaceId`, and `token`.
- **Description:** Fetches information about the currently authenticated user
  based on the provided JWT.

## V1 Endpoints (`/api/{resource}`)

These routes are defined in separate `sdf-v1-routes-*` crates.

### Change Set Routes (`/api/change_set`)

_(Defined in `sdf-v1-routes-change-sets`)_

- **`POST /add_action`**: Adds an action to the change set.
  - Handler: `add_action::add_action`
  - Request: `AddActionRequest` (`prototype_id`, `component_id`, `visibility`)
  - Response: `ForceChangeSetResponse<()>`
- **`POST /create_change_set`**: Creates a new change set.
  - Handler: `create_change_set::create_change_set`
  - Request: `CreateChangeSetRequest` (`change_set_name`)
  - Response: `CreateChangeSetResponse` (`change_set`)
- **`POST /abandon_change_set`**: Abandons a change set.
  - Handler: `abandon_change_set::abandon_change_set`
  - Request: `AbandonChangeSetRequest` (`change_set_id`)
  - Response: `()` (Success)
- **`POST /begin_approval_process`**: Starts the approval process for a change
  set.
  - Handler: `begin_approval_process::begin_approval_process`
  - Request: `BeginMergeFlow` (`visibility`)
  - Response: `()` (Success)
- **`POST /cancel_approval_process`**: Cancels the approval process for a change
  set.
  - Handler: `begin_approval_process::cancel_approval_process`
  - Request: `CancelMergeFlow` (`visibility`)
  - Response: `()` (Success)

### Component Routes (`/api/component`)

_(Defined in `sdf-v1-routes-component`)_

- **`GET /get_actions`**: Lists available actions for a component.
  - Handler: `get_actions::get_actions`
  - Request Query: `GetActionsRequest` (`component_id`, `visibility`)
  - Response: `GetActionsResponse` (`actions`: Vec<`ActionPrototypeView`>)
- **`GET /get_property_editor_schema`**: Gets the property editor schema for a
  component.
  - Handler: `get_property_editor_schema::get_property_editor_schema`
  - Request Query: `GetPropertyEditorSchemaRequest` (`component_id`,
    `visibility`)
  - Response: `PropertyEditorSchema`
- **`GET /get_property_editor_values`**: Gets the property editor values for a
  component.
  - Handler: `get_property_editor_values::get_property_editor_values`
  - Request Query: `GetPropertyEditorValuesRequest` (`component_id`,
    `visibility`)
  - Response: JSON Value (`PropertyEditorValues`)
- **`GET /list_qualifications`**: Lists qualifications for a component.
  - Handler: `list_qualifications::list_qualifications`
  - Request Query: `ListQualificationsRequest` (`component_id`, `visibility`)
  - Response: `QualificationResponse` (Vec<`QualificationView`>)
- **`GET /get_code`**: Gets generated code views for a component.
  - Handler: `get_code::get_code`
  - Request Query: `GetCodeRequest` (`component_id`, `visibility`)
  - Response: `GetCodeResponse` (`code_views`, `has_code`)
- **`GET /get_diff`**: Gets the diff for a component compared to HEAD.
  - Handler: `get_diff::get_diff`
  - Request Query: `GetDiffRequest` (`component_id`, `visibility`)
  - Response: `GetDiffResponse` (`component_diff`)
- **`GET /get_resource`**: Gets the resource view for a component.
  - Handler: `get_resource::get_resource`
  - Request Query: `GetResourceRequest` (`component_id`, `visibility`)
  - Response: `GetResourceResponse` (`resource`)
- **`POST /update_property_editor_value`**: Updates a specific property value.
  - Handler: `update_property_editor_value::update_property_editor_value`
  - Request: `UpdatePropertyEditorValueRequest` (`attribute_value_id`,
    `parent_attribute_value_id`, `prop_id`, `component_id`, `value`, `key`,
    `is_for_secret`, `visibility`)
  - Response: `ForceChangeSetResponse<()>`
- **`POST /insert_property_editor_value`**: Inserts a value into a map or array
  property.
  - Handler: `insert_property_editor_value::insert_property_editor_value`
  - Request: `InsertPropertyEditorValueRequest` (`parent_attribute_value_id`,
    `prop_id`, `component_id`, `value`, `key`, `visibility`)
  - Response: `ForceChangeSetResponse<()>`
- **`POST /delete_property_editor_value`**: Deletes a property value (or
  map/array entry).
  - Handler: `delete_property_editor_value::delete_property_editor_value`
  - Request: `DeletePropertyEditorValueRequest` (`attribute_value_id`,
    `prop_id`, `component_id`, `key`, `visibility`)
  - Response: `ForceChangeSetResponse<()>`
- **`POST /restore_default_function`**: Restores the default function for an
  attribute value.
  - Handler: `restore_default_function::restore_default_function`
  - Request: `RestoreDefaultFunctionRequest` (`attribute_value_id`,
    `visibility`)
  - Response: `ForceChangeSetResponse<()>`
- **`POST /set_type`**: Sets the type (Component/ConfigurationFrame) of a
  component.
  - Handler: `set_type::set_type`
  - Request: `SetTypeRequest` (`component_id`, `component_type`, `visibility`)
  - Response: `ForceChangeSetResponse<()>`
- **`POST /set_name`**: Sets the name of a component.
  - Handler: `set_name::set_name`
  - Request: `SetNameRequest` (`component_id`, `name`, `visibility`)
  - Response: `ForceChangeSetResponse<()>`
- **`POST /set_resource_id`**: Sets the resource ID for a component.
  - Handler: `set_resource_id::set_resource_id`
  - Request: `SetResourceIdRequest` (`component_id`, `resource_id`,
    `visibility`)
  - Response: `ForceChangeSetResponse<()>`
- **`POST /refresh`**: Enqueues a refresh action for a component.
  - Handler: `refresh::refresh`
  - Request: `RefreshRequest` (`component_id`, `visibility`)
  - Response: `RefreshResponse` (`success`)
- **`GET /debug`**: Gets debug information for a component.
  - Handler: `debug::debug_component`
  - Request Query: `DebugComponentRequest` (`component_id`, `visibility`)
  - Response: `ComponentDebugView`
- **`POST /autoconnect`**: Attempts to automatically connect component sockets.
  - Handler: `autoconnect::autoconnect`
  - Request: `AutoconnectComponentRequest` (`component_id`, `visibility`)
  - Response: `ForceChangeSetResponse<AutoconnectComponentResponse>` (`created`,
    `potential_incoming`)
- **`POST /override_with_connection`**: Resets an attribute value and creates a
  connection to it.
  - Handler: `override_with_connection::override_with_connection`
  - Request: `OverrideWithConnectionRequest` (`from_component_id`,
    `from_socket_id`, `to_component_id`, `to_socket_id`,
    `attribute_value_id_to_override`, `visibility`)
  - Response: `ForceChangeSetResponse<()>`
- **`GET /json`**: Gets the JSON representation of a component's properties.
  - Handler: `json::json`
  - Request Query: `JsonRequest` (`component_id`, `visibility`)
  - Response: `JsonResponse` (`json`)
- **`POST /upgrade_component`**: Upgrades a component to the latest unlocked
  schema variant.
  - Handler: `upgrade::upgrade`
  - Request: `UpgradeComponentRequest` (`component_id`, `visibility`)
  - Response: `ForceChangeSetResponse<()>`
- **`GET /conflicts`**: Gets conflicts for a component (currently returns
  empty).
  - Handler: `conflicts_for_component::conflicts_for_component`
  - Request Query: `ConflictsForComponentRequest` (`component_id`, `visibility`)
  - Response: `ConflictsForComponentResponse` (HashMap<AttributeValueId,
    ConflictWithHead>)
- **`POST /manage`**: Creates a "manages" edge between two components.
  - Handler: `manage::manage`
  - Request: `ManageComponentRequest` (`manager_component_id`,
    `managed_component_id`, `visibility`)
  - Response: `ForceChangeSetResponse<()>`
- **`POST /unmanage`**: Removes a "manages" edge between two components.
  - Handler: `unmanage::unmanage`
  - Request: `UnmanageComponentRequest` (`manager_component_id`,
    `managed_component_id`, `visibility`)
  - Response: `ForceChangeSetResponse<()>`

### Action Routes (`/api/action`)

_(Defined in `sdf-v1-routes-actions`)_

- **`GET /list`**: Lists actions in the current change set.
  - Handler: `list_actions::list_actions`
  - Request Query: `LoadQueuedRequest` (`visibility`)
  - Response: `LoadQueuedResponse` (Vec<`ActionView`>)
- **`POST /put_on_hold`**: Sets the state of specified actions to "OnHold".
  - Handler: `put_on_hold::put_on_hold`
  - Request: `PutOnHoldRequest` (`ids`, `visibility`)
  - Response: `()` (Success)
- **`POST /cancel`**: Cancels (removes) specified actions.
  - Handler: `cancel::cancel`
  - Request: `PutOnHoldRequest` (`ids`, `visibility`) (Reuses request type)
  - Response: `()` (Success)
- **`POST /retry`**: Sets the state of specified actions back to "Queued".
  - Handler: `retry::retry`
  - Request: `RetryRequest` (`ids`, `visibility`)
  - Response: `()` (Success)
- **`GET /history`**: Gets the history of action runs.
  - Handler: `history::history`
  - Request Query: `ActionHistoryRequest` (`visibility`)
  - Response: `ActionHistoryResponse` (Vec<`ActionHistoryView`>)

### Node Debug Routes (`/api/node_debug`)

_(Defined in `sdf-v1-routes-node-debug`)_

- **`GET /`**: Gets debug information for a specific node in the workspace
  snapshot graph.
  - Handler: `node_debug`
  - Request Query: `NodeDebugRequest` (`id`, `visibility`)
  - Response: `NodeDebugResponse` (`node`, `incoming_edges`, `outgoing_edges`)

### Attribute Routes (`/api/attribute`)

_(Defined in `sdf-v1-routes-attribute`)_

- **`GET /get_prototype_arguments`**: Gets the prepared arguments for an
  attribute prototype function execution.
  - Handler: `get_prototype_arguments::get_prototype_arguments`
  - Request Query: `GetPrototypeArgumentsRequest` (`prop_id`,
    `output_socket_id`, `visibility`)
  - Response: `GetPrototypeArgumentsResponse` (`prepared_arguments`)

### Diagram Routes (`/api/diagram`)

_(Defined in `sdf-v1-routes-diagram`)_

- **`POST /add_components_to_view`**: Adds existing components to a specific
  view with geometry.
  - Handler: `add_components_to_view::add_components_to_view`
  - Request: `Request` (`source_view_id`, `destination_view_id`,
    `geometries_by_component_id`, `remove_from_original_view`, `visibility`)
  - Response: `ForceChangeSetResponse<()>`
- **`POST /delete_connection`**: Deletes a connection between two components.
  - Handler: `delete_connection::delete_connection`
  - Request: `DeleteConnectionRequest` (`from_socket_id`, `from_component_id`,
    `to_component_id`, `to_socket_id`, `visibility`)
  - Response: `ForceChangeSetResponse<()>`
- **`POST /delete_components`**: Deletes a set of components.
  - Handler: `delete_component::delete_components`
  - Request: `DeleteComponentsRequest` (`component_ids`, `force_erase`,
    `visibility`)
  - Response: `ForceChangeSetResponse<HashMap<ComponentId, bool>>` (Map of
    component ID to whether it was marked for deletion)
- **`POST /remove_delete_intent`**: Restores components marked for deletion or
  restores from base change set.
  - Handler: `remove_delete_intent::remove_delete_intent`
  - Request: `RemoveDeleteIntentRequest` (`components`, `visibility`)
  - Response: `ForceChangeSetResponse<()>`
- **`POST /create_connection`**: Creates a connection between two components.
  - Handler: `create_connection::create_connection`
  - Request: `CreateConnectionRequest` (`from_component_id`, `from_socket_id`,
    `to_component_id`, `to_socket_id`, `visibility`)
  - Response: `ForceChangeSetResponse<()>`
- **`POST /create_component`**: Creates a new component.
  - Handler: `create_component::create_component`
  - Request: `CreateComponentRequest` (`schema_type`, `schema_variant_id`,
    `schema_id`, `parent_id`, `x`, `y`, `height`, `width`, `visibility`)
  - Response: `ForceChangeSetResponse<CreateComponentResponse>` (`component_id`,
    `installed_variant`)
- **`POST /set_component_position`**: Sets the position and potentially parent
  of multiple components.
  - Handler: `set_component_position::set_component_position`
  - Request: `SetComponentPositionRequest` (`visibility`,
    `data_by_component_id`, `client_ulid`, `request_ulid`)
  - Response: `ForceChangeSetResponse<SetComponentPositionResponse>`
    (`request_ulid`)
- **`GET /get_diagram`**: Gets the diagram data for the default view.
  (Deprecated?)
  - Handler: `get_diagram::get_diagram`
  - Request Query: `GetDiagramRequest` (`visibility`)
  - Response: `Diagram`
- **`GET /get_all_components_and_edges`**: Gets all components and edges for all
  views.
  - Handler: `get_all_components_and_edges::get_all_components_and_edges`
  - Request Query: `Request` (`visibility`)
  - Response: `Diagram`
- **`GET /list_schemas`**: Lists available schemas (installed and cached).
  - Handler: `list_schemas::list_schemas`
  - Request Query: `ListSchemaVariantsRequest` (`visibility`)
  - Response: `ListSchemasResponse` (Vec<`SchemaView`>)
- **`GET /dvu_roots`**: Gets the count of dependent value update roots.
  - Handler: `dvu_roots::dvu_roots`
  - Request Query: `DvuRootsRequest` (`visibility`)
  - Response: `DvuRootsResponse` (`count`)

### Graphviz Routes (`/api/graphviz`)

_(Defined in `sdf-v1-routes-graphviz`)_

- **`GET /schema_variant`**: Generates graphviz data for a specific schema
  variant.
  - Handler: `schema_variant`
  - Request Query: `SchemaVariantVizRequest` (`visibility`, `schema_variant_id`)
  - Response: `GraphVizResponse` (`nodes`, `edges`, `root_node_id`)
- **`GET /nodes_edges`**: Generates graphviz data for the entire workspace
  snapshot graph.
  - Handler: `nodes_edges`
  - Request Query: `GraphVizRequest` (`visibility`)
  - Response: `GraphVizResponse` (`nodes`, `edges`, `root_node_id`)
- **`GET /components`**: Generates graphviz data focused on components and their
  connections/properties.
  - Handler: `components`
  - Request Query: `GraphVizRequest` (`visibility`)
  - Response: `GraphVizResponse` (`nodes`, `edges`, `root_node_id` is None)

### Qualification Routes (`/api/qualification`)

_(Defined in `sdf-v1-routes-qualification`)_

- **`GET /get_summary`**: Gets a summary of qualification statuses across
  components.
  - Handler: `get_summary::get_summary`
  - Request Query: `GetSummaryRequest` (`visibility`)
  - Response: `QualificationSummaryResponse`

### Secret Routes (`/api/secret`)

_(Defined in `sdf-v1-routes-secret`)_

- **`GET /get_public_key`**: Gets the current public key for secrets.
  - Handler: `get_public_key::get_public_key`
  - Response: `PublicKey`
- **`POST /`**: Creates a new secret.
  - Handler: `create_secret::create_secret`
  - Request: `CreateSecretRequest` (`name`, `definition`, `description`,
    `crypted`, `key_pair_pk`, `version`, `algorithm`, `visibility`)
  - Response: `ForceChangeSetResponse<SecretView>`
- **`GET /`**: Lists all secrets grouped by definition.
  - Handler: `list_secrets::list_secrets`
  - Request Query: `ListSecretRequest` (`visibility`)
  - Response: `ListSecretResponse` (HashMap<String,
    `SecretDefinitionViewWithSecrets`>)
- **`PATCH /`**: Updates an existing secret's metadata or encrypted value.
  - Handler: `update_secret::update_secret`
  - Request: `UpdateSecretRequest` (`id`, `name`, `description`,
    `new_secret_data`, `visibility`)
  - Response: `ForceChangeSetResponse<SecretView>`
- **`DELETE /`**: Deletes a secret.
  - Handler: `delete_secret::delete_secret`
  - Request: `DeleteSecretRequest` (`id`, `visibility`)
  - Response: `ForceChangeSetResponse<()>`

### Session Routes (`/api/session`)

_(Defined in `sdf-v1-routes-session`)_

- **`POST /connect`**: Completes the authentication flow using a code from the
  auth provider.
  - Handler: `auth_connect::auth_connect`
  - Request: `AuthConnectRequest` (`code`, `on_demand_assets`)
  - Response: `AuthConnectResponse` (`user`, `workspace`, `token`)
- **`GET /reconnect`**: Re-establishes session using an existing access token.
  - Handler: `auth_connect::auth_reconnect`
  - Requires `Authorization: Bearer <token>` header.
  - Response: `AuthReconnectResponse` (`user`, `workspace`)
- **`GET /restore_authentication`**: Restores authentication state based on
  token.
  - Handler: `restore_authentication::restore_authentication`
  - Requires `Authorization: Bearer <token>` header.
  - Response: `RestoreAuthenticationResponse` (`user`, `workspace`)
- **`GET /load_workspaces`**: Lists workspaces accessible to the user.
  - Handler: `load_workspaces::load_workspaces`
  - Requires `Authorization: Bearer <token>` header.
  - Response: `LoadWorkspaceResponse` (`workspaces`)
- **`POST /refresh_workspace_members`**: Refreshes workspace members from the
  auth provider.
  - Handler: `refresh_workspace_members::refresh_workspace_members`
  - Request: `RefreshWorkspaceMembersRequest` (`workspace_id`)
  - Requires `Authorization: Bearer <token>` header.
  - Response: `RefreshWorkspaceMembersResponse` (`success`)

### WebSocket Routes (`/api/ws`)

_(Defined in `sdf-v1-routes-ws`)_

- **`GET /workspace_updates`**: Establishes WebSocket connection for real-time
  workspace updates.
  - Handler: `workspace_updates::workspace_updates`
  - Protocol: Streams NATS messages for the workspace; receives client events
    (cursor, online) to publish to NATS.
- **`GET /crdt`**: Establishes WebSocket connection for CRDT synchronization
  (Yjs).
  - Handler: `crdt::crdt`
  - Protocol: Uses Yjs protocol over NATS for collaborative editing.
- **`GET /bifrost`**: Establishes WebSocket connection for another protocol
  (likely data caching).
  - Handler: `bifrost::bifrost_handler`
  - Protocol: Specific protocol handled by `bifrost::proto`.

### Module Routes (`/api/module`)

_(Defined in `sdf-v1-routes-module`)_

- **`POST /install_module`**: Installs modules from the module index.
  - Handler: `install_module::install_module`
  - Request: `InstallModuleRequest` (`ids`, `visibility`)
  - Response: `ForceChangeSetResponse<Vec<FrontendVariant>>`
- **`POST /upgrade_modules`**: Upgrades specified schemas to their latest module
  versions (async).
  - Handler: `upgrade_modules::upgrade_modules`
  - Request: `UpgradeModulesRequest` (`schema_ids`, `visibility`)
  - Response: `ForceChangeSetResponse<Ulid>` (Task ID)
- **`POST /begin_approval_process`**: Begins the approval process for importing
  a workspace backup.
  - Handler: `approval_process::begin_approval_process`
  - Request: `BeginImportFlow` (`id`)
  - Response: `()` (Success)
- **`POST /cancel_approval_process`**: Cancels the workspace import approval
  process.
  - Handler: `approval_process::cancel_approval_process`
  - Response: `()` (Success)
- **`POST /import_workspace_vote`**: Records a user's vote on importing a
  workspace.
  - Handler: `import_workspace_vote::import_workspace_vote`
  - Request: `ImportVoteRequest` (`vote`)
  - Response: `()` (Success)

### Variant Routes (`/api/variant`)

_(Defined in `sdf-v1-routes-variant`)_

- **`POST /create_variant`**: Creates a new schema and default variant.
  - Handler: `create_variant::create_variant`
  - Request: `CreateVariantRequest` (`name`, `color`, `visibility`)
  - Response: `ForceChangeSetResponse<FrontendVariant>`
- **`POST /regenerate_variant`**: Regenerates a schema variant based on updated
  asset definition code.
  - Handler: `regenerate_variant::regenerate_variant`
  - Request: `RegenerateVariantRequest` (`variant`, `code`, `visibility`)
  - Response: `ForceChangeSetResponse<RegenerateVariantResponse>`
    (`schema_variant_id`)
- **`POST /clone_variant`**: Clones an existing schema variant into a new
  schema.
  - Handler: `clone_variant::clone_variant`
  - Request: `CloneVariantRequest` (`id`, `name`, `visibility`)
  - Response: `ForceChangeSetResponse<FrontendVariant>`
- **`POST /save_variant`**: Saves changes to an unlocked schema variant's
  content (metadata, code).
  - Handler: `save_variant::save_variant`
  - Request: `SaveVariantRequest` (`variant`, `code`, `visibility`)
  - Response: `ForceChangeSetResponse<SaveVariantResponse>` (`success`)

## V2 Endpoints (`/api/v2`)

These routes are defined within `sdf-server/src/service/v2/`.

### Admin Routes (`/api/v2/admin`)

_(Requires SystemInit user)_

- **`POST /update_module_cache`**: Triggers an async update of the module cache.
  - Handler: `update_module_cache::update_module_cache`
  - Response: `UpdateModuleCacheResponse` (`id` - Task ID)
- **`PUT /func/runs/:func_run_id/kill_execution`**: Kills a specific function
  execution.
  - Handler: `kill_execution::kill_execution`
  - Response: `()` (Success)
- **`GET /workspaces`**: Searches for workspaces.
  - Handler: `search_workspaces::search_workspaces`
  - Request Query: `SearchWorkspacesRequest` (`query`)
  - Response: `SearchWorkspacesResponse` (`workspaces`)
- **`POST /workspaces/:workspace_id/set_concurrency_limit`**: Sets the component
  concurrency limit for a workspace.
  - Handler: `set_concurrency_limit::set_concurrency_limit`
  - Request: `SetComponentConcurrencyLimitRequest` (`concurrency_limit`)
  - Response: `SetComponentConcurrencyLimitResponse` (`concurrency_limit`)
- **`GET /workspaces/:workspace_id/change_sets`**: Lists all change sets for a
  workspace.
  - Handler: `list_change_sets::list_change_sets`
  - Response: `ListChangesetsResponse` (`change_sets`)
- **`GET /workspaces/:workspace_id/change_sets/:change_set_id/get_snapshot`**:
  Downloads a workspace snapshot.
  - Handler: `get_snapshot::get_snapshot`
  - Response: Base64 encoded snapshot data (octet-stream)
- **`POST /workspaces/:workspace_id/change_sets/:change_set_id/set_snapshot`**:
  Uploads and sets a workspace snapshot.
  - Handler: `set_snapshot::set_snapshot`
  - Request: Multipart form data containing the snapshot file.
  - Response: `SetSnapshotResponse` (`workspace_snapshot_address`)

### Workspace Routes (`/api/v2/workspaces/{workspace_id}`)

- **`POST /install`**: Installs a workspace from a backup (likely related to
  import).
  - Handler: `install_workspace::install_workspace`
  - (Request/Response details not fully captured from `lib.rs`)
- **`GET /users`**: Lists users associated with the workspace.
  - Handler: `list_workspace_users::list_workspace_users`
  - (Request/Response details not fully captured from `lib.rs`)

### Change Set Routes (`/api/v2/workspaces/{workspace_id}/change-sets`)

- **`GET /`**: Lists actionable change sets (likely non-HEAD, active ones).
  - Handler: `list::list_actionable`
  - (Request/Response details not fully captured from `lib.rs`)

### Change Set Specific Routes (`/api/v2/workspaces/{workspace_id}/change-sets/{change_set_id}`)

- **`POST /apply`**: Applies the change set to HEAD.
  - Handler: `apply::apply`
- **`GET /approval_status`**: Gets the approval status of the change set.
  - Handler: `approval_status::approval_status`
- **`POST /approve`**: Approves the change set.
  - Handler: `approve::approve`
- **`POST /cancel_approval_request`**: Cancels the request for approval.
  - Handler: `cancel_approval_request::cancel_approval_request`
- **`POST /force_apply`**: Force applies the change set (requires permission).
  - Handler: `force_apply::force_apply`
- **`POST /rename`**: Renames the change set.
  - Handler: `rename::rename`
- **`POST /reopen`**: Reopens a closed/applied change set.
  - Handler: `reopen::reopen`
- **`POST /request_approval`**: Requests approval for the change set.
  - Handler: `request_approval::request_approval`
- **Nested Routes**: Further routes under `/audit-logs`, `/funcs`, `/modules`,
  `/schema-variants`, `/management`, `/views`,
  `/approval-requirement-definitions`, `/index`.

### FS Routes (`/api/v2/workspaces/{workspace_id}/fs`)

_(Provides a filesystem-like interface for editing assets)_

- **`GET /hydrate`**: Gets initial data for the FS interface (change sets,
  schemas, funcs).
  - Handler: `hydrate`
  - Response: Vec<`HydratedChangeSet`>
- **`GET /change-sets`**: Lists active change sets.
  - Handler: `list_change_sets`
  - Response: `ListChangeSetsResponse`
- **`POST /change-sets/create`**: Creates a new change set.
  - Handler: `create_change_set`
  - Request: `CreateChangeSetRequest` (`name`)
  - Response: `CreateChangeSetResponse`
- **Nested Routes under `/change-sets/:change_set_id/`**: Numerous routes for
  managing funcs (code, types, bindings, unlock), schemas (list, categories,
  create, asset funcs, attrs, unlock, bindings, install), similar to V1 but
  potentially structured differently.

### Index Routes (`/api/v2/workspaces/{workspace_id}/index`)

- **`GET /`**: Gets the index data for the workspace (likely HEAD).
  - Handler: `get_workspace_index::get_workspace_index`

### Index Routes (`/api/v2/workspaces/{workspace_id}/change-sets/{change_set_id}/index`)

- **`GET /`**: Gets the index data for a specific change set.
  - Handler: `get_change_set_index::get_change_set_index`
- **`GET /mjolnir`**: Gets the frontend object representation for the index.
  - Handler: `get_front_end_object::get_front_end_object`
- **`POST /rebuild`**: Triggers a rebuild of the index for the change set.
  - Handler: `rebuild_change_set_index::rebuild_change_set_index`

### Integrations Routes (`/api/v2/workspaces/{workspace_id}/integrations`)

- **`POST /:workspace_integration_id`**: Updates a specific workspace
  integration.
  - Handler: `update_integration::update_integration`
- **`GET /`**: Gets workspace integration details.
  - Handler: `get_integrations::get_integration`

### Audit Log Routes (`/api/v2/workspaces/{workspace_id}/change-sets/{change_set_id}/audit-logs`)

- **`GET /`**: Lists audit logs for the change set.
  - Handler: `list_audit_logs::list_audit_logs`

### Func Routes (`/api/v2/workspaces/{workspace_id}/change-sets/{change_set_id}/funcs`)

_(See V1 Func routes for likely overlap; specific handlers might differ)_

- Routes for listing, getting code, getting runs, creating, updating, saving
  code, testing, executing, unlocking, deleting funcs, managing bindings and
  arguments.

### Module Routes (`/api/v2/workspaces/{workspace_id}/change-sets/{change_set_id}/modules`)

- **`POST /contribute`**: Contributes a module.
  - Handler: `contribute::contribute`
- **`GET /sync`**: Synchronizes modules with the index.
  - Handler: `sync::sync`
- **`GET /`**: Lists available modules.
  - Handler: `list::list`
- **`POST /:module_id/builtins/reject`**: Rejects a builtin module update.
  - Handler: `builtins::reject`
- **`POST /:module_id/builtins/promote`**: Promotes a builtin module update.
  - Handler: `builtins::promote`
- **`GET /module_by_hash`**: Gets module details by hash.
  - Handler: `module_by_hash::module_by_hash`
- **`GET /module_by_id`**: Gets remote module details by ID.
  - Handler: `module_by_id::remote_module_by_id`
- **`POST /install_from_file`**: Installs a module from an uploaded file.
  - Handler: `install_from_file::install_module_from_file`

### Schema Variant Routes (`/api/v2/workspaces/{workspace_id}/change-sets/{change_set_id}/schema-variants`)

- **`GET /`**: Lists schema variants.
  - Handler: `list_variants::list_variants`
- **`GET /:schema_variant_id`**: Gets details for a specific schema variant.
  - Handler: `get_variant::get_variant`
- **`POST /:schema_variant_id`**: Creates an unlocked copy of a schema variant.
  - Handler: `create_unlocked_copy::create_unlocked_copy`
- **`DELETE /:schema_variant_id`**: Deletes an unlocked schema variant.
  - Handler: `delete_unlocked_variant::delete_unlocked_variant`

### Management Routes (`/api/v2/workspaces/{workspace_id}/change-sets/{change_set_id}/management`)

- **`POST /prototype/:prototypeId/:componentId/:viewId`**: Runs a management
  prototype function.
  - Handler: `run_prototype`
- **`GET /prototype/:prototypeId/:componentId/latest`**: Gets the latest run
  details for a management prototype.
  - Handler: `latest::latest`
- **`GET /history`**: Gets the history of management function runs.
  - Handler: `history::history`
- **`POST /generate_template/:viewId`**: Generates a template based on a view.
  - Handler: `generate_template::generate_template`

### View Routes (`/api/v2/workspaces/{workspace_id}/change-sets/{change_set_id}/views`)

- **`GET /`**: Lists views in the change set.
  - Handler: `list_views::list_views`
- **`POST /`**: Creates a new view.
  - Handler: `create_view::create_view`
- **`POST /create_and_move`**: Creates a new view and moves components to it.
  - Handler: `create_view_and_move::create_view_and_move`
- **`POST /convert_to_view`**: Converts a component (frame) into a view.
  - Handler: `convert_to_view::convert_to_view`
- **`PUT /:view_id`**: Updates view properties.
  - Handler: `update_view::update_view`
- **`DELETE /:view_id`**: Removes a view.
  - Handler: `remove_view::remove_view`
- **`GET /:view_id/get_diagram`**: Gets diagram data for a specific view.
  - Handler: `get_diagram::get_diagram`
- **`GET /:view_id/get_geometry`**: Gets geometry data for a specific view.
  - Handler: `get_diagram::get_geometry`
- **`GET /default/get_diagram`**: Gets diagram data for the default view.
  - Handler: `get_diagram::get_default_diagram`
- **`POST /:view_id/component`**: Creates a component within a specific view.
  - Handler: `create_component::create_component`
- **`POST /:view_id/paste_components`**: Pastes components into a view.
  - Handler: `paste_component::paste_component`
- **`DELETE /:view_id/erase_components`**: Erases components from a view
  (removes geometry).
  - Handler: `erase_components::erase_components`
- **`PUT /:view_id/component/set_geometry`**: Sets the geometry of components
  within a view.
  - Handler: `set_geometry::set_component_geometry`
- **`PUT /:view_id/component/set_parent`**: Sets the parent frame for components
  within a view.
  - Handler: `set_component_parent::set_component_parent`
- **`POST /:view_id/view_object`**: Creates a view object (e.g., text box).
  - Handler: `create_view_object::create_view_object`
- **`DELETE /:view_id/view_object`**: Erases a view object.
  - Handler: `erase_view_object::erase_view_object`
- **`PUT /:view_id/view_object/set_geometry`**: Sets the geometry of a view
  object.
  - Handler: `set_geometry::set_view_object_geometry`

### Approval Requirement Definition Routes (`/api/v2/workspaces/{workspace_id}/change-sets/{change_set_id}/approval-requirement-definitions`)

- **`PUT /`**: Creates a new approval requirement definition.
  - Handler: `new::new`
- **`GET /entity/:entity-id`**: Lists definitions for a specific entity.
  - Handler: `list::list_for_entity`
- **`DELETE /:id`**: Removes a definition.
  - Handler: `remove::remove`
- **`PUT /:id/individual-approver/:user-id`**: Adds an individual user as an
  approver.
  - Handler: `add_individual_approver::add_individual_approver`
- **`DELETE /:id/individual-approver/:user-id`**: Removes an individual user as
  an approver.
  - Handler: `remove_individual_approver::remove_individual_approver`

## Public API Endpoints (`/api/public/v0`)

These routes are defined within `sdf-server/src/service/public/`.

### Change Set Routes (`/api/public/v0/workspaces/{workspace_id}/change-sets`)

- **`POST /`**: Creates a new change set.
  - Handler: `create_change_set`
  - Request: `CreateChangeSetRequest` (`change_set_name`)
  - Response: `CreateChangeSetResponse` (`change_set`)
- **Nested Routes under `/:change_set_id/`**:
  - **`GET /merge_status`**: Gets merge status and actions.
    - Handler: `merge_status`
    - Response: `MergeStatusResponse`
  - **`POST /force_apply`**: Force applies the change set.
    - Handler: `force_apply`
  - **`POST /request_approval`**: Requests approval for the change set.
    - Handler: `request_approval`
  - **Nested Component Routes (`/components`)**:
    - **`GET /:component_id`**: Gets component details.
      - Handler: `get_component`
      - Response: `GetComponentResponse`
    - **`PUT /:component_id/properties`**: Updates component properties.
      - Handler: `update_component_properties`
      - Request: `UpdateComponentPropertiesRequest` (`domain`)
      - Response: `UpdateComponentPropertiesResponse`
  - **Nested Management Routes (`/management`)**:
    - **`POST /prototype/:management_prototype_id/:component_id/:view_id`**:
      Runs a management prototype.
      - Handler: `run_prototype`
      - Request: `RunPrototypeRequest` (`request_ulid`)
      - Response: `RunPrototypeResponse` (`status`, `message`)

## Dev Endpoints (`/api/dev`)

_(Only available in debug builds)_

- **`GET /get_current_git_sha`**: Gets the current git SHA of the running build.
  - Handler: `get_current_git_sha::get_current_git_sha`
