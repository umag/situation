// tests/unit/api_models/test_deserialize_list_change_set_response.rs

// Intention: Test deserialization of the ListChangeSetV1Response model.

use situation::ListChangeSetV1Response; // Use the library crate namespace

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
