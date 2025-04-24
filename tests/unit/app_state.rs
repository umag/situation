// tests/unit/app_state.rs

// Intention:
// Declares unit test modules for the application state logic (App struct).
// Each submodule corresponds to a file containing a single test function or helpers.

// Design Choices:
// - Follows the one-function-per-file rule for tests.
// - This file now only contains module declarations.

// Declare helper module
mod helpers;

// Declare test function modules
mod test_app_add_log;
mod test_app_add_log_auto_scroll;
mod test_app_change_set_nav_empty;
mod test_app_change_set_nav_none;
mod test_app_change_set_next;
mod test_app_change_set_previous;
mod test_app_log_scroll;
mod test_app_new;
mod test_app_select_change_set_by_id;

// Note: The original file contained imports (ratatui::widgets::ListState, situation::*, situation::api_models::*)
// and the test functions. These are no longer needed here as the actual test code and necessary imports
// reside within the submodules.
