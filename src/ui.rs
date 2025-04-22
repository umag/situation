// Intention: Define the function responsible for rendering the TUI layout and widgets.
// Design Choice: Moved from main.rs to its own module. Takes a Frame and App reference.
// Constructs the layout and renders widgets based on the App state.

use crate::app::{App, DropdownFocus, InputMode}; // Use App, Enums from local app module
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,                      // Import common traits and types
    style::{Color, Modifier, Style}, // Added Style, Color, Modifier for highlighting
    widgets::{
        Block, Borders, Clear, HighlightSpacing, List, ListItem, Paragraph,
        Wrap,
    }, // Added Clear, HighlightSpacing
};

// Intention: Main UI rendering function. Sets up the layout and calls helper functions for each section.
// Design Choice: Split rendering logic into focused helper functions for clarity and maintainability.
pub fn ui(f: &mut Frame, app: &App) {
    // Define main layout: Top Bar, Main Content, Logs, optional Input Line.
    let log_height = 10; // Keep log height definition here for layout calculation
    let (log_constraint, input_constraint) =
        if app.input_mode == InputMode::ChangeSetName {
            (Constraint::Length(log_height), Constraint::Length(1))
        } else {
            (Constraint::Length(log_height), Constraint::Length(0))
        };

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top bar for dropdown triggers & email
            Constraint::Min(0),    // Main content area (for details)
            log_constraint,        // Log window area
            input_constraint,      // Input line area (conditional)
        ])
        .split(f.size());

    let top_bar_area = main_chunks[0];
    let content_area = main_chunks[1]; // Area for details view
    let log_area = main_chunks[2];
    let input_area = if main_chunks.len() > 3 {
        Some(main_chunks[3])
    } else {
        None // Input area is optional
    };

    // Call helper functions to render each part of the UI
    let cs_trigger_area = render_top_bar(f, app, top_bar_area);
    render_content_area(f, app, content_area);
    render_log_panel(f, app, log_area);
    if let Some(input_area_rect) = input_area {
        render_input_line(f, app, input_area_rect);
    }
    render_changeset_dropdown(f, app, cs_trigger_area); // Needs cs_trigger_area for positioning
}

// --- Helper Functions ---

// Intention: Render the top bar containing Workspace trigger, Change Set trigger, and Email.
// Design Choice: Encapsulates the horizontal layout and widget rendering for the top bar.
// Returns the Rect of the Change Set trigger area for dropdown positioning.
fn render_top_bar(f: &mut Frame, app: &App, area: Rect) -> Rect {
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
    let ws_style = if app.dropdown_focus == DropdownFocus::Workspace
        && app.input_mode == InputMode::Normal
    {
        Style::default().bg(Color::Blue).fg(Color::White)
    } else {
        Style::default()
    };
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
    let cs_style = if app.dropdown_focus == DropdownFocus::ChangeSet
        && app.input_mode == InputMode::Normal
    {
        Style::default().bg(Color::Blue).fg(Color::White)
    } else {
        Style::default()
    };
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

// Intention: Render the main content area, currently displaying static keybindings.
// Design Choice: Encapsulates the "Details" block and the keybindings paragraph.
fn render_content_area(f: &mut Frame, _app: &App, area: Rect) {
    // _app is unused for now, but kept for potential future use (e.g., showing details)
    let details_block = Block::default().title("Details").borders(Borders::ALL);
    let inner_details_area = details_block.inner(area);
    f.render_widget(details_block, area);

    // Keybindings (Static for now)
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
        Line::from("  Tab        : Switch Focus (Workspace <-> Change Set)"),
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

    let keybindings_paragraph =
        Paragraph::new(keybindings).wrap(Wrap { trim: true });
    f.render_widget(keybindings_paragraph, inner_details_area);
}

// Intention: Render the log panel at the bottom.
// Design Choice: Encapsulates the log block (with dynamic title) and the scrollable log paragraph.
fn render_log_panel(f: &mut Frame, app: &App, area: Rect) {
    let log_title = if let Some(action) = &app.current_action {
        format!("Logs (j/k: Scroll) - [{}]", action)
    } else {
        "Logs (j/k: Scroll)".to_string()
    };
    let log_block = Block::default().title(log_title).borders(Borders::ALL);
    let inner_log_area = log_block.inner(area); // Calculate inner area once

    f.render_widget(log_block, area); // Render the block (border + title)

    let log_lines: Vec<Line> = app
        .logs
        .iter()
        .map(|log| Line::from(log.as_str()))
        .collect();
    let log_paragraph = Paragraph::new(log_lines)
        .wrap(Wrap { trim: false })
        .scroll((app.log_scroll as u16, 0));

    f.render_widget(log_paragraph, inner_log_area); // Render the paragraph inside
}

// Intention: Render the input line when in ChangeSetName mode.
// Design Choice: Encapsulates the conditional rendering of the input prompt and buffer.
fn render_input_line(f: &mut Frame, app: &App, area: Rect) {
    if app.input_mode == InputMode::ChangeSetName {
        let input_prompt_text =
            "Enter Change Set Name (Esc: Cancel, Enter: Create):";
        let input_paragraph = Paragraph::new(format!(
            "{} {}{}",
            input_prompt_text,
            app.input_buffer,
            "_" // Simple cursor indicator
        ))
        .style(Style::default().fg(Color::Yellow));
        f.render_widget(input_paragraph, area);
    }
}

// Intention: Render the Change Set dropdown list overlay if active.
// Design Choice: Encapsulates the logic for calculating dropdown position, creating list items,
// and rendering the stateful List widget. Requires the Change Set trigger area for positioning.
fn render_changeset_dropdown(f: &mut Frame, app: &App, cs_trigger_area: Rect) {
    if app.changeset_dropdown_active {
        let list_height =
            app.change_sets.as_ref().map_or(1, |cs| cs.len()).min(10) as u16
                + 2;
        let list_width = 50; // Fixed width

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

// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*; // Import items from parent module (ui)
    use crate::app::App; // Import App from crate root
    use ratatui::Terminal;
    use ratatui::backend::TestBackend; // Required for terminal.draw()

    #[test]
    fn test_ui_renders_without_panic() {
        // Intention: Verify that the main ui function can execute with default state without panicking.
        // Design Choice: Use TestBackend and terminal.draw(). Panic will be caught by test runner.
        let backend = TestBackend::new(80, 24); // Arbitrary size
        let mut terminal = Terminal::new(backend).unwrap();
        let app = App::new(); // Use the App::new() constructor

        // The actual test is whether this call panics or not.
        // If it panics, the test framework will catch it and fail the test.
        terminal
            .draw(|f| {
                ui(f, &app); // Call the ui function from the parent module
            })
            .expect("UI rendering failed"); // Use expect for a clearer error if draw itself fails
    }
}
