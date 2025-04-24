// src/ui/get_trigger_style.rs

// Intention: Get the appropriate style for a top bar trigger based on focus state.
// Design Choice: Centralizes the focus style logic to avoid duplication. Extracted from ui.rs.

use ratatui::style::{
    Color,
    Style,
};

// Intention: Get the appropriate style for a top bar trigger based on focus state.
// Design Choice: Centralizes the focus style logic to avoid duplication.
pub(super) fn get_trigger_style(is_focused: bool) -> Style {
    if is_focused {
        Style::default().bg(Color::Blue).fg(Color::White) // Focused style
    } else {
        Style::default() // Normal style
    }
}
