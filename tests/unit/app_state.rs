// tests/unit/app_state.rs

// Intention:
// Contains unit tests for the application state management logic
// defined in the App struct in src/main.rs.

// Design Choices:
// - Uses standard Rust test conventions (`#[cfg(test)]`, `#[test]`).
// - Imports necessary items from the main crate (`situation`).
// - Tests focus on individual methods of the App struct in isolation.

// Note: Since this is an integration-style test file (`tests/` directory),
// we need to import items from our crate (`situation`).
// We need to make App and its fields/methods public for this to work from outside the crate.
// Alternatively, these tests could live inside src/main.rs as `#[cfg(test)] mod tests { ... }`
// For now, assuming App and relevant parts are made public in src/main.rs.
use ratatui::widgets::ListState;
use situation::App;
use situation::api_models::ChangeSetSummary; // Need ListState for assertions

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


// Helper function to create dummy change sets
fn create_dummy_change_sets(count: usize) -> Vec<ChangeSetSummary> {
    (0..count)
        .map(|i| ChangeSetSummary {
            id: format!("id_{}", i),
            name: format!("Change Set {}", i),
            status: "Draft".to_string(),
        })
        .collect()
}

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

// Test change set list navigation (previous)
#[test]
fn test_app_change_set_previous() {
    let mut app = App::new();
    app.change_sets = Some(create_dummy_change_sets(3)); // 3 items

    // Start with nothing selected
    assert!(app.change_set_list_state.selected().is_none());
    app.change_set_previous();
    assert_eq!(app.change_set_list_state.selected(), Some(0)); // Selects first

    // Move to previous (wraps around)
    app.change_set_previous();
    assert_eq!(app.change_set_list_state.selected(), Some(2));
    app.change_set_previous();
    assert_eq!(app.change_set_list_state.selected(), Some(1));
    app.change_set_previous();
    assert_eq!(app.change_set_list_state.selected(), Some(0));
}

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
