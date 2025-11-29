//! Rendering functions for dashboard TUI.

use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::once;

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

use crate::scheduler::RunnerSnapshot;
use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::TaskContextSnapshot;
use crate::scheduler::TaskID;
use crate::scheduler::TaskState;

use super::format::format_deps;
use super::format::format_duration;
use super::format::format_task_id;
use super::format::format_tick;
use super::histogram;
use super::layout::HEADER_PADDING;
use super::layout::LINES_PER_TASK;
use super::layout::OVERFLOW_LINES;
use super::progress;
use super::state::BarSegments;
use super::state::StateCounts;
use super::state::count_states;

/* IMPLEMENTATIONS */

/* Header Rendering */

pub fn header(
    frame: &mut ratatui::Frame,
    area: Rect,
    snapshot: &SchedulerSnapshot,
    runner: Option<&RunnerSnapshot>,
    centroid_values: &[u64],
    centroid_counts: &[u64],
    label_every_n: usize,
) {
    let counts = count_states(snapshot);
    let width = area
        .width
        .saturating_sub(HEADER_PADDING) as usize;

    let workers = runner
        .map(|r| format!("{}/{}", counts.running, r.capacity))
        .unwrap_or_else(|| format!("{}", counts.running));

    let title = format!(
        " Tick: {} │ Workers: {} │ Tasks: {} │ Press 'q' or ESC to exit ",
        snapshot.tick,
        workers,
        snapshot.tasks.len()
    );

    let mut lines = vec![
        Line::from(""),
        status_line(&counts),
        render_state_bar(&counts, width),
        Line::from(""),
    ];

    if runner.is_some() {
        lines.extend(runner_lines(centroid_values, centroid_counts, label_every_n));
    }

    let block = Block::default()
        .border_style(Style::default().fg(Color::White))
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .title(title);

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn status_line(counts: &StateCounts) -> Line<'static> {
    let styled = |label, color| Span::styled(label, Style::default().fg(color));
    let dot = styled("• ", Color::White);

    Line::from(vec![
        styled(" Running: ", Color::Green),
        Span::raw(format!("{} ", counts.running)),
        dot.clone(),
        styled("Ready: ", Color::Blue),
        Span::raw(format!("{} ", counts.ready)),
        dot.clone(),
        styled("Waiting: ", Color::Cyan),
        Span::raw(format!("{} ", counts.waiting)),
        dot.clone(),
        styled("Suspended: ", Color::Gray),
        Span::raw(format!("{} ", counts.suspended)),
        dot,
        styled("Errors: ", Color::Red),
        Span::raw(format!("{}", counts.errors)),
    ])
}

/* State Bar */

pub fn render_state_bar(counts: &StateCounts, width: usize) -> Line<'static> {
    if width == 0 {
        return Line::from(" ");
    }
    Line::from(bar_spans(&bar_segments(counts, width)))
}

fn bar_segments(counts: &StateCounts, width: usize) -> BarSegments {
    let total = counts.suspended
        + counts.running
        + counts.waiting
        + counts.errors
        + counts.ready;

    if total == 0 {
        return BarSegments::default();
    }

    let ratio = |n: usize| ((n as f64 / total as f64) * width as f64).round();

    let suspended = ratio(counts.suspended) as usize;
    let running = ratio(counts.running) as usize;
    let waiting = ratio(counts.waiting) as usize;
    let errors = ratio(counts.errors) as usize;
    let ready = ratio(counts.ready) as usize;

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

fn bar_spans(seg: &BarSegments) -> Vec<Span<'static>> {
    let bar = |n, c| Span::styled("━".repeat(n), Style::default().fg(c));

    [
        Some(Span::raw(" ")),
        (seg.running > 0).then(|| bar(seg.running, Color::Green)),
        (seg.ready > 0).then(|| bar(seg.ready, Color::Blue)),
        (seg.waiting > 0).then(|| bar(seg.waiting, Color::Cyan)),
        (seg.suspended > 0).then(|| bar(seg.suspended, Color::Gray)),
        (seg.errors > 0).then(|| bar(seg.errors, Color::Red)),
        Some(Span::raw(" ")),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn runner_lines(
    centroid_values: &[u64],
    centroid_counts: &[u64],
    label_every_n: usize,
) -> Vec<Line<'static>> {
    let hist = histogram::render(centroid_counts);
    let labels = centroid_labels(centroid_values, label_every_n);

    vec![
        Line::from(
            once(Span::raw(" "))
                .chain(histogram_with_markers(hist, &labels, centroid_values))
                .collect::<Vec<_>>(),
        ),
        Line::from(vec![
            Span::raw(" "),
            Span::styled(
                axis_with_labels(&labels),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
    ]
}

fn centroid_labels(vals: &[u64], step: usize) -> Vec<(usize, String)> {
    vals.iter()
        .enumerate()
        .step_by(step)
        .map(|(idx, &val)| (idx, format_tick(val)))
        .collect()
}

struct CentroidStats {
    mean: f64,
    stddev: f64,
}

fn centroid_stats(vals: &[u64]) -> CentroidStats {
    if vals.is_empty() {
        return CentroidStats {
            mean: 0.0,
            stddev: 1.0,
        };
    }

    let mean = vals.iter().sum::<u64>() as f64 / vals.len() as f64;

    let variance = if vals.len() <= 1 {
        1.0
    } else {
        vals.iter()
            .map(|&v| {
                let diff = v as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / (vals.len() - 1) as f64
    };

    CentroidStats {
        mean,
        stddev: variance.sqrt().max(1.0),
    }
}

fn color_bar(
    idx: usize,
    ch: char,
    marked: &HashSet<usize>,
    vals: &[u64],
    stats: &CentroidStats,
) -> Span<'static> {
    let z = if idx < vals.len() {
        (vals[idx] as f64 - stats.mean) / stats.stddev
    } else {
        0.0
    };

    let labeled = marked.contains(&idx);
    let color = z_score_color(z, labeled);

    Span::styled(ch.to_string(), Style::default().fg(color))
}

fn z_score_color(z: f64, labeled: bool) -> Color {
    match z {
        z if z < -2.0 => {
            if labeled {
                Color::Magenta
            } else {
                Color::LightMagenta
            }
        }
        z if z < -1.0 => {
            if labeled {
                Color::Blue
            } else {
                Color::LightBlue
            }
        }
        z if z < 1.0 => {
            if labeled {
                Color::DarkGray
            } else {
                Color::Gray
            }
        }
        z if z < 2.0 => {
            if labeled {
                Color::Yellow
            } else {
                Color::LightYellow
            }
        }
        _ => {
            if labeled {
                Color::Red
            } else {
                Color::LightRed
            }
        }
    }
}

fn histogram_with_markers(
    hist: String,
    labels: &[(usize, String)],
    vals: &[u64],
) -> Vec<Span<'static>> {
    let marked: HashSet<usize> = labels.iter().map(|(i, _)| *i).collect();
    let stats = centroid_stats(vals);

    hist.chars()
        .enumerate()
        .map(|(i, ch)| color_bar(i, ch, &marked, vals, &stats))
        .collect()
}

fn axis_with_labels(labels: &[(usize, String)]) -> String {
    if labels.is_empty() {
        return String::new();
    }

    let max_pos = labels.last().map(|(i, _)| *i).unwrap_or(0);
    let width = max_pos + labels.last().map(|(_, s)| s.len()).unwrap_or(1);

    let mut buf = vec![' '; width];

    for (pos, label) in labels {
        for (offset, ch) in label.chars().enumerate() {
            if pos + offset < width {
                buf[pos + offset] = ch;
            }
        }
    }

    buf.into_iter().collect()
}

/* Section Rendering */

pub fn section(
    frame: &mut ratatui::Frame,
    area: Rect,
    title: &str,
    tasks: &[(TaskID, &TaskContextSnapshot)],
    weights: &HashMap<TaskID, String>,
    snapshot: &SchedulerSnapshot,
    spinner: usize,
    max_deps: usize,
    throughput: &HashMap<TaskID, f64>,
    show_time: bool,
    spin_char: impl Fn(usize) -> &'static str,
    icon: impl Fn(&TaskState) -> &'static str,
    badge: impl Fn(&TaskState) -> (&'static str, Color),
) {
    let avail = area.height.saturating_sub(3) as usize;
    let capacity = avail / LINES_PER_TASK;

    let (visible, overflow) = if tasks.len() <= capacity {
        (tasks, &[][..])
    } else {
        let cap = (avail - OVERFLOW_LINES) / LINES_PER_TASK;
        (&tasks[..cap], &tasks[cap..])
    };

    let mut lines = vec![Line::from("")];

    for (tid, ctx) in visible {
        let w = weights
            .get(tid)
            .map(|s| s.as_str());
        lines.extend(task(
            *tid, ctx, w, snapshot, spinner, max_deps, throughput, &spin_char,
            &icon, &badge,
        ));
    }

    if !overflow.is_empty() {
        lines.push(overflow_line(overflow, throughput, show_time));
    }

    let block = Block::default()
        .border_style(Style::default().fg(Color::White))
        .border_type(BorderType::Rounded)
        .title(format!(" {}: {} ", title, tasks.len()))
        .borders(Borders::ALL);

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

/* Task Rendering */

fn task<'a>(
    tid: TaskID,
    ctx: &'a TaskContextSnapshot,
    weight: Option<&'a str>,
    snapshot: &'a SchedulerSnapshot,
    spinner: usize,
    max_deps: usize,
    throughput: &'a HashMap<TaskID, f64>,
    spin_char: impl Fn(usize) -> &'static str,
    icon: impl Fn(&TaskState) -> &'static str,
    badge: impl Fn(&TaskState) -> (&'static str, Color),
) -> Vec<Line<'a>> {
    let mut lines = vec![task_line(
        ctx, weight, spinner, spin_char, icon, badge,
    )];

    if let Some(line) = progress_line(ctx, throughput.get(&tid).copied()) {
        lines.push(line);
    }

    if let Some(line) = deps_line(ctx, snapshot, max_deps) {
        lines.push(line);
    }

    lines.push(Line::from(""));
    lines
}

fn task_line<'a>(
    ctx: &'a TaskContextSnapshot,
    weight: Option<&'a str>,
    spinner: usize,
    spin_char: impl Fn(usize) -> &'static str,
    icon: impl Fn(&TaskState) -> &'static str,
    badge: impl Fn(&TaskState) -> (&'static str, Color),
) -> Line<'a> {
    let ic = match ctx.state {
        TaskState::Preempting | TaskState::Running => spin_char(spinner),
        _ => icon(&ctx.state),
    };

    let (badge_text, color) = badge(&ctx.state);
    let badge_fmt = match weight {
        Some(w) => format!("[{} @ {}]", badge_text, w),
        None => format!("[{}]", badge_text),
    };

    let desc = format!(
        "{}: {}",
        format_task_id(&ctx.category, ctx.component),
        &ctx.about
    );

    Line::from(vec![
        Span::raw(" "),
        Span::styled(ic, Style::default().fg(Color::White)),
        Span::raw(" "),
        Span::styled(
            badge_fmt,
            Style::default()
                .fg(color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(desc, Style::default().fg(Color::White)),
    ])
}

fn progress_line(
    ctx: &TaskContextSnapshot,
    rate: Option<f64>,
) -> Option<Line<'static>> {
    progress::render(ctx, rate)
}

fn deps_line<'a>(
    ctx: &'a TaskContextSnapshot,
    snapshot: &'a SchedulerSnapshot,
    max_deps: usize,
) -> Option<Line<'a>> {
    let TaskState::Waiting(deps) = &ctx.state else {
        return None;
    };

    let list: Vec<TaskID> = deps.iter().copied().collect();
    let text = format_deps(&list, snapshot, max_deps);

    if text.is_empty() {
        return None;
    }

    Some(Line::from(vec![
        Span::raw("    "),
        Span::styled(text, Style::default().fg(Color::Gray)),
    ]))
}

fn overflow_line(
    tasks: &[(TaskID, &TaskContextSnapshot)],
    throughput: &HashMap<TaskID, f64>,
    show_time: bool,
) -> Line<'static> {
    let count = tasks.len();

    let msg = if show_time {
        match progress::max_eta(tasks, throughput) {
            Some(eta) => format!(
                "    ...and {} more (max remaining: {}).",
                count,
                format_duration(eta)
            ),
            None => format!("    ...and {} more tasks.", count),
        }
    } else {
        format!("    ...and {} more tasks.", count)
    };

    Line::from(Span::styled(
        msg,
        Style::default().fg(Color::DarkGray),
    ))
}
