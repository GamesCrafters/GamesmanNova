//! Progress bar rendering and ETA calculation.

use std::cmp::Ordering;
use std::collections::HashMap;

use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;

use crate::scheduler::TaskContextSnapshot;
use crate::scheduler::TaskID;

use super::format::format_duration;

/* CONSTANTS */

const PROGRESS_WIDTH: usize = 30;

/* IMPLEMENTATIONS */

pub fn render(
    ctx: &TaskContextSnapshot,
    rate: Option<f64>,
) -> Option<Line<'static>> {
    let progress = ctx.progress?;
    let spans = match ctx.size {
        Some(size) => sized(progress, size, rate),
        None => unbounded(progress, rate),
    };
    Some(Line::from(spans))
}

pub fn max_eta(
    tasks: &[(TaskID, &TaskContextSnapshot)],
    throughput: &HashMap<TaskID, f64>,
) -> Option<f64> {
    let eta = |(tid, ctx): &(TaskID, &TaskContextSnapshot)| {
        let progress = ctx.progress?;
        let size = ctx.size?;
        let rate = throughput
            .get(tid)
            .copied()
            .filter(|&r| r > 0.0)?;
        Some(size.saturating_sub(progress) as f64 / rate)
    };

    tasks
        .iter()
        .filter_map(eta)
        .max_by(|a, b| {
            a.partial_cmp(b)
                .unwrap_or(Ordering::Equal)
        })
}

fn sized(progress: u64, size: u64, rate: Option<f64>) -> Vec<Span<'static>> {
    let pct = (progress as f64 / size as f64 * 100.0) as u16;
    let b = bar(progress, size);

    let mut spans = vec![
        Span::raw("   "),
        Span::styled(b, Style::default().fg(Color::White)),
        Span::raw(format!(" {}% ({}/{})", pct, progress, size)),
    ];

    if let Some(r) = rate
        && r > 0.0
    {
        eta_spans(&mut spans, size, progress, r);
    }

    spans
}

fn unbounded(progress: u64, rate: Option<f64>) -> Vec<Span<'static>> {
    let mut spans = vec![
        Span::raw("   Progress: "),
        Span::styled(
            format!("{} ops", progress),
            Style::default().fg(Color::White),
        ),
    ];

    if let Some(r) = rate {
        rate_spans(&mut spans, r);
    }

    spans
}

fn bar(progress: u64, size: u64) -> String {
    let ratio = progress as f64 / size as f64;
    let filled = (ratio * PROGRESS_WIDTH as f64) as usize;
    let empty = PROGRESS_WIDTH - filled;
    format!("{}{}", "━".repeat(filled), "─".repeat(empty))
}

fn eta_spans(
    spans: &mut Vec<Span<'static>>,
    size: u64,
    progress: u64,
    rate: f64,
) {
    let remaining = size.saturating_sub(progress) as f64;
    let eta = remaining / rate;
    rate_spans(spans, rate);
    spans.push(Span::raw(" │ Left: "));
    spans.push(Span::styled(
        format_duration(eta),
        Style::default().fg(Color::White),
    ));
}

fn rate_spans(spans: &mut Vec<Span<'static>>, rate: f64) {
    spans.push(Span::raw(" @ "));
    spans.push(Span::styled(
        format!("{:.1} ops/s", rate),
        Style::default().fg(Color::White),
    ));
}
