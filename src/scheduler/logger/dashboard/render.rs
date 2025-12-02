//! Rendering functions for dashboard TUI.

use std::collections::HashMap;
use std::collections::HashSet;

use ratatui::widgets::Paragraph;
use ratatui::style::Modifier;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use derive_builder::Builder;

use crate::scheduler::TaskContextSnapshot;
use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::RunnerSnapshot;
use crate::scheduler::TaskState;
use crate::scheduler::TaskID;

use super::support::format::format_task_id;
use super::support::format::format_deps;
use super::support::format::format_tick;
use super::components::histogram;
use super::support::progress;

/* STRUCTURES */

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct TaskRenderParams<'a> {
    pub throughput: &'a HashMap<TaskID, f64>,
    pub weights: &'a HashMap<TaskID, String>,
    pub snapshot: &'a SchedulerSnapshot,
    pub max_deps: usize,

    #[builder(default)]
    pub max_tasks: Option<usize>,

    pub spinner: usize,
}

/* IMPLEMENTATIONS */

fn runner_lines(
    centroid_values: &[u64],
    centroid_counts: &[u64],
    label_every_n: usize,
    width: usize,
) -> Vec<Line<'static>> {
    let display_count = width.min(centroid_counts.len());
    let display_values = &centroid_values[..display_count];
    let hist = histogram::render(&centroid_counts[..display_count]);
    let labels = centroid_labels(display_values, label_every_n);

    vec![
        Line::from(histogram_with_markers(
            hist,
            &labels,
            display_values,
        )),
        Line::from(vec![Span::styled(
            axis_with_labels(&labels),
            Style::default().fg(Color::DarkGray),
        )]),
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

fn compute_variance(vals: &[u64], mean: f64) -> f64 {
    vals.iter()
        .map(|&v| {
            let diff = v as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / (vals.len() - 1) as f64
}

fn centroid_stats(vals: &[u64]) -> CentroidStats {
    if vals.is_empty() {
        return CentroidStats {
            mean: 0.0,
            stddev: 1.0,
        };
    }

    let mean = vals.iter().sum::<u64>() as f64 / vals.len() as f64;

    let variance =
        if vals.len() <= 1 { 1.0 } else { compute_variance(vals, mean) };

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
    let (dark, light) = match z {
        z if z < -2.0 => (Color::Magenta, Color::LightMagenta),
        z if z < -1.0 => (Color::Blue, Color::LightBlue),
        z if z < 1.0 => (Color::DarkGray, Color::Gray),
        z if z < 2.0 => (Color::Yellow, Color::LightYellow),
        _ => (Color::Red, Color::LightRed),
    };

    if labeled { dark } else { light }
}

fn histogram_with_markers(
    hist: String,
    labels: &[(usize, String)],
    vals: &[u64],
) -> Vec<Span<'static>> {
    let marked: HashSet<usize> = labels
        .iter()
        .map(|(i, _)| *i)
        .collect();
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

    let max_pos = labels
        .last()
        .map(|(i, _)| *i)
        .unwrap_or(0);
    let width = max_pos
        + labels
            .last()
            .map(|(_, s)| s.len())
            .unwrap_or(1);

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

/* SECTION RENDERING */

pub fn section(
    frame: &mut ratatui::Frame,
    area: Rect,
    tasks: &[(TaskID, &TaskContextSnapshot)],
    params: &TaskRenderParams,
    spin_char: impl Fn(usize) -> &'static str,
    icon: impl Fn(&TaskState) -> &'static str,
    badge: impl Fn(&TaskState) -> (&'static str, Color),
) {
    let mut lines = vec![];

    let display_count = match params.max_tasks {
        Some(max) => tasks.len().min(max),
        None => tasks.len(),
    };

    for (tid, ctx) in tasks.iter().take(display_count) {
        let w = params
            .weights
            .get(tid)
            .map(|s| s.as_str());
        lines.extend(task(
            *tid, ctx, w, params, &spin_char, &icon, &badge,
        ));
    }

    if display_count < tasks.len() {
        let remaining = tasks.len() - display_count;
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(
                format!("... +{} more", remaining),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    if !tasks.is_empty() {
        lines.push(Line::from(""));
    }

    let render_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: (lines.len() as u16).min(area.height),
    };

    frame.render_widget(Paragraph::new(lines), render_area);
}

/* TASK RENDERING */

pub fn count_task_lines(ctx: &TaskContextSnapshot) -> usize {
    let mut count = 0;

    count += 1;
    count += 1;

    if ctx.progress.is_some() {
        count += 1;
    }

    match &ctx.state {
        TaskState::Waiting(_) => count += 1,
        TaskState::Preempting
        | TaskState::Suspended(_)
        | TaskState::Running
        | TaskState::Error
        | TaskState::Ready => {},
    }

    count
}

fn task<'a>(
    tid: TaskID,
    ctx: &'a TaskContextSnapshot,
    weight: Option<&'a str>,
    params: &'a TaskRenderParams,
    spin_char: impl Fn(usize) -> &'static str,
    icon: impl Fn(&TaskState) -> &'static str,
    badge: impl Fn(&TaskState) -> (&'static str, Color),
) -> Vec<Line<'a>> {
    let mut lines = vec![
        Line::from(""),
        task_line(
            ctx,
            weight,
            params.spinner,
            spin_char,
            icon,
            badge,
        ),
    ];

    if let Some(line) = progress_line(
        ctx,
        params
            .throughput
            .get(&tid)
            .copied(),
    ) {
        lines.push(line);
    }

    if let Some(line) = deps_line(ctx, params.snapshot, params.max_deps) {
        lines.push(line);
    }

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

/* HISTOGRAM RENDERING */

pub fn render_histogram(
    frame: &mut ratatui::Frame,
    area: Rect,
    centroid_values: &[u64],
    centroid_counts: &[u64],
    label_every_n: usize,
) {
    let width = area.width as usize;
    let lines = runner_lines(
        centroid_values,
        centroid_counts,
        label_every_n,
        width,
    );

    frame.render_widget(Paragraph::new(lines), area);
}
