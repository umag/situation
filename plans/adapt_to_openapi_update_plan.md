# Plan for Adapting to Updated OpenAPI Specification

## 1. Analysis Summary

- **Renamed Operations:**
  - `DELETE /v1/.../change-sets/{id}`: `delete_change_set` ->
    `abandon_change_set`
  - `POST /v1/.../change-sets/{id}/force_apply`: `force_apply_change_set` ->
    `force_apply`
- **Changed Response:**
  - `DeleteChangeSetV1Response`: Now `{"success": boolean}`.
- **Verification Needed:**
  - `GetChangeSetV1Response` structure (`changeSet: object`).
  - `ListChangeSetV1Response` structure vs. `ChangeSetSummary` model.
- **New Endpoints:** Components, Management Prototypes, Status, Approvals.

## 2. Scope of Changes (Current Task)

- Adapt existing delete and force-apply functionality to new operation IDs and
  response schemas.
- Verify/update models for list/get change sets.
- Implement API client, models, and tests for new Component endpoints (Create,
  Get, Update, Delete).
- Update code, tests, and documentation.
- **Note:** TUI integration for component management is deferred to a future
  task.

## 3. Detailed Implementation Steps

### API Client (`src/api_client.rs`)

- Rename `delete_change_set` -> `abandon_change_set`.
- Update `abandon_change_set` return type to
  `Result<(DeleteChangeSetV1Response, Vec<String>), ...>`.
- Update `abandon_change_set` implementation to deserialize the new response.
- Rename `force_apply_change_set` -> `force_apply`.

### API Models (`src/api_models.rs`)

- Define/Update `DeleteChangeSetV1Response`:
  ```rust
  #[derive(Deserialize, Debug, Clone)]
  pub struct DeleteChangeSetV1Response {
      pub success: bool,
  }
  ```
- Verify `ChangeSetSummary` structure.
- Verify `ChangeSet` structure.

### TUI Logic (`src/run_app.rs`)

- Update calls: `delete_change_set` -> `abandon_change_set`.
- Update result handling for `abandon_change_set` (check `response.success`).
- Update calls: `force_apply_change_set` -> `force_apply`.

### Tests (`tests/api/change_sets.rs`, etc.)

- Rename delete test functions (e.g., `test_delete_change_set` ->
  `test_abandon_change_set`).
- Update assertions in abandon tests for
  `DeleteChangeSetV1Response { success: true }`.
- Rename force-apply test functions (e.g., `test_force_apply_change_set` ->
  `test_force_apply`).
- Review/update list/get change set tests.

### Component API (`src/api_models.rs`, `src/api_client.rs`, `tests/api/`)

- **Models (`src/api_models.rs`):**
  - Define `CreateComponentV1Request`
  - Define `CreateComponentV1Response`
  - Define `GetComponentV1Response` (and nested structs like
    `GetComponentV1ResponseManagementFunction`, `GeometryAndViewAndName`)
  - Define `UpdateComponentV1Request`
  - Define `UpdateComponentV1Response`
  - Define `DeleteComponentV1Response`
  - Define shared/nested structs: `ComponentReference`, `ConnectionPoint`,
    `Connection`, `DomainPropPath`, `ComponentPropKey`.
- **Client (`src/api_client.rs`):**
  - Implement `create_component(workspace_id, change_set_id, request_body)`
  - Implement `get_component(workspace_id, change_set_id, component_id)`
  - Implement
    `update_component(workspace_id, change_set_id, component_id, request_body)`
  - Implement `delete_component(workspace_id, change_set_id, component_id)`
- **Tests (`tests/api/components.rs` - new file):**
  - Add integration tests for `create_component`.
  - Add integration tests for `get_component`.
  - Add integration tests for `update_component`.
  - Add integration tests for `delete_component`.
  - Declare `components` module in `tests/api/mod.rs`.

### Specification (`specs/system_specification.txt`)

- Update function names (`abandon_change_set`, `force_apply`).
- Update `DELETE` endpoint description (operationId, response).
- Update `POST /force_apply` endpoint description (operationId).
- Review/update model descriptions (`GetChangeSetV1Response`,
  `ListChangeSetV1Response`, `ChangeSet`, `ChangeSetSummary`).
- Add descriptions for new Component endpoints, client functions, and models.
