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
    }, // Added Clear, HighlightSpacing, Removed unused ListState
};

// Intention: Define the UI layout and render widgets based on application state.
// Design Choice: Top bar with dropdown triggers, main area for details, logs at the bottom.
// Dropdown list rendered conditionally as an overlay.
pub fn ui(f: &mut Frame, app: &App) {
    // Intention: Define main layout: Top Bar, Main Content, Logs, optional Input Line.
    let log_height = 10;
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
        None
    };

    // --- Top Bar Rendering ---
    // Intention: Display Workspace trigger, Change Set trigger, and Email.
    // Design Choice: Horizontal layout. Highlight focused trigger.
    let top_bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Workspace trigger
            Constraint::Percentage(40), // Change Set trigger
            Constraint::Percentage(30), // Email
        ])
        .split(top_bar_area);

    let ws_trigger_area = top_bar_chunks[0];
    let cs_trigger_area = top_bar_chunks[1];
    let email_area = top_bar_chunks[2];

    // Workspace Trigger (Static for now)
    let ws_name = app
        .whoami_data
        .as_ref()
        .map_or("Loading...", |d| &d.workspace_id);
    let ws_style = if app.dropdown_focus == DropdownFocus::Workspace
        && app.input_mode == InputMode::Normal
    {
        Style::default().bg(Color::Blue).fg(Color::White) // Focused style
    } else {
        Style::default() // Normal style for the whole paragraph background/focus
    };
    // Intention: Display workspace name with color.
    // Design Choice: Use Spans within a Line to apply color only to the name.
    let ws_line = Line::from(vec![
        Span::raw(" Workspace: "),
        Span::styled(ws_name, Style::default().fg(Color::Cyan)), // Apply Cyan color here
        Span::raw(" "),
    ]);
    let ws_trigger = Paragraph::new(ws_line)
        .style(ws_style) // Apply focus style to the whole paragraph
        .block(Block::default());
    f.render_widget(ws_trigger, ws_trigger_area);

    // Change Set Trigger
    let (selected_cs_name, selected_cs_status) =
        app.get_selected_changeset_summary().map_or(
            ("Select Change Set".to_string(), "".to_string()), // Default text
            |cs| (cs.name.clone(), format!(" ({})", cs.status)), // Extract name and status
        );
    let cs_indicator = if app.changeset_dropdown_active {
        "▼"
    } else {
        "▶"
    };
    let cs_style = if app.dropdown_focus == DropdownFocus::ChangeSet
        && app.input_mode == InputMode::Normal
    {
        Style::default().bg(Color::Blue).fg(Color::White) // Focused style
    } else {
        Style::default() // Normal style for the whole paragraph background/focus
    };
    // Intention: Display change set name with color.
    // Design Choice: Use Spans within a Line to apply color only to the name.
    let cs_line = Line::from(vec![
        Span::raw(" Change Set: "),
        Span::styled(selected_cs_name, Style::default().fg(Color::Yellow)), // Apply Yellow color here
        Span::raw(selected_cs_status), // Status without color
        Span::raw(" "),
        Span::raw(cs_indicator),
        Span::raw(" "),
    ]);
    let cs_trigger = Paragraph::new(cs_line)
        .style(cs_style) // Apply focus style to the whole paragraph
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

    // --- Main Content Area (Details) ---
    // Intention: Display details of the selected change set.
    // Design Choice: Render details if available, otherwise show placeholder. Use a simpler title.
    let details_block = Block::default()
        .title("Details") // Simplified title
        .borders(Borders::ALL);
    let inner_details_area = details_block.inner(content_area);
    f.render_widget(details_block, content_area);

    // Intention: Display keybindings in the main content area.
    // Design Choice: Use a static list of Lines in a Paragraph, replacing the previous details view.
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

    // --- Log Window Rendering (Bottom) ---
    let log_block = Block::default()
        .title("Logs (j/k: Scroll)")
        .borders(Borders::ALL);
    let inner_log_area = log_block.inner(log_area);
    f.render_widget(log_block, log_area);

    let log_lines: Vec<ratatui::text::Line> = app
        .logs
        .iter()
        .map(|log| ratatui::text::Line::from(log.as_str()))
        .collect();
    let log_paragraph = Paragraph::new(log_lines)
        .wrap(Wrap { trim: false })
        .scroll((app.log_scroll as u16, 0));
    // Store the paragraph before rendering it the first time
    let log_paragraph_clone = log_paragraph.clone();
    f.render_widget(log_paragraph, inner_log_area);

    // --- Input Line Rendering (Conditional, Bottom) ---
    if let Some(input_area) = input_area {
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
            f.render_widget(input_paragraph, input_area);
        }
    }

    // --- Change Set Dropdown List (Overlay) ---
    // Intention: Render the dropdown list if active.
    // Design Choice: Render last, potentially overlapping content. Use Clear widget first.
    if app.changeset_dropdown_active {
        let list_height =
            app.change_sets.as_ref().map_or(1, |cs| cs.len()).min(10) as u16
                + 2; // Max 10 items + borders
        let list_width = 50; // Fixed width for dropdown

        // Calculate position below the trigger
        let list_area = Rect {
            x: cs_trigger_area.x,
            y: cs_trigger_area.y + 1, // Position below the trigger
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
                            ListItem::new(format!(
                                "{} ({}) - {}",
                                cs.name, cs.status, cs.id
                            ))
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
            .highlight_spacing(HighlightSpacing::Always); // Ensure highlight shows even if list loses focus conceptually

        // Render the dropdown list
        // Use Clear widget first to ensure it draws cleanly over existing content
        f.render_widget(Clear, list_area);
        let mut list_state = app.change_set_list_state.clone(); // Clone state for rendering
        f.render_stateful_widget(dropdown_list, list_area, &mut list_state);
    }

    // Display current action in the log area title bar maybe? Or keep it simple.
    // Let's add it to the log title
    let log_title = if let Some(action) = &app.current_action {
        format!("Logs (j/k: Scroll) - [{}]", action)
    } else {
        "Logs (j/k: Scroll)".to_string()
    };
    let log_block_with_title =
        Block::default().title(log_title).borders(Borders::ALL);
    f.render_widget(log_block_with_title, log_area); // Re-render block with potentially updated title
    // Re-render paragraph inside
    f.render_widget(log_paragraph_clone, inner_log_area); // Render the stored paragraph again
}
