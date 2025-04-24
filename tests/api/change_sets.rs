// tests/api/change_sets.rs

// Intention:
// Declares test modules for the change set related API endpoints.
// Each submodule corresponds to a file containing a single test function or helpers.

// Design Choices:
// - Follows the one-function-per-file rule for tests.
// - This file now only contains module declarations.

// Declare helper module
mod helpers;

// Declare test function modules
mod test_abandon_change_set_endpoint;
mod test_create_change_set_endpoint;
mod test_force_apply_endpoint;
mod test_get_change_set_endpoint;
mod test_get_merge_status_endpoint;
mod test_list_change_sets_endpoint;

// Note: The original file contained imports (std::env, chrono::Utc, dotenvy, situation::*, tokio::time::sleep)
// and a #[cfg(test)] mod tests { ... } block. These are no longer needed here as the actual
// test code and necessary imports reside within the submodules.
