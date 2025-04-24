// tests/unit/api_models/test_deserialize_list_change_set_response_empty.rs

// Intention: Test deserialization of the ListChangeSetV1Response model when the list is empty.

use situation::ListChangeSetV1Response; // Use the library crate namespace

#[test]
fn test_deserialize_list_change_set_response_empty() {
    let json = r#"{ "changeSets": [] }"#;
    let response: ListChangeSetV1Response = serde_json::from_str(json)
        .expect("Failed to deserialize empty ListChangeSetV1Response");
    assert!(response.change_sets.is_empty());
}
