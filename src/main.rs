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
    // text::Line, // Removed unused Line
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap}, // Added List, ListItem, ListState
};
use tokio;

// Use the library crate 'situation' to access shared modules
use situation::api_client;
// Import necessary models from the library crate
use situation::api_models::{
    ChangeSet, CreateChangeSetV1Request, MergeStatusV1Response, WhoamiResponse,
}; // Ensure correct import name: MergeStatusV1Response

use std::{cmp::min, error::Error, io, time::Duration};

// Intention: Define different input modes for the application.
// Design Choice: Enum to represent distinct input states, starting with Normal and ChangeSetName input.
#[derive(Debug, Clone, PartialEq, Eq)] // Added PartialEq, Eq for comparison
enum InputMode {
    Normal,
    ChangeSetName,
}

// Intention: Hold the application's state.
// Design Choice: Simple struct holding API data, logs, and scroll state.
// Intention: Hold the application's state, including TUI interaction state.
// Design Choice: Added ListState for managing the change set list selection.
// Intention: Hold the application's state, including TUI interaction state,
// selected item details, merge status, and UI flags.
// Design Choice: Added fields for details, status, action feedback, pane visibility, input mode, and input buffer.
#[derive(Debug, Clone)]
struct App {
    whoami_data: Option<WhoamiResponse>,
    // Use fully qualified path for ChangeSetSummary just in case
    change_sets: Option<Vec<situation::api_models::ChangeSetSummary>>,
    change_set_list_state: ListState, // State for the change set list selection
    selected_change_set_details: Option<ChangeSet>, // Details of the selected change set
    selected_change_set_merge_status: Option<MergeStatusV1Response>, // Corrected type AGAIN: Merge status of the selected change set
    current_action: Option<String>, // Feedback for ongoing actions (e.g., "Deleting...")
    show_details_pane: bool, // Flag to control visibility of the details pane
    input_mode: InputMode,   // Current input mode
    input_buffer: String,    // Buffer for text input
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
            selected_change_set_details: None,
            selected_change_set_merge_status: None, // Initialize correctly
            current_action: None,
            show_details_pane: false, // Details pane hidden by default
            input_mode: InputMode::Normal, // Start in Normal mode
            input_buffer: String::new(), // Initialize empty input buffer
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

// Intention: Helper function to refresh the list of change sets.
// Design Choice: Encapsulates the API call and state update logic.
async fn refresh_change_sets(app: &mut App) {
    if let Some(whoami_data) = &app.whoami_data {
        let workspace_id = whoami_data.workspace_id.clone();
        app.add_log(format!(
            "Refreshing change sets for workspace {}...",
            workspace_id
        ));
        match api_client::list_change_sets(&workspace_id).await {
            Ok((list_response, cs_logs)) => {
                // Preserve selection if possible, otherwise select first or none
                let current_selection = app.change_set_list_state.selected();
                let new_len = list_response.change_sets.len();

                if new_len == 0 {
                    app.change_set_list_state.select(None);
                } else if let Some(selected_idx) = current_selection {
                    // If previous selection index is still valid, keep it
                    if selected_idx >= new_len {
                        app.change_set_list_state.select(Some(new_len - 1)); // Select last if out of bounds
                    } else {
                        // Keep selection - no need to call select
                    }
                } else {
                    // No previous selection or list was empty, select first
                    app.change_set_list_state.select(Some(0));
                }

                app.change_sets = Some(list_response.change_sets);
                app.logs.extend(cs_logs);
                app.add_log("Change set list refreshed.".to_string());
            }
            Err(e) => {
                app.change_set_list_state.select(None); // Ensure nothing selected on error
                let error_msg = format!("Error refreshing change sets: {}", e);
                app.add_log(error_msg);
            }
        }
    } else {
        app.add_log(
            "Cannot refresh change sets: Whoami data not available."
                .to_string(),
        );
    }
}

// Intention: Main application loop for handling events and rendering the UI.
// Design Choice: A loop that initializes state, fetches data, draws UI, and handles input asynchronously.
async fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    // Intention: Initialize application state using the new constructor.
    let mut app = App::new();

    // Intention: Perform initial data fetch (whoami and change sets) and log the process.
    // Design Choice: Call whoami first, then list_change_sets if whoami succeeds.
    app.add_log("Fetching initial /whoami data...".to_string());
    match api_client::whoami().await {
        Ok((whoami_data, whoami_logs)) => {
            let _workspace_id = whoami_data.workspace_id.clone(); // Prefix with _ as it's not directly used here
            app.whoami_data = Some(whoami_data);
            app.logs.extend(whoami_logs); // Add logs from the whoami call
            app.add_log("/whoami call successful.".to_string());
            // Initial fetch of change sets
            refresh_change_sets(&mut app).await;
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

        // Intention: Handle user input events asynchronously.
        // Design Choice: Poll for events with a timeout, handle keys, await API calls directly.
        if event::poll(Duration::from_millis(100))? {
            // Poll more frequently for responsiveness
            if let Event::Key(key) = event::read()? {
                // Clone necessary data *before* the mode match, so it's available in all arms.
                let selected_index = app.change_set_list_state.selected();
                let workspace_id =
                    app.whoami_data.as_ref().map(|d| d.workspace_id.clone());
                let selected_cs_id = selected_index.and_then(|idx| {
                    app.change_sets
                        .as_ref()
                        .and_then(|css| css.get(idx))
                        .map(|cs| cs.id.clone())
                });

                // Handle input based on the current mode
                match app.input_mode {
                    InputMode::Normal => {
                        // Normal mode key handling
                        let log_height = 10; // Must match the Constraint::Length in ui()

                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Up => app.change_set_previous(),
                            KeyCode::Down => app.change_set_next(),
                            KeyCode::Char('k') => app.scroll_logs_up(),
                            KeyCode::Char('j') => {
                                app.scroll_logs_down(log_height)
                            }

                            // --- Change Set Actions (Normal Mode) ---
                            KeyCode::Enter => {
                                // Toggle details pane only in Normal mode
                                app.show_details_pane = !app.show_details_pane;
                                if app.show_details_pane {
                                    if let (Some(ws_id), Some(cs_id)) =
                                        (workspace_id, selected_cs_id)
                                    {
                                        app.current_action = Some(
                                            "Fetching details...".to_string(),
                                        );
                                        terminal.draw(|f| ui(f, &app))?; // Redraw to show action message

                                        // Fetch details
                                        match api_client::get_change_set(
                                            &ws_id, &cs_id,
                                        )
                                        .await
                                        {
                                            // Extract the inner change_set from the response
                                            Ok((get_response, logs)) => {
                                                app.selected_change_set_details =
                                            Some(get_response.change_set); // Access inner field
                                                app.logs.extend(logs);
                                                app.add_log(format!(
                                                    "Details fetched for {}",
                                                    cs_id
                                                ));
                                            }
                                            Err(e) => {
                                                app.selected_change_set_details = None; // Clear on error
                                                app.add_log(format!(
                                            "Error fetching details for {}: {}",
                                            cs_id, e
                                        ));
                                            }
                                        }
                                        // Fetch merge status (using correct type MergeStatusV1Response)
                                        match api_client::get_merge_status(
                                            &ws_id, &cs_id,
                                        )
                                        .await
                                        {
                                            Ok((status_response, logs)) => {
                                                app.selected_change_set_merge_status =
                                            Some(status_response); // Store the full response
                                                app.logs.extend(logs);
                                                app.add_log(format!(
                                            "Merge status fetched for {}",
                                            cs_id
                                        ));
                                            }
                                            Err(e) => {
                                                app.selected_change_set_merge_status =
                                            None; // Clear on error
                                                app.add_log(format!("Error fetching merge status for {}: {}", cs_id, e));
                                            }
                                        }
                                        app.current_action = None; // Clear action message
                                    } else {
                                        app.add_log("Cannot fetch details: No workspace or changeset selected.".to_string());
                                        app.show_details_pane = false; // Don't show pane if we can't fetch
                                    }
                                } else {
                                    // Clear details when hiding pane
                                    app.selected_change_set_details = None;
                                    app.selected_change_set_merge_status = None;
                                }
                            }
                            KeyCode::Char('d') => {
                                // Delete Change Set
                                if let (Some(ws_id), Some(cs_id)) =
                                    (workspace_id, selected_cs_id)
                                {
                                    app.current_action =
                                        Some(format!("Deleting {}...", cs_id));
                                    terminal.draw(|f| ui(f, &app))?; // Redraw

                                    match api_client::delete_change_set(
                                        &ws_id, &cs_id,
                                    )
                                    .await
                                    {
                                        // Ensure matching the response tuple (DeleteChangeSetV1Response, Vec<String>)
                                        Ok((_delete_response, logs)) => {
                                            app.logs.extend(logs);
                                            app.add_log(format!(
                                                "Deleted changeset {}",
                                                cs_id
                                            ));
                                            // Clear details if they were for the deleted item
                                            if app
                                                .selected_change_set_details
                                                .as_ref()
                                                .map(|d| &d.id)
                                                == Some(&cs_id)
                                            {
                                                app.selected_change_set_details = None;
                                                app.selected_change_set_merge_status =
                                            None;
                                                app.show_details_pane = false;
                                            }
                                        }
                                        Err(e) => {
                                            app.add_log(format!(
                                        "Error deleting changeset {}: {}",
                                        cs_id, e
                                    ));
                                        }
                                    }
                                    app.current_action = None;
                                    refresh_change_sets(&mut app).await; // Refresh list
                                } else {
                                    app.add_log("Cannot delete: No workspace or changeset selected.".to_string());
                                }
                            }
                            KeyCode::Char('c') => {
                                // Enter ChangeSetName input mode
                                if workspace_id.is_some() {
                                    app.input_mode = InputMode::ChangeSetName;
                                    app.input_buffer.clear(); // Clear buffer for new input
                                    app.current_action = Some(
                                "Enter Change Set Name (Esc: Cancel, Enter: Create):"
                                    .to_string(),
                            );
                                } else {
                                    app.add_log(
                                "Cannot create: No workspace available."
                                    .to_string(),
                            );
                                }
                            }
                            KeyCode::Char('f') => {
                                // Force Apply Change Set
                                if let (Some(ws_id), Some(cs_id)) =
                                    (workspace_id, selected_cs_id)
                                {
                                    app.current_action =
                                        Some(format!("Applying {}...", cs_id));
                                    terminal.draw(|f| ui(f, &app))?; // Redraw

                                    match api_client::force_apply_change_set(
                                        &ws_id, &cs_id,
                                    )
                                    .await
                                    {
                                        Ok((_apply_response, logs)) => {
                                            // Prefix with _ as it's not used
                                            // Assuming apply returns some response struct
                                            app.logs.extend(logs);
                                            // TODO: Inspect _apply_response if needed
                                            app.add_log(format!(
                                        "Apply initiated for changeset {}",
                                        cs_id
                                    ));
                                            // Note: Apply might take time, status might change later. Refresh shows current state.
                                        }
                                        Err(e) => {
                                            app.add_log(format!(
                                        "Error applying changeset {}: {}",
                                        cs_id, e
                                    ));
                                        }
                                    }
                                    app.current_action = None;
                                    refresh_change_sets(&mut app).await; // Refresh list to see status update
                                } else {
                                    app.add_log("Cannot apply: No workspace or changeset selected.".to_string());
                                }
                            }

                            _ => {} // Ignore other keys in Normal mode
                        }
                    } // End Normal Mode Match KeyCode
                    InputMode::ChangeSetName => {
                        // Clone workspace_id again inside this scope to potentially help the compiler
                        let current_workspace_id = workspace_id.clone();
                        // ChangeSetName input mode key handling
                        match key.code {
                            KeyCode::Enter => {
                                // Attempt to create the change set
                                if let Some(ws_id) = current_workspace_id {
                                    // Use the cloned Option
                                    let new_cs_name =
                                        app.input_buffer.trim().to_string();
                                    if !new_cs_name.is_empty() {
                                        app.current_action = Some(format!(
                                            "Creating '{}'...",
                                            new_cs_name
                                        ));
                                        terminal.draw(|f| ui(f, &app))?; // Redraw

                                        // Construct the request object
                                        let request =
                                            CreateChangeSetV1Request {
                                                change_set_name: new_cs_name
                                                    .clone(),
                                            };

                                        match api_client::create_change_set(
                                            &ws_id, request,
                                        )
                                        .await
                                        {
                                            Ok((created_cs_response, logs)) => {
                                                app.logs.extend(logs);
                                                app.add_log(format!(
                                            "Created changeset '{}' ({})",
                                            created_cs_response.change_set.name,
                                            created_cs_response.change_set.id
                                        ));
                                            }
                                            Err(e) => {
                                                app.add_log(format!(
                                            "Error creating changeset: {}",
                                            e
                                        ));
                                            }
                                        }
                                        refresh_change_sets(&mut app).await; // Refresh list regardless of success/failure
                                    } else {
                                        app.add_log(
                                            "Change set name cannot be empty."
                                                .to_string(),
                                        );
                                    }
                                } else {
                                    app.add_log(
                                        "Cannot create: Workspace ID missing."
                                            .to_string(),
                                    );
                                }
                                // Exit input mode whether creation succeeded, failed, or was skipped
                                app.input_mode = InputMode::Normal;
                                app.input_buffer.clear();
                                app.current_action = None; // Clear prompt/action message
                            }
                            KeyCode::Char(c) => {
                                app.input_buffer.push(c); // Add character to buffer
                            }
                            KeyCode::Backspace => {
                                app.input_buffer.pop(); // Remove last character
                            }
                            KeyCode::Esc => {
                                // Cancel input mode
                                app.input_mode = InputMode::Normal;
                                app.input_buffer.clear();
                                app.current_action = None; // Clear prompt message
                                app.add_log(
                                    "Change set creation cancelled."
                                        .to_string(),
                                );
                            }
                            _ => {} // Ignore other keys in this input mode
                        }
                    } // End ChangeSetName Mode Match KeyCode
                } // End Match app.input_mode
            } // End Key Event Handling
        } // End Event Poll
        // Placeholder for other async tasks or periodic refresh if needed later
        // tokio::time::sleep(Duration::from_millis(50)).await; // Small sleep to prevent busy-looping if no events
    }
    // Note: Loop is infinite, exit happens via `return Ok(())` on 'q' press.
}

use ratatui::text::Line; // Import Line for constructing text

// Intention: Define the UI layout and render widgets based on application state.
// Design Choice: A layout displaying whoami data, logs, list, optional details pane, and conditional input line.
fn ui(f: &mut Frame, app: &App) {
    // Intention: Create the main layout with a top bar, main content, log area, and optional input line.
    // Design Choice: Vertical layout. Input line appears at the bottom if input_mode is ChangeSetName.
    let log_height = 10; // Define default height for the log window
    let (log_constraint, input_constraint) =
        if app.input_mode == InputMode::ChangeSetName {
            (Constraint::Length(log_height), Constraint::Length(1)) // Log area + Input line
        } else {
            (Constraint::Length(log_height), Constraint::Length(0)) // Log area only, no input line
        };

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top bar for email/status
            Constraint::Min(0),    // Main content area (flexible)
            log_constraint,        // Log window area (potentially adjusted)
            input_constraint,      // Input line area (0 or 1 height)
        ])
        .split(f.size());

    let top_bar_area = main_chunks[0];
    let content_area = main_chunks[1];
    let log_area = main_chunks[2];
    let input_area = if main_chunks.len() > 3 {
        Some(main_chunks[3])
    } else {
        None
    };

    // Intention: Display user email in the top right corner.
    // Design Choice: Create a paragraph aligned to the right within the top_bar_area.
    let email_text = match &app.whoami_data {
        Some(data) => data.user_email.clone(),
        None => "".to_string(), // Show nothing if data isn't loaded yet
    };
    // Intention: Display user email and current action status in the top bar.
    // Design Choice: Split top bar horizontally. Email on right, status on left.
    let top_bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // Space for status/action message
            Constraint::Percentage(30), // Space for email
        ])
        .split(top_bar_area);

    let status_area = top_bar_chunks[0];
    let email_area = top_bar_chunks[1];

    let email_paragraph =
        Paragraph::new(email_text).alignment(Alignment::Right);
    f.render_widget(email_paragraph, email_area);

    // Display current action if any
    if let Some(action) = &app.current_action {
        // Display action message in status bar, unless it's the input prompt
        if app.input_mode != InputMode::ChangeSetName {
            let action_paragraph = Paragraph::new(action.as_str())
                .style(Style::default().fg(Color::Yellow)); // Highlight action
            f.render_widget(action_paragraph, status_area);
        }
    }
    // Note: Input prompt is now handled at the bottom

    // Intention: Create a block for the main content area below the top bar.
    // Design Choice: Add borders. Title indicates main content.
    let content_block =
        Block::default().title("Main Content").borders(Borders::ALL);
    let inner_content_area = content_block.inner(content_area);
    f.render_widget(content_block, content_area); // Render the block frame

    // Intention: Divide the main content area based on whether the details pane is shown.
    // Design Choice: If details shown, split horizontally (list | details). Otherwise, list takes full width.
    let content_chunks = if app.show_details_pane {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // List area
                Constraint::Percentage(50), // Details area
            ])
            .split(inner_content_area) // Split the area *inside* the content block
    } else {
        // Only the list area is needed
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(inner_content_area)
    };

    let list_and_info_area = content_chunks[0]; // Always exists

    // Intention: Divide the list area vertically for user info and the change set list.
    // Design Choice: Split vertically. User info on top, list below.
    let list_info_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Fixed height for user info
            Constraint::Min(0),    // Remaining space for the change set list
        ])
        .split(list_and_info_area); // Split the left/main pane

    let user_info_area = list_info_chunks[0];
    let change_set_list_area = list_info_chunks[1];

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
    // Design Choice: Use `List` widget, map `ChangeSetSummary` to `ListItem`. Highlight selected. Add key hints.
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

    let list_title = match app.input_mode {
        InputMode::Normal => {
            if app.show_details_pane {
                "Change Sets (Up/Down, Enter: Close Details, d: Del, f: Apply)"
            } else {
                "Change Sets (Up/Down, Enter: Show Details, c: Create)"
            }
        }
        InputMode::ChangeSetName => "Change Sets (Input Mode Active)", // Indicate input mode in list title
    };
    let change_set_list = List::new(change_set_items)
        .block(Block::default().title(list_title).borders(Borders::NONE)) // Title for the list area
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

    // --- Details Pane Rendering (Conditional) ---
    if app.show_details_pane && content_chunks.len() > 1 {
        let details_area = content_chunks[1];
        let details_block = Block::default()
            .title("Details (Enter: Close)")
            .borders(Borders::ALL);
        let inner_details_area = details_block.inner(details_area);
        f.render_widget(details_block, details_area);

        let mut details_text = Vec::new();

        // Display Change Set Details
        if let Some(details) = &app.selected_change_set_details {
            details_text.push(Line::from(format!("ID: {}", details.id)));
            details_text.push(Line::from(format!("Name: {}", details.name)));
            details_text
                .push(Line::from(format!("Status: {}", details.status)));
            // TODO: Add more fields from ChangeSet if they exist
            details_text.push(Line::from("---")); // Separator
        } else {
            details_text.push(Line::from("Loading details..."));
        }

        // Display Merge Status
        if let Some(status) = &app.selected_change_set_merge_status {
            details_text.push(Line::from("Merge Status:"));
            if status.actions.is_empty() {
                details_text.push(Line::from("  No actions required."));
            } else {
                for action in &status.actions {
                    let component_info = action
                        .component
                        .as_ref()
                        .map_or_else(String::new, |c| format!(" ({})", c.name));
                    details_text.push(Line::from(format!(
                        "  - {} {} {}",
                        action.kind, action.name, component_info
                    )));
                }
            }
        } else {
            details_text.push(Line::from("Loading merge status..."));
        }

        let details_paragraph =
            Paragraph::new(details_text).wrap(Wrap { trim: false });
        f.render_widget(details_paragraph, inner_details_area);
    }

    // --- Log Window Rendering ---

    // Intention: Create a block for the log window.
    // Design Choice: Add borders and a title with scroll hints.
    let log_block = Block::default()
        .title("Logs (j/k: Scroll)")
        .borders(Borders::ALL);
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

    // --- Input Line Rendering (Conditional) ---
    if let Some(input_area) = input_area {
        if app.input_mode == InputMode::ChangeSetName {
            let input_prompt_text =
                app.current_action.as_deref().unwrap_or("Enter Name:"); // Use action as prompt if set
            let input_paragraph = Paragraph::new(format!(
                "{} {}{}", // Prompt, buffer, cursor
                input_prompt_text,
                app.input_buffer,
                "_" // Simple cursor indicator
            ))
            .style(Style::default().fg(Color::Yellow));
            f.render_widget(input_paragraph, input_area);

            // Optional: Position the actual terminal cursor (might flicker)
            // f.set_cursor(
            //     input_area.x + input_prompt_text.len() as u16 + 1 + app.input_buffer.len() as u16,
            //     input_area.y,
            // );
        }
    }
}
