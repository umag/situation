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
    layout::{Alignment, Constraint, Direction, Layout}, // Added Alignment
    text::Line, // Added Line import for log rendering
    widgets::{Block, Borders, Paragraph, Wrap}, // Added Wrap
};
use tokio;

mod api_client; // Added module
mod api_models; // Added module

use api_models::WhoamiResponse; // Use the specific model

use std::{cmp::min, error::Error, io, time::Duration}; // Added Duration, cmp::min

// Intention: Hold the application's state.
// Design Choice: Simple struct holding API data, logs, and scroll state.
#[derive(Debug, Clone)] // Removed Default derive, will implement new()
struct App {
    whoami_data: Option<WhoamiResponse>,
    logs: Vec<String>, // To store log messages
    log_scroll: usize, // To track scroll position in the log window
                       // Add other state fields as needed
}

impl App {
    // Intention: Create a new App instance with default values.
    fn new() -> Self {
        Self {
            whoami_data: None,
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

    // Intention: Perform initial data fetch and log the process.
    // Design Choice: Call whoami once at the start. Handle the new tuple return type and log results/errors.
    app.add_log("Fetching initial /whoami data...".to_string());
    match api_client::whoami().await {
        Ok((data, logs)) => {
            app.whoami_data = Some(data);
            app.logs.extend(logs); // Add logs from the API call
            app.add_log("/whoami call successful.".to_string());
        }
        Err(e) => {
            // Log the error message into the app's log buffer.
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
                // Intention: Handle log scrolling input.
                // Design Choice: Use Up/Down arrows or j/k keys. Pass log window height to scroll_down.
                // Note: log_height is hardcoded here for simplicity, matching the value in ui().
                // A more robust solution might involve passing layout info around.
                let log_height = 10; // Must match the Constraint::Length in ui()
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => app.scroll_logs_up(),
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.scroll_logs_down(log_height)
                    }
                    _ => {} // Ignore other keys
                }
            }
        }
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
    f.render_widget(content_block, content_area);

    // Intention: Display user info (excluding email) or loading/error in the main content area.
    // Design Choice: Use a match statement on `app.whoami_data`. Use Wrap for long lines.
    let main_content_text = match &app.whoami_data {
        Some(data) => {
            format!(
                "User Info:\nUser ID: {}\nWorkspace ID: {}\n\n(Logs below - Use Up/Down or j/k to scroll)",
                data.user_id, data.workspace_id
            )
        }
        None => {
            "Loading user info... (or error occurred)\n\n(Logs below - Use Up/Down or j/k to scroll)"
                .to_string()
        }
    };
    let main_paragraph =
        Paragraph::new(main_content_text).wrap(Wrap { trim: true });
    f.render_widget(main_paragraph, inner_content_area);

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
