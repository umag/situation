// tests/unit/app_state/test_app_change_set_nav_none.rs

// Intention: Test change set list navigation when the change_sets field is None.

use situation::App; // Assuming App is accessible

// Test change set navigation when change_sets is None
#[test]
fn test_app_change_set_nav_none() {
    let mut app = App::new();
    app.change_sets = None; // No list yet

    app.change_set_next();
    assert!(app.change_set_list_state.selected().is_none()); // Stays None
    app.change_set_previous();
    assert!(app.change_set_list_state.selected().is_none()); // Stays None
}
