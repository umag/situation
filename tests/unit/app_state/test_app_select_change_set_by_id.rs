// tests/unit/app_state/test_app_select_change_set_by_id.rs

// Intention: Test the App::select_change_set_by_id method.

use situation::App; // Assuming App is accessible
use situation::api_models::ChangeSetSummary; // Need ChangeSetSummary

// Test selecting a change set by ID
#[test]
fn test_app_select_change_set_by_id() {
    let mut app = App::new();
    let change_sets = vec![
        ChangeSetSummary {
            id: "cs-1".to_string(),
            name: "One".to_string(),
            status: "Draft".to_string(),
        },
        ChangeSetSummary {
            id: "cs-new".to_string(),
            name: "Newly Created".to_string(),
            status: "Draft".to_string(),
        },
        ChangeSetSummary {
            id: "cs-3".to_string(),
            name: "Three".to_string(),
            status: "Draft".to_string(),
        },
    ];
    app.change_sets = Some(change_sets);

    // Initially nothing selected
    assert!(app.change_set_list_state.selected().is_none());

    // Select existing ID
    app.select_change_set_by_id("cs-new");
    assert_eq!(app.change_set_list_state.selected(), Some(1)); // Should select index 1

    // Select another existing ID
    app.select_change_set_by_id("cs-1");
    assert_eq!(app.change_set_list_state.selected(), Some(0)); // Should select index 0

    // Select non-existent ID
    app.select_change_set_by_id("cs-unknown");
    assert_eq!(app.change_set_list_state.selected(), Some(0)); // Selection should remain unchanged

    // Select with empty list
    app.change_sets = Some(vec![]);
    app.change_set_list_state.select(None); // Reset selection
    app.select_change_set_by_id("cs-1");
    assert!(app.change_set_list_state.selected().is_none()); // Should remain None

    // Select with None list
    app.change_sets = None;
    app.change_set_list_state.select(None); // Reset selection
    app.select_change_set_by_id("cs-1");
    assert!(app.change_set_list_state.selected().is_none()); // Should remain None
}
