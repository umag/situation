// tests/unit/app_state/test_app_change_set_next.rs

// Intention: Test the App::change_set_next method for list navigation.

use situation::App; // Assuming App is accessible

// Import helper function from the same directory
use super::helpers::create_dummy_change_sets;

// Test change set list navigation (next)
#[test]
fn test_app_change_set_next() {
    let mut app = App::new();
    app.change_sets = Some(create_dummy_change_sets(3)); // 3 items

    // Start with nothing selected
    assert!(app.change_set_list_state.selected().is_none());
    app.change_set_next();
    assert_eq!(app.change_set_list_state.selected(), Some(0)); // Selects first

    // Move to next
    app.change_set_next();
    assert_eq!(app.change_set_list_state.selected(), Some(1));
    app.change_set_next();
    assert_eq!(app.change_set_list_state.selected(), Some(2));

    // Wrap around
    app.change_set_next();
    assert_eq!(app.change_set_list_state.selected(), Some(0));
}
