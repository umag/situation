// src/ui/render_log_panel.rs

// Intention: Render the log panel at the bottom.
// Design Choice: Encapsulates the log block (with dynamic title) and the scrollable log paragraph. Extracted from ui.rs.

use ratatui::{
    Frame,
    layout::Rect,
    prelude::*, // Import common traits and types
    widgets::{
        Block,
        Borders,
        Paragraph,
        Wrap,
    },
};

use crate::app::App; // Use App from local app module

// Intention: Render the log panel at the bottom.
// Design Choice: Encapsulates the log block (with dynamic title) and the scrollable log paragraph.
pub(super) fn render_log_panel(f: &mut Frame, app: &App, area: Rect) {
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
