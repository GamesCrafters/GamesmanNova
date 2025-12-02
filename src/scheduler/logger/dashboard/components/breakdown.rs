//! Task state breakdown bar component.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::scheduler::SchedulerSnapshot;

use super::super::support::state::BarSegments;
use super::super::support::state::StateCounts;
use super::super::support::state::count_states;

/* IMPLEMENTATIONS */

pub fn render(frame: &mut Frame, area: Rect, snapshot: &SchedulerSnapshot) {
    let counts = count_states(snapshot);
    let width = area.width as usize;

    let lines = vec![
        status_line(&counts),
        render_state_bar(&counts, width),
    ];

    frame.render_widget(Paragraph::new(lines), area);
}

fn styled_span(label: &'static str, color: Color) -> Span<'static> {
    Span::styled(label, Style::default().fg(color))
}

fn status_line(counts: &StateCounts) -> Line<'static> {
    let dot = styled_span("• ", Color::White);

    Line::from(vec![
        styled_span(" Running: ", Color::Green),
        Span::raw(format!("{} ", counts.running)),
        dot.clone(),
        styled_span("Ready: ", Color::Blue),
        Span::raw(format!("{} ", counts.ready)),
        dot.clone(),
        styled_span("Waiting: ", Color::Cyan),
        Span::raw(format!("{} ", counts.waiting)),
        dot.clone(),
        styled_span("Suspended: ", Color::Gray),
        Span::raw(format!("{} ", counts.suspended)),
        dot,
        styled_span("Errors: ", Color::Red),
        Span::raw(format!("{}", counts.errors)),
    ])
}

fn render_state_bar(counts: &StateCounts, width: usize) -> Line<'static> {
    Line::from(bar_spans(&bar_segments(counts, width)))
}

fn compute_ratio(count: usize, total: usize, width: usize) -> f64 {
    ((count as f64 / total as f64) * width as f64).round()
}

fn bar_segments(counts: &StateCounts, width: usize) -> BarSegments {
    let total = counts.suspended
        + counts.running
        + counts.waiting
        + counts.errors
        + counts.ready;

    if total == 0 {
        return BarSegments {
            errors: width,
            suspended: 0,
            running: 0,
            waiting: 0,
            ready: 0,
        };
    }

    let suspended = compute_ratio(counts.suspended, total, width) as usize;
    let running = compute_ratio(counts.running, total, width) as usize;
    let waiting = compute_ratio(counts.waiting, total, width) as usize;
    let errors = compute_ratio(counts.errors, total, width) as usize;
    let ready = compute_ratio(counts.ready, total, width) as usize;

    let used = suspended + running + waiting + errors + ready;
    let adjust = width.saturating_sub(used);

    BarSegments {
        errors: errors + adjust,
        suspended,
        running,
        waiting,
        ready,
    }
}

fn bar_span(count: usize, color: Color) -> Span<'static> {
    Span::styled("▃".repeat(count), Style::default().fg(color))
}

fn bar_spans(seg: &BarSegments) -> Vec<Span<'static>> {
    vec![
        bar_span(seg.running, Color::Green),
        bar_span(seg.ready, Color::Blue),
        bar_span(seg.waiting, Color::Cyan),
        bar_span(seg.suspended, Color::Gray),
        bar_span(seg.errors, Color::Red),
    ]
}
