// tests/unit/app_state/test_app_new.rs

// Intention: Test the App::new() constructor.

// Note: App needs to be accessible from the test crate.
// Assuming App is in src/app.rs and made public or accessible.
use situation::App;

// Test App initialization
#[test]
fn test_app_new() {
    let app = App::new();
    assert!(app.whoami_data.is_none());
    assert!(app.change_sets.is_none());
    assert!(app.change_set_list_state.selected().is_none());
    assert!(app.logs.is_empty());
    assert_eq!(app.log_scroll, 0);
}
