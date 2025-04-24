// tests/unit/app_state/helpers.rs

// Intention: Contains helper functions for app state unit tests.

// Note: Need to import ChangeSetSummary from the main crate.
// Assuming it's made public or this test module has access.
use situation::api_models::ChangeSetSummary;

// Helper function to create dummy change sets
pub(super) fn create_dummy_change_sets(count: usize) -> Vec<ChangeSetSummary> {
    (0..count)
        .map(|i| ChangeSetSummary {
            id: format!("id_{}", i),
            name: format!("Change Set {}", i),
            status: "Draft".to_string(),
        })
        .collect()
}
