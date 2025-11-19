//! Rendering functions for dashboard TUI.

use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;

use std::collections::HashMap;

use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::TaskContextSnapshot;
use crate::scheduler::TaskID;
use crate::scheduler::TaskState;

use super::SortOrder;
use super::TaskFilter;
use super::format::*;
use super::layout::*;
use super::state::*;

/* CONSTANTS */

const WIDTH: usize = 30;

/* STRUCTURES */

/// State bar renderer for visual task state representation.
pub struct StateBar;

impl StateBar {
    pub fn render(counts: &StateCounts, width: usize) -> Line<'static> {
        if width == 0 {
            return Line::from(" ");
        }

        let segments = Self::calculate_segments(counts, width);
        let spans = Self::build_spans(&segments);
        Line::from(spans)
    }

    fn calculate_segments(counts: &StateCounts, width: usize) -> BarSegments {
        let total = counts.running
            + counts.ready
            + counts.waiting
            + counts.suspended
            + counts.errors;

        if total == 0 {
            return BarSegments {
                running: 0,
                ready: 0,
                waiting: 0,
                suspended: 0,
                errors: 0,
            };
        }

        let running = ((counts.running as f64 / total as f64) * width as f64)
            .round() as usize;

        let ready = ((counts.ready as f64 / total as f64) * width as f64)
            .round() as usize;

        let waiting = ((counts.waiting as f64 / total as f64) * width as f64)
            .round() as usize;

        let suspended = ((counts.suspended as f64 / total as f64)
            * width as f64)
            .round() as usize;

        let errors = ((counts.errors as f64 / total as f64) * width as f64)
            .round() as usize;

        let used = running + ready + waiting + suspended + errors;
        let adjust = if used < width {
            width - used
        } else if used > width {
            width.saturating_sub(used - errors)
        } else {
            0
        };

        BarSegments {
            running,
            ready,
            waiting,
            suspended,
            errors: errors + adjust,
        }
    }

    fn build_spans(segments: &BarSegments) -> Vec<Span<'static>> {
        let mut spans = vec![Span::raw(" ")];
        if segments.running > 0 {
            spans.push(Span::styled(
                "━".repeat(segments.running),
                Style::default().fg(Color::Green),
            ));
        }

        if segments.ready > 0 {
            spans.push(Span::styled(
                "━".repeat(segments.ready),
                Style::default().fg(Color::Blue),
            ));
        }

        if segments.waiting > 0 {
            spans.push(Span::styled(
                "━".repeat(segments.waiting),
                Style::default().fg(Color::Cyan),
            ));
        }

        if segments.suspended > 0 {
            spans.push(Span::styled(
                "━".repeat(segments.suspended),
                Style::default().fg(Color::Gray),
            ));
        }

        if segments.errors > 0 {
            spans.push(Span::styled(
                "━".repeat(segments.errors),
                Style::default().fg(Color::Red),
            ));
        }

        spans.push(Span::raw(" "));
        spans
    }
}

/* TASK OPERATIONS */

pub fn collect_tasks<'a>(
    snapshot: &'a SchedulerSnapshot,
    filters: &[TaskFilter],
    matches_filter: impl Fn(&TaskState, &[TaskFilter]) -> bool,
) -> Vec<(TaskID, &'a TaskContextSnapshot)> {
    let matches = |(_, ctx): &(&TaskID, &TaskContextSnapshot)| {
        matches_filter(&ctx.state, filters)
    };

    snapshot
        .tasks
        .iter()
        .filter(matches)
        .map(|(tid, ctx)| (*tid, ctx))
        .collect()
}

pub fn sort_tasks(
    tasks: &mut [(TaskID, &TaskContextSnapshot)],
    sort_order: SortOrder,
    task_start_times: &HashMap<TaskID, std::time::Instant>,
) {
    match sort_order {
        SortOrder::Progress => {
            let percent = |ctx: &TaskContextSnapshot| {
                ctx.progress.and_then(|p| {
                    ctx.size
                        .map(|s| p as f64 / s as f64)
                })
            };

            tasks.sort_by(|a, b| {
                let prog_a = percent(a.1);
                let prog_b = percent(b.1);
                prog_b
                    .partial_cmp(&prog_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.0.cmp(&b.0))
            });
        },
        SortOrder::TaskID => {
            tasks.sort_by_key(|(tid, _)| *tid);
        },
        SortOrder::StartTime => {
            let time = |tid: &TaskID| task_start_times.get(tid);
            tasks.sort_by(|a, b| {
                time(&a.0)
                    .cmp(&time(&b.0))
                    .then_with(|| a.0.cmp(&b.0))
            });
        },
    }
}

/* RENDERING - TOP LEVEL */

pub fn render_header(
    frame: &mut ratatui::Frame,
    area: Rect,
    snapshot: &SchedulerSnapshot,
    _throughput: &HashMap<TaskID, f64>,
) {
    let counts = count_states(snapshot);
    let bar_line =
        StateBar::render(&counts, area.width.saturating_sub(4) as usize);

    let title = format!(
        " Nova Scheduler │ Tick: {} │ Total: {} tasks │ Press 'q' or ESC to exit ",
        snapshot.tick,
        snapshot.tasks.len()
    );

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" Running: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{} ", counts.running)),
            Span::styled("• ", Style::default().fg(Color::White)),
            Span::styled("Ready: ", Style::default().fg(Color::Blue)),
            Span::raw(format!("{} ", counts.ready)),
            Span::styled("• ", Style::default().fg(Color::White)),
            Span::styled("Waiting: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} ", counts.waiting)),
            Span::styled("• ", Style::default().fg(Color::White)),
            Span::styled("Suspended: ", Style::default().fg(Color::Gray)),
            Span::raw(format!("{} ", counts.suspended)),
            Span::styled("• ", Style::default().fg(Color::White)),
            Span::styled("Errors: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}", counts.errors)),
        ]),
        bar_line,
        Line::from(""),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White)),
    );

    frame.render_widget(paragraph, area);
}

pub fn render_section(
    frame: &mut ratatui::Frame,
    area: Rect,
    title: &str,
    tasks: &[(TaskID, &TaskContextSnapshot)],
    snapshot: &SchedulerSnapshot,
    spinner: usize,
    max_deps: usize,
    throughput: &HashMap<TaskID, f64>,
    show_time: bool,
    spinner_char: impl Fn(usize) -> &'static str,
    task_icon: impl Fn(&TaskState) -> &'static str,
    task_badge: impl Fn(&TaskState) -> (&'static str, Color),
) {
    let available = area.height.saturating_sub(3) as usize;
    let max_capacity = available / LINES_PER_TASK;

    let (visible, overflow) = if tasks.len() <= max_capacity {
        (tasks, &[][..])
    } else {
        let capacity = (available - OVERFLOW_LINES) / LINES_PER_TASK;
        (&tasks[..capacity], &tasks[capacity..])
    };

    let mut lines = vec![Line::from("")];
    for (tid, ctx) in visible {
        lines.extend(render_task(
            *tid,
            ctx,
            snapshot,
            spinner,
            max_deps,
            throughput,
            &spinner_char,
            &task_icon,
            &task_badge,
        ));
    }

    if !overflow.is_empty() {
        lines.push(render_overflow(overflow, throughput, show_time));
    }

    let block = format!(" {}: {} ", title, tasks.len());
    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(block)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White)),
    );

    frame.render_widget(paragraph, area);
}

/* RENDERING - TASK DETAILS */

fn render_task<'a>(
    tid: TaskID,
    ctx: &'a TaskContextSnapshot,
    snapshot: &'a SchedulerSnapshot,
    spinner: usize,
    max_deps: usize,
    throughput: &'a HashMap<TaskID, f64>,
    spinner_char: impl Fn(usize) -> &'static str,
    task_icon: impl Fn(&TaskState) -> &'static str,
    task_badge: impl Fn(&TaskState) -> (&'static str, Color),
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    lines.push(task_header(
        ctx,
        spinner,
        &spinner_char,
        &task_icon,
        &task_badge,
    ));
    if let Some(line) = render_progress(ctx, throughput.get(&tid).copied()) {
        lines.push(line);
    }

    if let Some(line) = render_deps(ctx, snapshot, max_deps) {
        lines.push(line);
    }

    lines.push(Line::from(""));
    lines
}

fn task_header(
    ctx: &TaskContextSnapshot,
    spinner: usize,
    spinner_char: impl Fn(usize) -> &'static str,
    task_icon: impl Fn(&TaskState) -> &'static str,
    task_badge: impl Fn(&TaskState) -> (&'static str, Color),
) -> Line<'_> {
    let icon = if matches!(
        ctx.state,
        TaskState::Running | TaskState::Preempting
    ) {
        spinner_char(spinner)
    } else {
        task_icon(&ctx.state)
    };

    let (badge, badge_color) = task_badge(&ctx.state);
    Line::from(vec![
        Span::raw(" "),
        Span::styled(icon, Style::default().fg(Color::White)),
        Span::raw(" "),
        Span::styled(
            format!("[{}]", badge),
            Style::default()
                .fg(badge_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!(
                "{}: {}",
                format_task_id(&ctx.category, ctx.component),
                &ctx.about
            ),
            Style::default().fg(Color::White),
        ),
    ])
}

fn render_progress(
    ctx: &TaskContextSnapshot,
    throughput: Option<f64>,
) -> Option<Line<'_>> {
    let progress = ctx.progress?;
    let spans = match ctx.size {
        Some(size) => progress_sized(progress, size, throughput),
        None => progress_unsized(progress, throughput),
    };
    Some(Line::from(spans))
}

fn render_deps<'a>(
    ctx: &'a TaskContextSnapshot,
    snapshot: &'a SchedulerSnapshot,
    max_deps: usize,
) -> Option<Line<'a>> {
    if let TaskState::Waiting(deps) = &ctx.state {
        let list: Vec<TaskID> = deps.iter().copied().collect();
        let text = format_deps(&list, snapshot, max_deps);
        if !text.is_empty() {
            return Some(Line::from(vec![
                Span::raw("    "),
                Span::styled(text, Style::default().fg(Color::Gray)),
            ]));
        }
    }

    None
}

fn render_overflow(
    overflow: &[(TaskID, &TaskContextSnapshot)],
    throughput: &HashMap<TaskID, f64>,
    show_time: bool,
) -> Line<'static> {
    let count = overflow.len();
    let message = if show_time {
        let max_eta = compute_max_eta(overflow, throughput);
        match max_eta {
            Some(eta) => format!(
                "    ...and {} more (maximum remaining time is {}).",
                count,
                format_duration(eta)
            ),
            None => format!("    ...and {} more tasks.", count),
        }
    } else {
        format!("    ...and {} more tasks.", count)
    };

    Line::from(Span::styled(
        message,
        Style::default().fg(Color::DarkGray),
    ))
}

/* RENDERING - PROGRESS BARS */

fn render_bar(progress: u64, size: u64) -> String {
    let ratio = progress as f64 / size as f64;
    let filled = (ratio * WIDTH as f64) as usize;
    let empty = WIDTH - filled;
    format!("{}{}", "━".repeat(filled), "─".repeat(empty))
}

fn progress_sized(
    progress: u64,
    size: u64,
    throughput: Option<f64>,
) -> Vec<Span<'static>> {
    let percent = (progress as f64 / size as f64 * 100.0) as u16;
    let bar = render_bar(progress, size);
    let mut spans = vec![
        Span::raw("   "),
        Span::styled(bar, Style::default().fg(Color::White)),
        Span::raw(format!(" {}% ({}/{})", percent, progress, size)),
    ];

    if let Some(rate) = throughput
        && rate > 0.0
    {
        add_eta(&mut spans, size, progress, rate);
    }

    spans
}

fn progress_unsized(
    progress: u64,
    throughput: Option<f64>,
) -> Vec<Span<'static>> {
    let mut spans = vec![
        Span::raw("   Progress: "),
        Span::styled(
            format!("{} ops", progress),
            Style::default().fg(Color::White),
        ),
    ];

    if let Some(rate) = throughput {
        add_rate(&mut spans, rate);
    }

    spans
}

fn add_eta(
    spans: &mut Vec<Span<'static>>,
    size: u64,
    progress: u64,
    rate: f64,
) {
    let remaining = size.saturating_sub(progress) as f64;
    let eta = remaining / rate;
    add_rate(spans, rate);
    spans.push(Span::raw(" │ Left: "));
    spans.push(Span::styled(
        format_duration(eta),
        Style::default().fg(Color::White),
    ));
}

fn add_rate(spans: &mut Vec<Span<'static>>, rate: f64) {
    spans.push(Span::raw(" @ "));
    spans.push(Span::styled(
        format!("{:.1} ops/s", rate),
        Style::default().fg(Color::White),
    ));
}

fn compute_max_eta(
    tasks: &[(TaskID, &TaskContextSnapshot)],
    throughput: &HashMap<TaskID, f64>,
) -> Option<f64> {
    let eta = |(tid, ctx): &(TaskID, &TaskContextSnapshot)| {
        let progress = ctx.progress?;
        let size = ctx.size?;
        let rate = throughput.get(tid).copied()?;
        if rate <= 0.0 {
            return None;
        }
        let remaining = size.saturating_sub(progress) as f64;
        Some(remaining / rate)
    };

    tasks
        .iter()
        .filter_map(eta)
        .max_by(|a, b| {
            a.partial_cmp(b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}
