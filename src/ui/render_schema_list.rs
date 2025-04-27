// src/ui/render_schema_list.rs

// Intention: Defines the function to render the schema list widget.
// Design Choice: Creates a List widget populated with schema names from the App state.
// Highlights the selected item based on `app.schema_list_state`.
// Applies a border and title, highlighting the border if the list has focus.

use ratatui::{
    prelude::*,
    widgets::{
        Block,
        Borders,
        List,
        ListItem,
        ListState,
    },
};

use crate::app::{
    App,
    AppFocus,
}; // Import App and AppFocus

pub fn render_schema_list(f: &mut Frame, app: &mut App, area: Rect) {
    // Intention: Create a List widget items from the app's schema names.
    // Design Choice: Map schema names to ListItem widgets.
    let schema_items: Vec<ListItem> = app
        .schemas
        .iter()
        .map(|name| ListItem::new(name.as_str()))
        .collect();

    // Intention: Determine border style based on focus.
    // Design Choice: Use different colors to indicate focus.
    let border_style = if app.current_focus == AppFocus::SchemaList {
        Style::default().fg(Color::Cyan) // Highlight color when focused
    } else {
        Style::default().fg(Color::DarkGray) // Default color when not focused
    };

    // Intention: Create the List widget with items, border, title, and highlight style.
    // Design Choice: Use standard List widget configuration. Apply conditional border style.
    let schemas_list = List::new(schema_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Schemas")
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray) // Background for selected item
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> "); // Symbol prefix for selected item

    // Intention: Render the list widget with its state.
    // Design Choice: Pass the mutable list state to the render_stateful_widget function.
    f.render_stateful_widget(schemas_list, area, &mut app.schema_list_state);
}
