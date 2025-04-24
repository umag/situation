// tests/unit/api_models/test_deserialize_token_details.rs

// Intention: Test deserialization of the TokenDetails model.

use situation::TokenDetails; // Use the library crate namespace

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
