//! Title bar component for dashboard.

use ratatui::widgets::Paragraph;
use ratatui::prelude::Stylize;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::Frame;

/* IMPLEMENTATIONS */

pub fn render(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::raw(" "),
        Span::styled("Nova Scheduler", Style::default().bold()),
        Span::raw("  Press 'q' or ESC to exit"),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
