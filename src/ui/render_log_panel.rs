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

use crate::app::{
    App,
    AppFocus,
}; // Use App from local app module

// Intention: Render the log panel at the bottom. Highlights border on focus.
// Design Choice: Encapsulates the log block (with dynamic title using Spans) and the scrollable log paragraph.
pub(super) fn render_log_panel(f: &mut Frame, app: &App, area: Rect) {
    // Determine border style based on focus
    let border_style = if app.current_focus == AppFocus::LogPanel {
        Style::default().fg(Color::Cyan) // Highlight color when focused
    } else {
        Style::default().fg(Color::DarkGray) // Default color when not focused
    };

    // Construct the title with highlighted 'L' and optional action
    let mut title_spans = vec![
        Span::styled("L", Style::default().fg(Color::Yellow)), // Highlighted 'L'
        Span::raw("ogs (j/k: Scroll)"), // Rest of base title
    ];
    if let Some(action) = &app.current_action {
        title_spans.push(Span::raw(" - ["));
        title_spans
            .push(Span::styled(action, Style::default().fg(Color::Cyan))); // Style the action
        title_spans.push(Span::raw("]"));
    }
    let log_title_line = Line::from(title_spans).alignment(Alignment::Left); // Align title left

    let log_block = Block::default()
        .title(log_title_line) // Use the constructed Line
        .borders(Borders::ALL)
        .border_style(border_style); // Apply conditional border style
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
