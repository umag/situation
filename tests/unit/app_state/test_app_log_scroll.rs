// tests/unit/app_state/test_app_log_scroll.rs

// Intention: Test the manual log scrolling logic (up/down).

use situation::App; // Assuming App is accessible

// Test log scrolling logic
#[test]
fn test_app_log_scroll() {
    let mut app = App::new();
    app.logs = vec![
        "1".to_string(),
        "2".to_string(),
        "3".to_string(),
        "4".to_string(),
        "5".to_string(),
    ];
    let view_height = 3; // Simulate a view height of 3 lines

    // Initial state
    assert_eq!(app.log_scroll, 0);

    // Scroll down
    app.scroll_logs_down(view_height);
    assert_eq!(app.log_scroll, 1);
    app.scroll_logs_down(view_height);
    assert_eq!(app.log_scroll, 2); // Max scroll is len(5) - height(3) = 2

    // Scroll down further (should not exceed max)
    app.scroll_logs_down(view_height);
    assert_eq!(app.log_scroll, 2);

    // Scroll up
    app.scroll_logs_up();
    assert_eq!(app.log_scroll, 1);
    app.scroll_logs_up();
    assert_eq!(app.log_scroll, 0);

    // Scroll up further (should not go below 0)
    app.scroll_logs_up();
    assert_eq!(app.log_scroll, 0);
}
