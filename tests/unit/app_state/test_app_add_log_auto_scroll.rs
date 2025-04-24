// tests/unit/app_state/test_app_add_log_auto_scroll.rs

// Intention: Test the automatic log scrolling when adding new logs.

use situation::App; // Assuming App is accessible

// Test automatic log scrolling when adding logs
#[test]
fn test_app_add_log_auto_scroll() {
    let mut app = App::new();
    let view_height = 3; // Simulate a view height of 3 lines

    // Add initial logs (less than view height)
    app.add_log_auto_scroll("Log 1".to_string(), view_height);
    assert_eq!(app.logs.len(), 1);
    assert_eq!(app.log_scroll, 0); // Should stay at 0

    app.add_log_auto_scroll("Log 2".to_string(), view_height);
    assert_eq!(app.logs.len(), 2);
    assert_eq!(app.log_scroll, 0); // Should stay at 0

    // Add log that fills the view
    app.add_log_auto_scroll("Log 3".to_string(), view_height);
    assert_eq!(app.logs.len(), 3);
    assert_eq!(app.log_scroll, 0); // Max scroll is len(3) - height(3) = 0

    // Add log that exceeds the view height
    app.add_log_auto_scroll("Log 4".to_string(), view_height);
    assert_eq!(app.logs.len(), 4);
    // Max scroll should be len(4) - height(3) = 1
    assert_eq!(app.log_scroll, 1);

    // Add another log
    app.add_log_auto_scroll("Log 5".to_string(), view_height);
    assert_eq!(app.logs.len(), 5);
    // Max scroll should be len(5) - height(3) = 2
    assert_eq!(app.log_scroll, 2);

    // Manually scroll up
    app.scroll_logs_up();
    assert_eq!(app.log_scroll, 1);

    // Add another log - should force scroll back to bottom
    app.add_log_auto_scroll("Log 6".to_string(), view_height);
    assert_eq!(app.logs.len(), 6);
    // Max scroll should be len(6) - height(3) = 3
    assert_eq!(app.log_scroll, 3);
}
