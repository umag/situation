// tests/unit/app_state/test_app_change_set_nav_empty.rs

// Intention: Test change set list navigation when the list is empty.

use situation::App; // Assuming App is accessible

// Import helper function from the same directory
use super::helpers::create_dummy_change_sets;

// Test change set navigation with empty list
#[test]
fn test_app_change_set_nav_empty() {
    let mut app = App::new();
    app.change_sets = Some(create_dummy_change_sets(0)); // Empty list

    app.change_set_next();
    assert!(app.change_set_list_state.selected().is_none()); // Stays None
    app.change_set_previous();
    assert!(app.change_set_list_state.selected().is_none()); // Stays None
}
