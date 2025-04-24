// tests/unit/api_models.rs

// Intention:
// Declares unit test modules for the API data models.
// Each submodule corresponds to a file containing a single test function.

// Design Choices:
// - Follows the one-function-per-file rule for tests.
// - This file now only contains module declarations.

// Declare test function modules
mod test_deserialize_api_error;
mod test_deserialize_api_error_null_code;
mod test_deserialize_change_set_summary;
mod test_deserialize_list_change_set_response;
mod test_deserialize_list_change_set_response_empty;
mod test_deserialize_token_details;
mod test_deserialize_whoami_response;

// Note: The original file contained imports (situation::*) and the test functions.
// These are no longer needed here as the actual test code and necessary imports
// reside within the submodules.
