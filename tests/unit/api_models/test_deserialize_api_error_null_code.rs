// tests/unit/api_models/test_deserialize_api_error_null_code.rs

// Intention: Test deserialization of the ApiError model when the 'code' field is null.

use situation::ApiError; // Use the library crate namespace

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
