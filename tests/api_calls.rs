// tests/api_calls.rs

// Intention:
// This file serves as the entry point for integration tests.
// It declares test modules located in subdirectories (like `tests/api/`).
// Specific tests related to API calls are organized within those submodules.

// Declare the module containing API-specific integration tests.
// This corresponds to the `tests/api/` directory and its `mod.rs` file.
mod api;

// Declare the module containing unit tests.
// This corresponds to the `tests/unit/` directory and its `mod.rs` file.
mod unit;

// Design Choices:
// - Uses standard Rust test conventions (`#[cfg(test)]`, `#[test]`).
// - Each test function focuses on a specific API endpoint or functionality.
// - Placeholder tests are used initially where the exact implementation details
//   (like using the `luminork` client) are pending clarification.

#[cfg(test)]
mod tests {
    // This file remains as the entry point for tests declared in submodules (api, unit).
    // Specific tests are now located in their respective modules.
    // Example: tests/api/whoami.rs, tests/api/change_sets.rs, tests/unit/api_models.rs

    // TODO: Add general integration tests here if needed, that don't fit specific API endpoints.
    // Examples (moved to specific modules):
    // - test_create_change_set() // -> tests/api/change_sets.rs
    // - test_get_component()     // -> tests/api/components.rs (example)
}
