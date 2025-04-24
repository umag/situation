// tests/unit/api_models/test_deserialize_api_error.rs

// Intention: Test deserialization of the ApiError model.

use situation::ApiError; // Use the library crate namespace

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
