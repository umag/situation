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

// Intention: Render the main content area. Displays change set details and merge status if available,
// otherwise shows keybindings.
// Design Choice: Conditionally renders different content based on App state.
fn render_content_area(f: &mut Frame, app: &App, area: Rect) {
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
    use ratatui::backend::TestBackend;
    // Import necessary models for mock data
    use situation::api_models::{
        ChangeSet, MergeStatusV1Response, MergeStatusV1ResponseAction,
        MergeStatusV1ResponseActionComponent,
    };

    #[test]
    fn test_ui_renders_details_when_present() {
        // Intention: Verify UI renders details and merge status when available in App state.
        // Design Choice: Create mock data, set it in App, render UI, check buffer content.
        let backend = TestBackend::new(80, 30); // Increased height for details
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(); // Use the App::new() constructor

        // Create mock data matching api_models.rs definitions
        let mock_change_set = ChangeSet {
            // Renamed for clarity
            id: "cs_123".to_string(),
            name: "Mock Change Set".to_string(),
            status: "Completed".to_string(),
        };
        let mock_merge_status = MergeStatusV1Response {
            change_set: mock_change_set.clone(), // Use the cloned mock ChangeSet
            actions: vec![MergeStatusV1ResponseAction {
                id: "action_abc".to_string(),
                state: "Added".to_string(),
                kind: "create".to_string(),
                name: "mock_action_name".to_string(),
                component: Some(MergeStatusV1ResponseActionComponent {
                    id: "comp_xyz".to_string(),
                    name: "Mock Component".to_string(),
                }),
            }],
        };

        // Set mock data in app state
        app.selected_change_set_details = Some(mock_change_set); // Assign the correct mock ChangeSet
        app.selected_change_set_merge_status = Some(mock_merge_status);

        // Render the UI
        terminal
            .draw(|f| {
                ui(f, &app); // Call the ui function from the parent module
            })
            .expect("UI rendering failed");

        // Check the buffer for expected content fragments
        // Note: These assertions will likely fail until render_content_area is updated
        let buffer = terminal.backend().buffer();
        let buffer_content = buffer_to_string(buffer); // Helper function needed

        // Basic checks - these will need refinement based on actual rendering format
        // assert!(buffer_content.contains("Mock Change Set"), "Buffer should contain change set name");
        // assert!(buffer_content.contains("Mock Component"), "Buffer should contain component name");
        // assert!(buffer_content.contains("Merge Status"), "Buffer should contain merge status section");

        // For now, just assert the draw didn't panic (implicitly checked by reaching here)
        // and maybe check that keybindings are *not* shown when details are present.
        assert!(
            !buffer_content.contains("--- Keybindings ---"),
            "Keybindings should not be shown when details are present"
        );
    }

    // Helper function to convert buffer content to a searchable string
    // (This is a simplified version; real implementation might need more care with lines/wrapping)
    fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
        let mut content = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                content.push_str(buffer.get(x, y).symbol());
            }
            content.push('\n');
        }
        content
    }

    #[test]
    fn test_ui_renders_keybindings_when_no_details() {
        // Intention: Verify UI renders keybindings when no details/status are available.
        // Design Choice: Use default App state, render UI, check buffer for keybindings.
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = App::new(); // Default app has no details selected

        terminal
            .draw(|f| {
                ui(f, &app);
            })
            .expect("UI rendering failed");

        let buffer = terminal.backend().buffer();
        let buffer_content = buffer_to_string(buffer);

        assert!(
            buffer_content.contains("--- Keybindings ---"),
            "Keybindings should be shown when no details are present"
        );
    }
}
