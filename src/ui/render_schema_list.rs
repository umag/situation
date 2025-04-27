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
use situation::api_models::SchemaSummary;

use crate::app::{
    App,
    AppFocus,
}; // Import App and AppFocus // Correct import path

pub fn render_schema_list(f: &mut Frame, app: &mut App, area: Rect) {
    // Intention: Create ListItems grouped by category with conditional styling.
    // Design Choice: Iterate sorted schemas, add category headers, indent items, style based on 'installed'.
    let mut list_items = Vec::new();
    let mut current_category: Option<String> = None; // Explicit type annotation

    for schema in &app.schemas {
        // Explicitly check if the category has changed using pattern matching
        let category_changed = match current_category {
            Some(ref current_cat_string) => {
                current_cat_string != &schema.category
            } // Compare String with String
            None => true, // Always true for the first category
        };

        if category_changed {
            // Add category header (bold)
            list_items.push(ListItem::new(Line::from(Span::styled(
                schema.category.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            ))));
            current_category = Some(schema.category.clone()); // Update current category
        }

        // Determine style based on installed status (now a boolean)
        let item_style = if schema.installed {
            Style::default().fg(Color::Blue) // Blue if installed
        } else {
            Style::default() // Default color otherwise
        };

        // Add schema name (indented) with conditional style
        list_items.push(ListItem::new(Line::from(Span::styled(
            format!("  {}", schema.schema_name), // Indent schema name
            item_style,
        ))));
    }

    // Intention: Determine border style based on focus.
    // Design Choice: Use different colors to indicate focus.
    let border_style = if app.current_focus == AppFocus::SchemaList {
        Style::default().fg(Color::Cyan) // Highlight color when focused
    } else {
        Style::default().fg(Color::DarkGray) // Default color when not focused
    };

    // Intention: Create the List widget with items, border, title, and highlight style.
    // Design Choice: Use standard List widget configuration. Apply conditional border style.
    // Construct the title with highlighted 'S'
    let title_spans = vec![
        Span::styled("S", Style::default().fg(Color::Yellow)), // Highlighted 'S'
        Span::raw("chemas"), // Rest of the title
    ];
    let title_line = Line::from(title_spans).alignment(Alignment::Left); // Align title left

    // Use the generated list_items (headers + schemas)
    let schemas_list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title_line) // Use the constructed Line as title
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
