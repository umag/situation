use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use ratatui::{
    Frame,
    Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style}, // Added Style, Color, Modifier for highlighting
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap}, // Added List, ListItem, ListState
};
use tokio;

mod api_client;
mod api_models;

// Import necessary models
use api_models::{ChangeSetSummary, WhoamiResponse}; // Added ChangeSetSummary

use std::{cmp::min, error::Error, io, time::Duration};

// Intention: Hold the application's state.
// Design Choice: Simple struct holding API data, logs, and scroll state.
// Intention: Hold the application's state, including TUI interaction state.
// Design Choice: Added ListState for managing the change set list selection.
#[derive(Debug, Clone)]
struct App {
    whoami_data: Option<WhoamiResponse>,
    change_sets: Option<Vec<ChangeSetSummary>>,
    change_set_list_state: ListState, // Added state for the change set list
    logs: Vec<String>,
    log_scroll: usize,
}

impl App {
    // Intention: Create a new App instance with default values.
    fn new() -> Self {
        Self {
            whoami_data: None,
            change_sets: None,
            change_set_list_state: ListState::default(), // Initialize list state
            logs: Vec::new(),
            log_scroll: 0,
        }
    }

    // Intention: Add a log message to the internal log buffer.
    fn add_log(&mut self, message: String) {
        self.logs.push(message);
        // Optional: Trim logs if they get too long
        // const MAX_LOGS: usize = 1000;
        // if self.logs.len() > MAX_LOGS {
        //     self.logs.remove(0);
        // }
    }

    // Intention: Scroll the log view up by one line.
    fn scroll_logs_up(&mut self) {
        self.log_scroll = self.log_scroll.saturating_sub(1);
    }

    // Intention: Scroll the log view down by one line.
    // Design Choice: Prevent scrolling beyond the available log lines.
    fn scroll_logs_down(&mut self, view_height: usize) {
        // Calculate max scroll based on number of logs and window height
        let max_scroll = self.logs.len().saturating_sub(view_height);
        self.log_scroll = min(self.log_scroll.saturating_add(1), max_scroll);
    }

    // Intention: Move selection down in the change set list.
    fn change_set_next(&mut self) {
        if let Some(change_sets) = &self.change_sets {
            let i = match self.change_set_list_state.selected() {
                Some(i) => {
                    if i >= change_sets.len() - 1 {
                        0 // Wrap around
                    } else {
                        i + 1
                    }
                }
                None => 0, // Select first if nothing selected
            };
            self.change_set_list_state.select(Some(i));
        }
    }

    // Intention: Move selection up in the change set list.
    fn change_set_previous(&mut self) {
        if let Some(change_sets) = &self.change_sets {
            let i = match self.change_set_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        change_sets.len() - 1 // Wrap around
                    } else {
                        i - 1
                    }
                }
                None => 0, // Select first if nothing selected
            };
            self.change_set_list_state.select(Some(i));
        }
    }
}

// Intention: Entry point for the TUI application.
// Design Choice: Using tokio::main for potential async operations later (like API calls).
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Intention: Set up the terminal for TUI rendering.
    // Design Choice: Enable raw mode and enter alternate screen for a clean TUI experience.
    // Ensure terminal is restored even on panic.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Intention: Run the main application loop.
    // Design Choice: Pass the terminal instance to the run_app function.
    let res = run_app(&mut terminal).await;

    // Intention: Restore the terminal to its original state after the application exits.
    // Design Choice: Disable raw mode, leave alternate screen, and disable mouse capture.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err)
    }

    Ok(())
}

// Intention: Main application loop for handling events and rendering the UI.
// Design Choice: A loop that initializes state, fetches data, draws UI, and handles input.
async fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    // Intention: Initialize application state using the new constructor.
    let mut app = App::new();

    // Intention: Perform initial data fetch (whoami and change sets) and log the process.
    // Design Choice: Call whoami first, then list_change_sets if whoami succeeds.
    app.add_log("Fetching initial /whoami data...".to_string());
    match api_client::whoami().await {
        Ok((whoami_data, whoami_logs)) => {
            let workspace_id = whoami_data.workspace_id.clone(); // Clone workspace_id for the next call
            app.whoami_data = Some(whoami_data);
            app.logs.extend(whoami_logs); // Add logs from the whoami call
            app.add_log("/whoami call successful.".to_string());

            // Now fetch change sets using the workspace_id
            app.add_log(format!(
                "Fetching change sets for workspace {}...",
                workspace_id
            ));
            match api_client::list_change_sets(&workspace_id).await {
                Ok((list_response, cs_logs)) => {
                    // Select the first item if the list is not empty
                    if !list_response.change_sets.is_empty() {
                        app.change_set_list_state.select(Some(0));
                    } else {
                        app.change_set_list_state.select(None); // Ensure nothing selected if empty
                    }
                    app.change_sets = Some(list_response.change_sets);
                    app.logs.extend(cs_logs);
                    app.add_log(
                        "list_change_sets call successful.".to_string(),
                    );
                }
                Err(e) => {
                    app.change_set_list_state.select(None); // Ensure nothing selected on error
                    let error_msg =
                        format!("Error fetching change sets: {}", e);
                    app.add_log(error_msg);
                }
            }
        }
        Err(e) => {
            // Log the error message for whoami failure into the app's log buffer.
            let error_msg = format!("Error fetching initial data: {}", e);
            app.add_log(error_msg);
            // Optionally, still print to stderr during development if helpful
            // eprintln!("Error fetching initial data: {}", e);
        }
    }

    loop {
        // Intention: Draw the current state of the UI using app state.
        terminal.draw(|f| ui(f, &app))?; // Pass app state to ui

        // Intention: Handle user input events.
        // Design Choice: Poll for events with a timeout.
        if event::poll(Duration::from_millis(250))? {
            // Use Duration directly
            if let Event::Key(key) = event::read()? {
                // Intention: Exit the application when 'q' is pressed.
                if key.code == KeyCode::Char('q') {
                    return Ok(());
                }
                // Intention: Handle input for list navigation and log scrolling.
                // Design Choice: Use Up/Down for list nav, j/k for log scroll.
                // TODO: Add focus switching mechanism later if needed. Assume list is focus for now.
                let log_height = 10; // Must match the Constraint::Length in ui()
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => app.change_set_previous(), // Navigate list up
                    KeyCode::Down => app.change_set_next(), // Navigate list down
                    KeyCode::Char('k') => app.scroll_logs_up(), // Scroll logs up
                    KeyCode::Char('j') => app.scroll_logs_down(log_height), // Scroll logs down
                    _ => {} // Ignore other keys
                }
            }
        }
        // TODO: Add key for selecting a change set (e.g., Enter)
        // Placeholder for periodic data refresh or other async tasks
    }
}

// Intention: Define the UI layout and render widgets based on application state.
// Design Choice: A layout displaying whoami data, logs, and handling scrolling.

fn ui(f: &mut Frame, app: &App) {
    // Intention: Create the main layout with a top bar, main content, and a bottom log area.
    // Design Choice: Vertical layout splitting screen: top bar (1), main content (flexible), logs (fixed height).
    let log_height = 10; // Define height for the log window
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),          // Top bar for email
            Constraint::Min(0),             // Main content area (flexible)
            Constraint::Length(log_height), // Log window area
        ])
        .split(f.size());

    let top_bar_area = main_chunks[0];
    let content_area = main_chunks[1];
    let log_area = main_chunks[2];

    // Intention: Display user email in the top right corner.
    // Design Choice: Create a paragraph aligned to the right within the top_bar_area.
    let email_text = match &app.whoami_data {
        Some(data) => data.user_email.clone(),
        None => "".to_string(), // Show nothing if data isn't loaded yet
    };
    let email_paragraph =
        Paragraph::new(email_text).alignment(Alignment::Right);
    f.render_widget(email_paragraph, top_bar_area);

    // Intention: Create a block for the main content area below the top bar.
    // Design Choice: Add borders to visually separate it. Title indicates main content.
    let content_block =
        Block::default().title("Main Content").borders(Borders::ALL);
    let inner_content_area = content_block.inner(content_area);
    f.render_widget(content_block, content_area); // Render the block frame

    // Intention: Divide the main content area for user info and the change set list.
    // Design Choice: Split horizontally. User info on top, list below.
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Fixed height for user info
            Constraint::Min(0),    // Remaining space for the change set list
        ])
        .split(inner_content_area); // Split the area *inside* the content block

    let user_info_area = content_chunks[0];
    let change_set_list_area = content_chunks[1];

    // Intention: Display user info in the top part of the content area.
    let user_info_text = match &app.whoami_data {
        Some(whoami_data) => format!(
            "User Info:\nUser ID: {}\nWorkspace ID: {}",
            whoami_data.user_id, whoami_data.workspace_id
        ),
        None => "Loading user info...".to_string(),
    };
    let user_info_paragraph = Paragraph::new(user_info_text);
    f.render_widget(user_info_paragraph, user_info_area);

    // Intention: Display the change sets in a selectable list.
    // Design Choice: Use `List` widget, map `ChangeSetSummary` to `ListItem`. Highlight selected.
    let change_set_items: Vec<ListItem> = match &app.change_sets {
        Some(change_sets) => change_sets
            .iter()
            .map(|cs| {
                ListItem::new(format!(
                    "{} ({}) - {}",
                    cs.name, cs.status, cs.id
                ))
            })
            .collect(),
        None => vec![ListItem::new("Loading change sets...")], // Show loading state
    };

    let change_set_list = List::new(change_set_items)
        .block(
            Block::default()
                .title("Change Sets (Use Up/Down)")
                .borders(Borders::NONE),
        ) // Title for the list area
        .highlight_style(
            Style::default()
                .bg(Color::LightBlue) // Highlight background
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> "); // Symbol for selected item

    // Render the list, passing the mutable state
    // Clone the state because render_stateful_widget requires mutable access
    let mut list_state = app.change_set_list_state.clone();
    f.render_stateful_widget(
        change_set_list,
        change_set_list_area,
        &mut list_state,
    );

    // --- Log Window Rendering ---

    // Intention: Create a block for the log window.
    // Design Choice: Add borders and a title.
    let log_block = Block::default().title("Logs").borders(Borders::ALL);
    let inner_log_area = log_block.inner(log_area); // Area inside the log block borders
    f.render_widget(log_block, log_area); // Render the block itself

    // Intention: Display the logs stored in the app state within the log block.
    // Design Choice: Create a Paragraph widget with the logs. Enable wrapping and scrolling.
    // Convert Vec<String> to Vec<ratatui::text::Line> for the Paragraph widget. Using fully qualified path due to import issues.
    let log_lines: Vec<ratatui::text::Line> = app
        .logs
        .iter()
        .map(|log| ratatui::text::Line::from(log.as_str())) // Use fully qualified path
        .collect();
    let log_paragraph = Paragraph::new(log_lines)
        .wrap(Wrap { trim: false }) // Keep log lines intact, wrap if necessary
        .scroll((app.log_scroll as u16, 0)); // Apply vertical scroll based on app state

    f.render_widget(log_paragraph, inner_log_area); // Render logs inside the log block
}
