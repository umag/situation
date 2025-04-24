// tests/unit/app_state/test_app_add_log.rs

// Intention: Test the App::add_log method.

use situation::App; // Assuming App is accessible

// Test adding logs
#[test]
fn test_app_add_log() {
    let mut app = App::new();
    app.add_log("Test log 1".to_string());
    app.add_log("Test log 2".to_string());
    assert_eq!(app.logs.len(), 2);
    assert_eq!(app.logs[0], "Test log 1");
    assert_eq!(app.logs[1], "Test log 2");
}
