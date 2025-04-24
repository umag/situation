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

use crate::app::App; // Use App from local app module

// Intention: Render the main content area. Displays change set details and merge status if available,
// otherwise shows keybindings.
// Design Choice: Conditionally renders different content based on App state.
pub(super) fn render_content_area(f: &mut Frame, app: &App, area: Rect) {
    let details_block = Block::default().title("Details").borders(Borders::ALL);
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
                "Normal Mode (Dropdown Closed):",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from("  q          : Quit"),
            Line::from(
                "  Tab        : Switch Focus (Workspace <-> Change Set)",
            ),
            Line::from(
                "  Enter/Space: Activate Focused Trigger (Open Dropdown / Fetch Details)",
            ),
            Line::from("  c          : Create Change Set (Enter Input Mode)"),
            Line::from("  d          : Delete Selected Change Set"),
            Line::from("  f          : Force Apply Selected Change Set"),
            Line::from("  k          : Scroll Logs Up"),
            Line::from("  j          : Scroll Logs Down"),
            Line::from(""),
            Line::from(Span::styled(
                "Normal Mode (Change Set Dropdown Active):",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from("  Up Arrow   : Select Previous Item"),
            Line::from("  Down Arrow : Select Next Item"),
            Line::from("  Enter      : Confirm Selection & Close Dropdown"),
            Line::from("  Esc / Tab  : Close Dropdown"),
            Line::from(""),
            Line::from(Span::styled(
                "ChangeSetName Input Mode:",
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
