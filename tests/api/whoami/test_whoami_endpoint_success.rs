// tests/api/whoami/test_whoami_endpoint_success.rs

// Intention:
// Contains the integration test for a successful call to the /whoami API endpoint.

// Design Choices:
// - Extracted from tests/api/whoami.rs to follow the one-function-per-file rule.
// - Uses standard Rust test conventions (`#[tokio::test]`).
// - Calls the actual `whoami` function from the library crate (`situation`).
// - Asserts that the call succeeds and the response contains expected data.
// - Requires a valid `.env` file with `SI_API` and `JWT_TOKEN` for the test to pass.

use situation::whoami; // Import the function from the library crate

/// Test Case: Verify the `/whoami` endpoint call.
/// Intention: Ensure the application can correctly call the `/whoami` endpoint
///            using the library function and handle a successful response.
/// Design: Calls `situation::whoami().await` and asserts that the result is Ok,
///         the WhoamiResponse contains non-empty user/workspace info, and logs are returned.
#[tokio::test]
// #[ignore = "Requires valid .env configuration and running API"] // Keep comment for context
async fn test_whoami_endpoint_success() {
    // Ensure .env is loaded (dotenvy is called within get_api_config)
    let result = whoami().await;

    assert!(
        result.is_ok(),
        "API call to /whoami failed: {:?}",
        result.err()
    );

    if let Ok((response, logs)) = result {
        // Check that essential fields are present and not empty
        assert!(!response.user_id.is_empty(), "User ID should not be empty");
        assert!(
            !response.user_email.is_empty(),
            "User Email should not be empty"
        );
        assert!(
            !response.workspace_id.is_empty(),
            "Workspace ID should not be empty"
        );

        // Check token details (basic check)
        assert!(
            !response.token.sub.is_empty(),
            "Token subject should not be empty"
        );
        assert!(
            !response.token.user_pk.is_empty(),
            "Token user_pk should not be empty"
        );
        assert!(
            !response.token.workspace_pk.is_empty(),
            "Token workspace_pk should not be empty"
        );
        assert!(response.token.iat > 0, "Token iat should be positive");

        // Check that logs were generated
        assert!(!logs.is_empty(), "Logs should have been generated");
        assert!(
            logs.iter().any(|log| log.contains("Calling API: GET")),
            "Logs should contain API call info"
        );
        assert!(
            logs.iter()
                .any(|log| log.contains("API Response Status: 200 OK")),
            "Logs should contain success status"
        );
        assert!(
            logs.iter().any(|log| log.contains("API Success Body:")),
            "Logs should contain success body"
        );
    }
}

// TODO: Add tests for error cases (e.g., invalid token -> 401) when error handling is more robust.
// These would go into separate files like tests/api/whoami/test_whoami_endpoint_unauthorized.rs
