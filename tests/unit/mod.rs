// tests/unit/mod.rs

// Intention:
// This file declares the modules containing unit tests.
// It allows the Rust test runner to discover tests within the `tests/unit/` subdirectory.

// Declare the module containing api_models unit tests.
pub mod api_models;
// Note: ui_rendering tests moved into src/ui.rs as inline module #[cfg(test)]

// Add declarations for other unit test modules here as they are created.
// Note: Removed app_state module as testing main binary internals from tests/ is complex.
