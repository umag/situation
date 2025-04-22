# Plan: Implement Remaining API Calls

**Goal:** Implement the remaining API calls defined in `openapi.json`.

**General Approach for Each Endpoint:** For each API endpoint listed below, the
following sub-steps will be performed:

1. **Model Definition (`src/api_models.rs`):**
   - Define necessary Rust structs for request and response bodies based on
     `openapi.json`.
   - Derive `serde::Deserialize`, `serde::Serialize`, `Debug`, `Clone`.
   - Add documentation comments (intention/design) to structs and fields.
2. **API Client Function (`src/api_client.rs`):**
   - Define an `async` function in the `ApiClient` implementation.
   - Function signature will include necessary parameters (IDs, request body
     struct).
   - Use the shared `reqwest::Client` and configuration.
   - Construct the URL and perform the HTTP request.
   - Implement error handling and logging similar to the existing `whoami`
     function.
   - Add documentation comments (intention/design) to the function.
3. **Testing (`tests/api_calls.rs`):**
   - Add a new `#[tokio::test]` function.
   - Call the new API client function.
   - Add assertions for success (and potentially response data, though this
     might require mocking or a live API).
   - Add documentation comments (intention/design) to the test.
4. **Specification Update (`specs/system_specification.txt`):**
   - Update the specification document to reflect the newly added API client
     function and its status.

**Step-by-Step Implementation Plan (Prioritized):**

1. **Implement `GET /v1/w/{workspace_id}/change-sets` (list_change_sets)**
   - Models: `ListChangeSetV1Response` and nested structs.
   - Client:
     `async fn list_change_sets(&self, workspace_id: &str) -> Result<ListChangeSetV1Response, ApiClientError>`
   - Test: `async fn test_list_change_sets()`
   - Docs & Spec updates.

2. **Implement `POST /v1/w/{workspace_id}/change-sets` (create_change_set)**
   - Models: `CreateChangeSetV1Request`, `CreateChangeSetV1Response`.
   - Client:
     `async fn create_change_set(&self, workspace_id: &str, request: CreateChangeSetV1Request) -> Result<CreateChangeSetV1Response, ApiClientError>`
   - Test: `async fn test_create_change_set()`
   - Docs & Spec updates.

3. **Implement `GET /v1/w/{workspace_id}/change-sets/{change_set_id}`
   (get_change_set)**
   - Models: `GetChangeSetV1Response`.
   - Client:
     `async fn get_change_set(&self, workspace_id: &str, change_set_id: &str) -> Result<GetChangeSetV1Response, ApiClientError>`
   - Test: `async fn test_get_change_set()`
   - Docs & Spec updates.

4. **Implement `DELETE /v1/w/{workspace_id}/change-sets/{change_set_id}`
   (abandon_change_set)**
   - Models: `DeleteChangeSetV1Response`.
   - Client:
     `async fn abandon_change_set(&self, workspace_id: &str, change_set_id: &str) -> Result<DeleteChangeSetV1Response, ApiClientError>`
   - Test: `async fn test_abandon_change_set()`
   - Docs & Spec updates.

5. **Implement
   `GET /v1/w/{workspace_id}/change-sets/{change_set_id}/merge_status`
   (merge_status)**
   - Models: `MergeStatusV1Response` and nested structs.
   - Client:
     `async fn merge_status(&self, workspace_id: &str, change_set_id: &str) -> Result<MergeStatusV1Response, ApiClientError>`
   - Test: `async fn test_merge_status()`
   - Docs & Spec updates.

6. **Implement
   `POST /v1/w/{workspace_id}/change-sets/{change_set_id}/force_apply`
   (force_apply)**
   - Models: Define simple success struct if needed based on actual API
     response.
   - Client:
     `async fn force_apply(&self, workspace_id: &str, change_set_id: &str) -> Result<(), ApiClientError>`
     (adjust return type based on response)
   - Test: `async fn test_force_apply()`
   - Docs & Spec updates.

7. **Implement Component Endpoints:**
   - `GET /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}`
     (get_component)
   - `PUT /v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/properties`
     (update_component_properties)
   - Follow sub-steps (Models, Client, Test, Docs, Spec) for each.

8. **Implement Other Endpoints:**
   - `POST /v1/w/{workspace_id}/change-sets/{change_set_id}/management/prototype/{management_prototype_id}/{component_id}/{view_id}`
     (run_prototype)
   - `POST /v1/w/{workspace_id}/change-sets/{change_set_id}/request_approval`
     (request_approval)
   - `GET /` (system_status_route) - Lower priority.
   - Follow sub-steps (Models, Client, Test, Docs, Spec) for each.

**Next Steps:** This plan outlines the implementation strategy. The next logical
action would be to start implementing Step 1 (`list_change_sets`).
