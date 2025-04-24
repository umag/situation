// tests/unit/api_models/test_deserialize_whoami_response.rs

// Intention: Test deserialization of the WhoamiResponse model.

use situation::WhoamiResponse; // Use the library crate namespace

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
