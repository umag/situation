use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io};
use tokio;

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
// Design Choice: A loop that continuously draws the UI and handles input events.
async fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    loop {
        // Intention: Draw the current state of the UI.
        terminal.draw(|f| ui(f))?;

        // Intention: Handle user input events.
        // Design Choice: Poll for events with a timeout to allow for UI updates.
        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                // Intention: Exit the application when 'q' is pressed.
                if key.code == KeyCode::Char('q') {
                    return Ok(());
                }
            }
        }
        // Placeholder for fetching data from luminork
        // let data = fetch_data().await?;
        // Update app state with data
    }
}

// Intention: Define the UI layout and render widgets.
// Design Choice: A simple layout with a single paragraph for now.
fn ui(f: &mut Frame) { // Removed <B: Backend> and <B> from Frame
    // Intention: Create the main layout structure.
    // Design Choice: A single vertical chunk covering the entire screen.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    // Intention: Create a block to contain the main content.
    // Design Choice: A simple block with borders and a title.
    let block = Block::default().title("Systeminit/si TUI").borders(Borders::ALL);
    f.render_widget(block, chunks[0]);

    // Intention: Display placeholder text.
    // Design Choice: A paragraph widget within the main block.
    let paragraph = Paragraph::new("Press 'q' to quit. API data will be shown here.")
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, chunks[0]); // Render inside the block's area
}

// Placeholder for luminork API call function
// async fn fetch_data() -> Result<String, Box<dyn Error>> {
//     // TODO: Implement luminork client and API call
//     Ok("Fetched data from luminork".to_string())
// }
#[cfg(test)]
mod tests {
    // Removed: use super::*; (unused)

    #[test]
    fn basic_test() {
        // Intention: Basic sanity check test.
        // Design Choice: Simple assertion to ensure tests run.
        assert_eq!(2 + 2, 4);
    }
}
