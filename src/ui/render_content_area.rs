// src/ui/render_content_area.rs

// Intention: Render the main content area. Displays components if loaded, otherwise
// change set details/status, or keybindings if nothing is selected.
// Design Choice: Prioritize showing only components if they are loaded and present.
// Otherwise, show details/status/component status. Fallback to keybindings.

use ratatui::{
    Frame,
    layout::Rect,
    prelude::*, // Import common traits and types
    style::{
        Modifier,
        Style,
        Stylize, // Added for direct styling
    },
    widgets::{
        Block,
        Borders,
        Paragraph,
        Wrap,
    },
};

use crate::app::{
    App,
    AppFocus,
}; // Use App from local app module

// Intention: Render the main content area based on application state.
// Priority:
// 1. If components are loaded and non-empty: Show ONLY components.
// 2. If components are loaded but empty OR components are loading/error: Show details/status/component status.
// 3. If no change set details are selected: Show keybindings.
pub(super) fn render_content_area(f: &mut Frame, app: &App, area: Rect) {
    // Changed app to immutable reference since we don't need to modify it in this function
    // We need a fixed height for the log panel to pass here, assuming 10 like in event_handler.
    const LOG_HEIGHT: usize = 10;

    // Determine border style based on focus
    let border_style = if app.current_focus == AppFocus::ContentArea {
        Style::default().fg(Color::Cyan) // Highlight color when focused
    } else {
        Style::default().fg(Color::DarkGray) // Default color when not focused
    };

    let details_block = Block::default()
        .title("Details")
        .borders(Borders::ALL)
        .border_style(border_style); // Apply conditional border style
    let inner_details_area = details_block.inner(area);
    f.render_widget(details_block, area); // Render the block border/title first

    // Debug: Log the state of components
    let debug_lines = match &app.selected_change_set_components {
        Some(components) => {
            format!("DEBUG: Components loaded: {}", components.len())
        }
        None => "DEBUG: No components loaded".to_string(),
    };
    f.render_widget(
        Paragraph::new(debug_lines).style(Style::default().fg(Color::Red)),
        Rect::new(area.x, area.y, area.width, 1),
    );

    let content_paragraph = match &app.selected_change_set_components {
        // Case 1: Components loaded and non-empty -> Show ONLY components
        Some(components) if !components.is_empty() => {
            let mut lines: Vec<Line> = Vec::new();

            // Debug: Add component IDs and schema IDs
            lines.push(Line::from(Span::styled(
                "DEBUG: Component IDs and Schema IDs:",
                Style::default().fg(Color::Red),
            )));
            for component in components.iter().take(3) {
                // Show first 3 for brevity
                lines.push(Line::from(Span::styled(
                    format!(
                        "  - {} (schema_id: {})",
                        component.name, component.schema_id
                    ),
                    Style::default().fg(Color::Red),
                )));
            }
            if components.len() > 3 {
                lines.push(Line::from(Span::styled(
                    format!("  ... and {} more", components.len() - 3),
                    Style::default().fg(Color::Red),
                )));
            }

            // Debug: Show selected schema info
            if let Some(selected_idx) = app.schema_list_state.selected() {
                if !app.schemas.is_empty() {
                    let selected_schema = &app.schemas[selected_idx];
                    lines.push(Line::from(Span::styled(
                        format!(
                            "DEBUG: Selected schema: {} (id: {})",
                            selected_schema.schema_name,
                            selected_schema.schema_id
                        ),
                        Style::default().fg(Color::Red),
                    )));
                } else {
                    lines.push(Line::from(Span::styled(
                        "DEBUG: No schemas available",
                        Style::default().fg(Color::Red),
                    )));
                }
            } else {
                lines.push(Line::from(Span::styled(
                    "DEBUG: No schema selected",
                    Style::default().fg(Color::Red),
                )));
            }

            // Display all components without filtering
            lines.push(Line::from(Span::styled(
                format!("Components ({})", components.len()),
                Style::default().add_modifier(Modifier::BOLD),
            )));

            // Add each component
            if components.is_empty() {
                lines.push(Line::from("  No components in this change set."));
            } else {
                for component in components.iter() {
                    // Look up the schema name for this component ID
                    // The component ID is the same as the schema ID
                    let schema_name = app
                        .schemas
                        .iter()
                        .find(|schema| schema.schema_id == component.id)
                        .map(|schema| schema.schema_name.clone())
                        .unwrap_or_else(|| "Unknown Schema".to_string());

                    // Display the component with its schema name
                    lines.push(Line::from(format!(
                        "  - {} ({})",
                        component.id, schema_name
                    )));
                    // TODO: Render as rectangles later if needed
                }
            }

            Paragraph::new(lines).wrap(Wrap { trim: true })
        }
        // Case 2, 3, 4: Components empty, loading, error, or no CS selected
        _ => {
            // Check if change set details are available to render details/status/component status
            if let Some(details) = &app.selected_change_set_details {
                let mut lines: Vec<Line> = vec![
                    Line::from(vec![
                        Span::styled(
                            "Change Set:",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!(
                            " {} ({})",
                            details.name, details.id
                        )),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "Status:",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!(" {}", details.status)), // TODO: Add color based on status?
                    ]),
                    Line::from(""), // Spacer
                ];

                // Add Merge Status section
                if let Some(merge_status) =
                    &app.selected_change_set_merge_status
                {
                    lines.push(Line::from(Span::styled(
                        "Merge Status:",
                        Style::default().add_modifier(Modifier::BOLD),
                    )));
                    if merge_status.actions.is_empty() {
                        lines.push(Line::from("  No actions required."));
                    } else {
                        for action in &merge_status.actions {
                            let component_info =
                                action.component.as_ref().map_or_else(
                                    || "".to_string(),
                                    |comp| {
                                        format!(
                                            " - {} ({})",
                                            comp.name, comp.id
                                        )
                                    },
                                );
                            lines.push(Line::from(format!(
                                "  [{}] {} {} {}",
                                action.kind,
                                action.state,
                                action.name,
                                component_info
                            )));
                        }
                    }
                } else {
                    lines.push(Line::from(
                        "  Merge status loading or unavailable.",
                    ));
                }

                // Add Components section status (since we are in the fallback case)
                lines.push(Line::from("")); // Spacer
                lines.push(Line::from(Span::styled(
                    "Components:",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                match &app.selected_change_set_components {
                    Some(components) if components.is_empty() => {
                        lines.push(Line::from(
                            "  No components in this change set.",
                        ));
                    }
                    None => {
                        lines.push(Line::from(
                            "  Components loading or unavailable.",
                        ));
                    }
                    // This case is handled by the outer match, but needed for exhaustiveness
                    Some(_) => {}
                }
                Paragraph::new(lines).wrap(Wrap { trim: true })
            } else {
                // Fallback: No change set details selected -> Render Keybindings
                render_keybindings()
            }
        }
    };

    f.render_widget(content_paragraph, inner_details_area);
}

// Helper function to generate keybindings paragraph (extracted for clarity)
fn render_keybindings<'a>() -> Paragraph<'a> {
    let keybindings = vec![
        Line::from("--- Keybindings ---".bold()),
        Line::from(""),
        Line::from("Global:".underlined()),
        Line::from("  q          : Quit"),
        Line::from(
            "  Tab        : Cycle Focus (Top Bar -> Schemas -> Details -> Logs)",
        ),
        Line::from("  Alt+W      : Focus Workspace Trigger"),
        Line::from("  Alt+C      : Focus Change Set Trigger"),
        Line::from("  Alt+S      : Focus Schema List"),
        Line::from("  Alt+L      : Focus Log Panel"),
        Line::from(""),
        Line::from("Top Bar:".underlined()),
        Line::from(
            "  Enter/Space: Activate Focused Trigger (Open Dropdown / Fetch Details)",
        ),
        Line::from("  c          : Create Change Set (Enter Input Mode)"),
        Line::from("  d          : Delete Selected Change Set"),
        Line::from("  f          : Force Apply Selected Change Set"),
        Line::from("  k          : Scroll Logs Up (Any Focus)"),
        Line::from("  j          : Scroll Logs Down (Any Focus)"),
        Line::from(""),
        Line::from("Top Bar (Change Set Dropdown Active):".underlined()),
        Line::from("  Up Arrow   : Select Previous Item"),
        Line::from("  Down Arrow : Select Next Item"),
        Line::from("  Enter      : Confirm Selection & Close Dropdown"),
        Line::from("  Esc / Tab  : Close Dropdown"),
        Line::from(""),
        Line::from("Schema List:".underlined()),
        Line::from("  Up Arrow   : Select Previous Schema"),
        Line::from("  Down Arrow : Select Next Schema"),
        Line::from(""),
        Line::from("Log Panel:".underlined()),
        Line::from("  Up/k       : Scroll Logs Up"),
        Line::from("  Down/j     : Scroll Logs Down"),
        Line::from(""),
        Line::from("Input Mode (Create Change Set):".underlined()),
        Line::from("  Enter      : Submit Name & Create"),
        Line::from("  Esc        : Cancel Input"),
        Line::from("  Backspace  : Delete Character"),
        Line::from("  (any char) : Append Character"),
    ];
    Paragraph::new(keybindings).wrap(Wrap { trim: true })
}
