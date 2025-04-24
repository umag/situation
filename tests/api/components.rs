// tests/api/components.rs

// Intention:
// Declares test modules for the component related API endpoints.
// Each submodule corresponds to a file containing a single test function or helpers.

// Design Choices:
// - Follows the one-function-per-file rule for tests.
// - This file now only contains module declarations.

// Declare helper module
mod helpers;

// Declare test function module(s)
mod test_component_crud_endpoints;

// Note: The original file contained imports (std::env, chrono::Utc, dotenvy, situation::*, tokio::time::sleep, serde_json)
// and a #[cfg(test)] mod tests { ... } block. These are no longer needed here as the actual
// test code and necessary imports reside within the submodules.
