// Intention: Define the application entry point and terminal setup/teardown.
// Design Choice: This file now only contains the `main` function.
// It declares the other modules (`app`, `refresh_change_sets`, `run_app`, `ui`)
// and calls `run_app::run_app` to start the TUI.

// Declare modules created from splitting the original main.rs
mod app;
mod refresh_change_sets;
mod run_app;
mod ui;

use std::{
    error::Error,
    io,
};

use crossterm::{
    event::{
        DisableMouseCapture,
        EnableMouseCapture,
    }, // Only need mouse capture events here
    execute,
    terminal::{
        EnterAlternateScreen,
        LeaveAlternateScreen,
        disable_raw_mode,
        enable_raw_mode,
    },
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
};
// Use the run_app function from the newly created module
use run_app::run_app;
use tokio;

// Intention: Entry point for the TUI application.
// Design Choice: Using tokio::main for the async `run_app` function.
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

    // Intention: Run the main application loop by calling the function from the run_app module.
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
        // Keep simple error printing to stdout after terminal is restored
        println!("Error running app: {:?}", err);
    }

    Ok(())
}
