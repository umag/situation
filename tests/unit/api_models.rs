// tests/unit/api_models.rs

// Intention:
// Contains unit tests for the API data models defined in src/api_models.rs.
// Focuses on ensuring correct deserialization from example JSON strings.

// Design Choices:
// - Uses standard Rust test conventions (`#[cfg(test)]`, `#[test]`).
// - Imports necessary models from the library crate (`situation`).
// - Uses `serde_json::from_str` to test deserialization.

// Use `situation::` to reference items from the library crate.
use situation::{
    ApiError, ChangeSetSummary, ListChangeSetV1Response, TokenDetails,
    WhoamiResponse,
};

#[test]
fn test_deserialize_token_details() {
    let json = r#"{
        "iat": 1745271246,
        "sub": "user_subject_id",
        "user_pk": "user_pk_123",
        "workspace_pk": "ws_pk_456"
    }"#;
    let details: TokenDetails =
        serde_json::from_str(json).expect("Failed to deserialize TokenDetails");
    assert_eq!(details.iat, 1745271246);
    assert_eq!(details.sub, "user_subject_id");
    assert_eq!(details.user_pk, "user_pk_123");
    assert_eq!(details.workspace_pk, "ws_pk_456");
}

#[test]
fn test_deserialize_whoami_response() {
    // Based on actual logged response structure
    let json = r#"{
        "userId": "01H7ZHE0XPPRD0MBH0BTJ6BW4M",
        "userEmail": "i+si@aopab.art",
        "workspaceId": "01JSD4BDWX6326J9Z4YVCAD4J3",
        "token": {
            "iat": 1745271246,
            "sub": "01H7ZHE0XPPRD0MBH0BTJ6BW4M",
            "user_pk": "01H7ZHE0XPPRD0MBH0BTJ6BW4M",
            "workspace_pk": "01JSD4BDWX6326J9Z4YVCAD4J3"
        }
    }"#;
    let response: WhoamiResponse = serde_json::from_str(json)
        .expect("Failed to deserialize WhoamiResponse");
    assert_eq!(response.user_id, "01H7ZHE0XPPRD0MBH0BTJ6BW4M");
    assert_eq!(response.user_email, "i+si@aopab.art");
    assert_eq!(response.workspace_id, "01JSD4BDWX6326J9Z4YVCAD4J3");
    assert_eq!(response.token.sub, "01H7ZHE0XPPRD0MBH0BTJ6BW4M");
    assert_eq!(response.token.workspace_pk, "01JSD4BDWX6326J9Z4YVCAD4J3");
}

#[test]
fn test_deserialize_api_error() {
    let json = r#"{
        "code": 123,
        "message": "Something went wrong",
        "statusCode": 500
    }"#;
    let error: ApiError =
        serde_json::from_str(json).expect("Failed to deserialize ApiError");
    assert_eq!(error.code, Some(123));
    assert_eq!(error.message, "Something went wrong");
    assert_eq!(error.status_code, 500);
}

#[test]
fn test_deserialize_api_error_null_code() {
    let json = r#"{
        "code": null,
        "message": "Another error",
        "statusCode": 404
    }"#;
    let error: ApiError = serde_json::from_str(json)
        .expect("Failed to deserialize ApiError with null code");
    assert_eq!(error.code, None);
    assert_eq!(error.message, "Another error");
    assert_eq!(error.status_code, 404);
}

#[test]
fn test_deserialize_change_set_summary() {
    let json = r#"{
        "id": "cs_id_1",
        "name": "My Change Set",
        "status": "Draft"
    }"#;
    let summary: ChangeSetSummary = serde_json::from_str(json)
        .expect("Failed to deserialize ChangeSetSummary");
    assert_eq!(summary.id, "cs_id_1");
    assert_eq!(summary.name, "My Change Set");
    assert_eq!(summary.status, "Draft");
}

#[test]
fn test_deserialize_list_change_set_response() {
    // Based on example in openapi.json
    let json = r#"{
        "changeSets": [
            {
                "id": "01H9ZQD35JPMBGHH69BT0Q79VY",
                "name": "Add new feature",
                "status": "Draft"
            },
            {
                "id": "cs_id_2",
                "name": "Another CS",
                "status": "Applied"
            }
        ]
    }"#;
    let response: ListChangeSetV1Response = serde_json::from_str(json)
        .expect("Failed to deserialize ListChangeSetV1Response");
    assert_eq!(response.change_sets.len(), 2);
    assert_eq!(response.change_sets[0].id, "01H9ZQD35JPMBGHH69BT0Q79VY");
    assert_eq!(response.change_sets[0].name, "Add new feature");
    assert_eq!(response.change_sets[1].status, "Applied");
}

#[test]
fn test_deserialize_list_change_set_response_empty() {
    let json = r#"{ "changeSets": [] }"#;
    let response: ListChangeSetV1Response = serde_json::from_str(json)
        .expect("Failed to deserialize empty ListChangeSetV1Response");
    assert!(response.change_sets.is_empty());
}
