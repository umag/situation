// src/ui/render_input_line.rs

// Intention: Render the input line when in ChangeSetName mode.
// Design Choice: Encapsulates the conditional rendering of the input prompt and buffer. Extracted from ui.rs.

use ratatui::{
    Frame,
    layout::Rect,
    prelude::*, // Import common traits and types
    style::{
        Color,
        Style,
    },
    widgets::Paragraph,
};

use crate::app::{
    App,
    InputMode,
}; // Use App, Enums from local app module

// Intention: Render the input line when in ChangeSetName mode.
// Design Choice: Encapsulates the conditional rendering of the input prompt and buffer.
pub(super) fn render_input_line(f: &mut Frame, app: &App, area: Rect) {
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
