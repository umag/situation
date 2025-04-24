// src/ui/render_top_bar.rs

// Intention: Render the top bar containing Workspace trigger, Change Set trigger, and Email.
// Design Choice: Encapsulates the horizontal layout and widget rendering for the top bar. Extracted from ui.rs.
// Returns the Rect of the Change Set trigger area for dropdown positioning.

use ratatui::{
    Frame,
    layout::{
        Alignment,
        Constraint,
        Direction,
        Layout,
        Rect,
    },
    prelude::*, // Import common traits and types
    style::{
        Color,
        Style,
    },
    widgets::{
        Block,
        Paragraph,
    },
};

// Import the helper function from its new module
use super::get_trigger_style::get_trigger_style;
use crate::app::{
    App,
    DropdownFocus,
    InputMode,
}; // Use App, Enums from local app module

// Intention: Render the top bar containing Workspace trigger, Change Set trigger, and Email.
// Design Choice: Encapsulates the horizontal layout and widget rendering for the top bar.
// Returns the Rect of the Change Set trigger area for dropdown positioning.
pub(super) fn render_top_bar(f: &mut Frame, app: &App, area: Rect) -> Rect {
    let top_bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Workspace trigger
            Constraint::Percentage(40), // Change Set trigger
            Constraint::Percentage(30), // Email
        ])
        .split(area);

    let ws_trigger_area = top_bar_chunks[0];
    let cs_trigger_area = top_bar_chunks[1];
    let email_area = top_bar_chunks[2];

    // Workspace Trigger
    let ws_name = app
        .whoami_data
        .as_ref()
        .map_or("Loading...", |d| &d.workspace_id);
    // Use helper function to get style
    let ws_is_focused = app.dropdown_focus == DropdownFocus::Workspace
        && app.input_mode == InputMode::Normal;
    let ws_style = get_trigger_style(ws_is_focused);
    let ws_line = Line::from(vec![
        Span::raw(" Workspace: "),
        Span::styled(ws_name, Style::default().fg(Color::Cyan)),
        Span::raw(" "),
    ]);
    let ws_trigger = Paragraph::new(ws_line)
        .style(ws_style)
        .block(Block::default());
    f.render_widget(ws_trigger, ws_trigger_area);

    // Change Set Trigger
    let (selected_cs_name, selected_cs_status) = app
        .get_selected_changeset_summary()
        .map_or(("Select Change Set".to_string(), "".to_string()), |cs| {
            (cs.name.clone(), format!(" ({})", cs.status))
        });
    let cs_indicator = if app.changeset_dropdown_active {
        "▼"
    } else {
        "▶"
    };
    // Use helper function to get style
    let cs_is_focused = app.dropdown_focus == DropdownFocus::ChangeSet
        && app.input_mode == InputMode::Normal;
    let cs_style = get_trigger_style(cs_is_focused);
    let cs_line = Line::from(vec![
        Span::raw(" Change Set: "),
        Span::styled(selected_cs_name, Style::default().fg(Color::Yellow)),
        Span::raw(selected_cs_status),
        Span::raw(" "),
        Span::raw(cs_indicator),
        Span::raw(" "),
    ]);
    let cs_trigger = Paragraph::new(cs_line)
        .style(cs_style)
        .block(Block::default());
    f.render_widget(cs_trigger, cs_trigger_area);

    // Email
    let email_text = app
        .whoami_data
        .as_ref()
        .map_or("".to_string(), |d| d.user_email.clone());
    let email_paragraph =
        Paragraph::new(email_text).alignment(Alignment::Right);
    f.render_widget(email_paragraph, email_area);

    cs_trigger_area // Return this area for dropdown positioning
}
