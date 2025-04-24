// src/ui/render_changeset_dropdown.rs

// Intention: Render the Change Set dropdown list overlay if active.
// Design Choice: Encapsulates the logic for calculating dropdown position, creating list items,
// and rendering the stateful List widget. Requires the Change Set trigger area for positioning. Extracted from ui.rs.

use ratatui::{
    Frame,
    layout::Rect,
    prelude::*, // Import common traits and types
    style::{
        Color,
        Modifier,
        Style,
    },
    widgets::{
        Block,
        Borders,
        Clear,
        HighlightSpacing,
        List,
        ListItem,
    },
};

use crate::app::App; // Use App from local app module

// --- Constants for UI Layout (Copied from original ui.rs) ---
const DROPDOWN_LIST_WIDTH: u16 = 50;
const DROPDOWN_MAX_ITEMS: usize = 10;

// Intention: Render the Change Set dropdown list overlay if active.
// Design Choice: Encapsulates the logic for calculating dropdown position, creating list items,
// and rendering the stateful List widget. Requires the Change Set trigger area for positioning.
pub(super) fn render_changeset_dropdown(
    f: &mut Frame,
    app: &App,
    cs_trigger_area: Rect,
) {
    if app.changeset_dropdown_active {
        // Use constants for dropdown dimensions
        let list_height = app
            .change_sets
            .as_ref()
            .map_or(1, |cs| cs.len())
            .min(DROPDOWN_MAX_ITEMS) as u16 // Use constant for max items
            + 2; // +2 for borders
        let list_width = DROPDOWN_LIST_WIDTH; // Use constant for width

        // Calculate position below the trigger
        let list_area = Rect {
            x: cs_trigger_area.x,
            y: cs_trigger_area.y + 1,
            width: list_width.min(f.size().width - cs_trigger_area.x), // Clamp width
            height: list_height.min(f.size().height - (cs_trigger_area.y + 1)), // Clamp height
        };

        // Items for the dropdown list
        let change_set_items: Vec<ListItem> = match &app.change_sets {
            Some(change_sets) => {
                if change_sets.is_empty() {
                    vec![ListItem::new("No change sets found.")]
                } else {
                    change_sets
                        .iter()
                        .map(|cs| {
                            let status_style = match cs.status.as_str() {
                                "Completed" => {
                                    Style::default().fg(Color::Green)
                                }
                                "Failed" => Style::default().fg(Color::Red),
                                "InProgress" => {
                                    Style::default().fg(Color::Yellow)
                                }
                                "Abandoned" => Style::default().fg(Color::Gray),
                                _ => Style::default(),
                            };
                            ListItem::new(format!(
                                "{} ({}) - {}",
                                cs.name, cs.status, cs.id
                            ))
                            .style(status_style)
                        })
                        .collect()
                }
            }
            None => vec![ListItem::new("Loading...")],
        };

        let dropdown_list = List::new(change_set_items)
            .block(
                Block::default()
                    .title("Select Change Set (Enter/Esc)")
                    .borders(Borders::ALL),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);

        // Render the dropdown list
        f.render_widget(Clear, list_area); // Clear the area first
        let mut list_state = app.change_set_list_state.clone(); // Clone state for rendering
        f.render_stateful_widget(dropdown_list, list_area, &mut list_state);
    }
}
