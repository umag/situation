// src/ui/render_content_area.rs

// Intention: Render the main content area. Displays change set details and merge status if available,
// otherwise shows keybindings.
// Design Choice: Conditionally renders different content based on App state. Extracted from ui.rs.

use ratatui::{
    Frame,
    layout::Rect,
    prelude::*, // Import common traits and types
    style::{
        Modifier,
        Style,
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

// Intention: Render the main content area. Displays change set details and merge status if available,
// otherwise shows keybindings. Highlights border on focus.
// Design Choice: Conditionally renders different content based on App state.
pub(super) fn render_content_area(f: &mut Frame, app: &App, area: Rect) {
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

    let content_paragraph = if let Some(details) =
        &app.selected_change_set_details
    {
        // --- Render Change Set Details & Merge Status ---
        let mut lines: Vec<Line> = vec![
            Line::from(vec![
                Span::styled(
                    "Change Set:",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" {} ({})", details.name, details.id)),
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

        if let Some(merge_status) = &app.selected_change_set_merge_status {
            lines.push(Line::from(Span::styled(
                "Merge Status:",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            if merge_status.actions.is_empty() {
                lines.push(Line::from("  No actions required."));
            } else {
                for action in &merge_status.actions {
                    let component_info = action.component.as_ref().map_or_else(
                        || "".to_string(),
                        |comp| format!(" - {} ({})", comp.name, comp.id),
                    );
                    lines.push(Line::from(format!(
                        "  [{}] {} {} {}",
                        action.kind, action.state, action.name, component_info
                    )));
                }
            }
        } else {
            lines.push(Line::from("  Merge status loading or unavailable."));
        }

        // --- Render Components ---
        // Intention: Display the names of components associated with the selected change set.
        // Design Choice: Add a header and list component names if available in the app state.
        lines.push(Line::from("")); // Spacer
        lines.push(Line::from(Span::styled(
            "Components:",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        if let Some(components) = &app.selected_change_set_components {
            if components.is_empty() {
                lines.push(Line::from("  No components in this change set."));
            } else {
                // TODO: Resolve component names against schema list? The prompt mentions this,
                // but ComponentViewV1 already seems to have a `name`. For now, just display the name.
                // Also, the prompt asked for rectangles, but rendering simple lines first.
                for component in components {
                    // Simple display of component name
                    lines.push(Line::from(format!("  - {}", component.name)));
                    // TODO: Future enhancement could render each component in its own Block (rectangle)
                    // let component_block = Block::default().title(component.name.clone()).borders(Borders::ALL);
                    // Need to manage layout within the inner_details_area for multiple blocks.
                }
            }
        } else {
            lines.push(Line::from("  Components loading or unavailable."));
        }

        Paragraph::new(lines).wrap(Wrap { trim: true })
    } else {
        // --- Render Keybindings ---
        let keybindings = vec![
            Line::from(Span::styled(
                "--- Keybindings ---",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Global:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from("  q          : Quit"),
            Line::from(
                "  Tab        : Cycle Focus (Top Bar -> Schemas -> Details -> Logs)",
            ),
            Line::from("  Alt+W      : Focus Workspace Trigger"), // Reverted Cmd to Alt
            Line::from("  Alt+C      : Focus Change Set Trigger"), // Reverted Cmd to Alt
            Line::from("  Alt+S      : Focus Schema List"), // Reverted Cmd to Alt
            Line::from("  Alt+L      : Focus Log Panel"), // Reverted Cmd to Alt
            Line::from(""),
            Line::from(Span::styled(
                "Top Bar:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from(
                "  Enter/Space: Activate Focused Trigger (Open Dropdown / Fetch Details)",
            ),
            Line::from("  c          : Create Change Set (Enter Input Mode)"),
            Line::from("  d          : Delete Selected Change Set"),
            Line::from("  f          : Force Apply Selected Change Set"),
            Line::from("  k          : Scroll Logs Up (Any Focus)"),
            Line::from("  j          : Scroll Logs Down (Any Focus)"),
            Line::from(""),
            Line::from(Span::styled(
                "Top Bar (Change Set Dropdown Active):",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from("  Up Arrow   : Select Previous Item"),
            Line::from("  Down Arrow : Select Next Item"),
            Line::from("  Enter      : Confirm Selection & Close Dropdown"),
            Line::from("  Esc / Tab  : Close Dropdown"),
            // Add Schema List and Log Panel specific keys if any
            Line::from(""),
            Line::from(Span::styled(
                "Schema List:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from("  Up Arrow   : Select Previous Schema"),
            Line::from("  Down Arrow : Select Next Schema"),
            Line::from(""),
            Line::from(Span::styled(
                "Log Panel:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from("  Up/k       : Scroll Logs Up"),
            Line::from("  Down/j     : Scroll Logs Down"),
            Line::from(""),
            Line::from(Span::styled(
                "Input Mode (Create Change Set):",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from("  Enter      : Submit Name & Create"),
            Line::from("  Esc        : Cancel Input"),
            Line::from("  Backspace  : Delete Character"),
            Line::from("  (any char) : Append Character"),
        ];

        // Return the paragraph for the else block
        Paragraph::new(keybindings).wrap(Wrap { trim: true })
    };

    f.render_widget(content_paragraph, inner_details_area);
}
