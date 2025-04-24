// tests/unit/api_models/test_deserialize_change_set_summary.rs

// Intention: Test deserialization of the ChangeSetSummary model.

use situation::ChangeSetSummary; // Use the library crate namespace

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
