//! # TUI Logger Implementation
//!
//! Scheduler dashboard.

use anyhow::Result;
use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use derive_builder::Builder;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;

use std::collections::HashMap;
use std::io::Stdout;
use std::time::Instant;

use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::TaskContextSnapshot;
use crate::scheduler::TaskID;
use crate::scheduler::TaskState;
use crate::scheduler::traits::Logger;

/* CONSTANTS */

// UI elements (spinners and progress bar width).
const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const WIDTH: usize = 30;

// EMA parameters for throughput calculation.
const MIN_INTERVAL: f64 = 1.0;
const ALPHA: f64 = 0.3;

/* ENUMERATIONS */

#[derive(Clone, Copy, Debug)]
pub enum SortOrder {
    /// Sort by progress percentage (closest to completion first).
    Progress,

    /// Sort by TaskID (stable ordering).
    TaskID,

    /// Sort by start time (oldest first).
    StartTime,
}

/// Task state filter for section configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskFilter {
    Preempting,
    Suspended,
    Waiting,
    Running,
    Ready,
    Error,
}

/* STRUCTURES */

/// Configuration for a single TUI section.
#[derive(Builder, Clone)]
#[builder(pattern = "owned", setter(into))]
pub struct SectionConfig {
    /// Section display name.
    pub name: &'static str,

    /// Task state filters for this section.
    #[builder(setter(each(name = "filter", into)))]
    pub filters: Vec<TaskFilter>,

    /// Relative size weight (proportional allocation).
    #[builder(default = "1")]
    pub weight: usize,
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned", setter(into))]
pub struct TuiLoggerConfig {
    /// Sections to display with their filters.
    #[builder(default = "default_sections()")]
    #[builder(setter(each(name = "section", into)))]
    pub sections: Vec<SectionConfig>,

    /// Sort order within sections.
    #[builder(default = "SortOrder::Progress")]
    pub order: SortOrder,

    /// Redraw every N observe calls.
    #[builder(default = "1")]
    pub frequency: usize,

    /// Max dependencies to display before abbreviation.
    #[builder(default = "5")]
    pub dependencies: usize,
}

/// Internal state for TUI logger.
struct TuiLoggerState {
    /// Current spinner animation frame.
    spinner_frame: usize,

    /// When each task started running (for duration tracking).
    task_start_times: HashMap<TaskID, Instant>,

    /// Last progress sample for each task (timestamp, progress).
    task_progress: HashMap<TaskID, (Instant, u64)>,

    /// Calculated throughput for each task (ops per second).
    task_throughput: HashMap<TaskID, f64>,

    /// Terminal interface.
    terminal: Terminal<CrosstermBackend<Stdout>>,

    /// Number of observe calls (for update frequency).
    observe_count: usize,
}

/// State count breakdown by task state.
struct StateCounts {
    running: usize,
    ready: usize,
    waiting: usize,
    suspended: usize,
    errors: usize,
}

/// Bar segment sizes for state visualization.
struct BarSegments {
    running: usize,
    ready: usize,
    waiting: usize,
    suspended: usize,
    errors: usize,
}

/// TUI logger that displays beautiful real-time scheduler state.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct DashboardLogger {
    config: TuiLoggerConfig,

    #[builder(default)]
    #[builder(setter(skip))]
    state: Option<TuiLoggerState>,
}

/* IMPLEMENTATIONS */

impl Logger for DashboardLogger {
    fn observe(
        &mut self,
        snapshot: &SchedulerSnapshot,
        _changed: bool,
    ) -> Result<()> {
        self.ensure_init()?;
        let should_update = {
            let state = self.state.as_mut().unwrap();
            state.observe_count += 1;
            state
                .observe_count
                .is_multiple_of(self.config.frequency)
        };

        if !should_update {
            return Ok(());
        }

        self.update_state(snapshot);
        self.render(snapshot)?;
        Ok(())
    }
}

impl Drop for DashboardLogger {
    fn drop(&mut self) {
        if let Some(state) = &mut self.state {
            let _ = Self::restore_terminal(&mut state.terminal);
        }
    }
}

impl DashboardLogger {
    /// Initialize the terminal for TUI mode.
    fn init_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(terminal)
    }

    /// Restore the terminal to normal mode.
    fn restore_terminal(
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;

        terminal.show_cursor()?;
        Ok(())
    }

    /// Get spinner character for current frame.
    fn spinner_char(frame: usize) -> &'static str {
        SPINNER[frame % SPINNER.len()]
    }

    /// Get icon and color for task state.
    fn task_icon(state: &TaskState) -> (&'static str, Color) {
        match state {
            TaskState::Suspended(_) => ("✓", Color::Blue),
            TaskState::Preempting => (Self::spinner_char(0), Color::Magenta),
            TaskState::Waiting(_) => ("⏸", Color::Yellow),
            TaskState::Running => (Self::spinner_char(0), Color::Green),
            TaskState::Ready => ("◯", Color::Yellow),
            TaskState::Error => ("✗", Color::Red),
        }
    }

    /// Get badge text and color for task state.
    fn task_badge(state: &TaskState) -> (&'static str, Color) {
        match state {
            TaskState::Suspended(_) => ("SUSPENDED", Color::Blue),
            TaskState::Preempting => ("PREEMPT", Color::Magenta),
            TaskState::Waiting(_) => ("WAITING", Color::Yellow),
            TaskState::Running => ("RUNNING", Color::Green),
            TaskState::Ready => ("READY", Color::Yellow),
            TaskState::Error => ("ERROR", Color::Red),
        }
    }

    /// Check if task matches any filter in the list.
    fn matches_filter(state: &TaskState, filters: &[TaskFilter]) -> bool {
        filters.iter().any(|f| {
            matches!(
                (f, state),
                (TaskFilter::Ready, TaskState::Ready)
                    | (TaskFilter::Running, TaskState::Running)
                    | (TaskFilter::Preempting, TaskState::Preempting)
                    | (TaskFilter::Waiting, TaskState::Waiting(_))
                    | (TaskFilter::Suspended, TaskState::Suspended(_))
                    | (TaskFilter::Error, TaskState::Error)
            )
        })
    }

    /// Initialize terminal state if needed.
    fn ensure_init(&mut self) -> Result<()> {
        if self.state.is_some() {
            return Ok(());
        }

        let terminal = Self::init_terminal()?;
        self.state = Some(TuiLoggerState {
            task_start_times: HashMap::new(),
            task_throughput: HashMap::new(),
            task_progress: HashMap::new(),
            observe_count: 0,
            spinner_frame: 0,
            terminal,
        });

        Ok(())
    }

    /// Check if we should update the display.
    fn should_update(&self, state: &TuiLoggerState) -> bool {
        state
            .observe_count
            .is_multiple_of(self.config.frequency)
    }

    /// Update internal state tracking.
    fn update_state(&mut self, snapshot: &SchedulerSnapshot) {
        let state = self.state.as_mut().unwrap();
        state.spinner_frame += 1;
        self.track_times(snapshot);
        self.update_throughput(snapshot);
    }

    /// Track task start times.
    fn track_times(&mut self, snapshot: &SchedulerSnapshot) {
        let state = self.state.as_mut().unwrap();
        let active = |(_, ctx): &(&TaskID, &TaskContextSnapshot)| {
            matches!(
                ctx.state,
                TaskState::Running | TaskState::Preempting
            )
        };

        let tasks = snapshot
            .tasks
            .iter()
            .filter(active);
        for (tid, _) in tasks {
            state
                .task_start_times
                .entry(*tid)
                .or_insert_with(Instant::now);
        }
    }

    /// Update per-task throughput calculation using exponential moving average.
    fn update_throughput(&mut self, snapshot: &SchedulerSnapshot) {
        let state = self.state.as_mut().unwrap();
        let now = Instant::now();
        for (tid, ctx) in &snapshot.tasks {
            let Some(current) = ctx.progress else {
                continue;
            };

            let Some((last_time, last_progress)) = state.task_progress.get(tid)
            else {
                state
                    .task_progress
                    .insert(*tid, (now, current));
                continue;
            };

            let elapsed = now
                .duration_since(*last_time)
                .as_secs_f64();
            if elapsed < MIN_INTERVAL {
                continue;
            }

            let delta = current.saturating_sub(*last_progress);
            let instantaneous = delta as f64 / elapsed;
            let smoothed =
                if let Some(&previous) = state.task_throughput.get(tid) {
                    ALPHA * instantaneous + (1.0 - ALPHA) * previous
                } else {
                    instantaneous
                };

            state
                .task_throughput
                .insert(*tid, smoothed);

            state
                .task_progress
                .insert(*tid, (now, current));
        }
    }

    /// Render the TUI display.
    fn render(&mut self, snapshot: &SchedulerSnapshot) -> Result<()> {
        let state = self.state.as_mut().unwrap();
        let throughput = state.task_throughput.clone();
        let sections = self.config.sections.clone();
        let max_deps = self.config.dependencies;
        let spinner = state.spinner_frame;
        let times = state.task_start_times.clone();
        let order = self.config.order;

        state.terminal.draw(|frame| {
            let area = frame.area();
            let chunks = layout(area, &sections);
            render_header(frame, chunks[0], snapshot, &throughput);
            for (i, section) in sections.iter().enumerate() {
                let mut tasks = collect_tasks(snapshot, &section.filters);
                sort_tasks(&mut tasks, order, &times);
                let show_time = section_shows_time(&section.filters);
                render_section(
                    frame,
                    chunks[i + 1],
                    section.name,
                    &tasks,
                    spinner,
                    max_deps,
                    &throughput,
                    show_time,
                );
            }
        })?;

        Ok(())
    }
}

/* CONFIGURATION HELPERS */

fn default_sections() -> Vec<SectionConfig> {
    vec![
        SectionConfig {
            name: "Active",
            filters: vec![
                TaskFilter::Running,
                TaskFilter::Preempting,
                TaskFilter::Ready,
            ],
            weight: 3,
        },
        SectionConfig {
            name: "Waiting",
            filters: vec![TaskFilter::Waiting],
            weight: 2,
        },
        SectionConfig {
            name: "Suspended",
            filters: vec![TaskFilter::Suspended],
            weight: 1,
        },
        SectionConfig {
            name: "Errors",
            filters: vec![TaskFilter::Error],
            weight: 1,
        },
    ]
}

/* STATE MANAGEMENT */

fn count_states(snapshot: &SchedulerSnapshot) -> StateCounts {
    let mut counts = StateCounts {
        running: 0,
        ready: 0,
        waiting: 0,
        suspended: 0,
        errors: 0,
    };

    for ctx in snapshot.tasks.values() {
        match ctx.state {
            TaskState::Running | TaskState::Preempting => counts.running += 1,
            TaskState::Ready => counts.ready += 1,
            TaskState::Waiting(_) => counts.waiting += 1,
            TaskState::Suspended(_) => counts.suspended += 1,
            TaskState::Error => counts.errors += 1,
        }
    }

    counts
}

/* TASK OPERATIONS */

fn collect_tasks<'a>(
    snapshot: &'a SchedulerSnapshot,
    filters: &[TaskFilter],
) -> Vec<(TaskID, &'a TaskContextSnapshot)> {
    let matches = |(_, ctx): &(&TaskID, &TaskContextSnapshot)| {
        DashboardLogger::matches_filter(&ctx.state, filters)
    };

    snapshot
        .tasks
        .iter()
        .filter(matches)
        .map(|(tid, ctx)| (*tid, ctx))
        .collect()
}

fn sort_tasks(
    tasks: &mut [(TaskID, &TaskContextSnapshot)],
    sort_order: SortOrder,
    task_start_times: &HashMap<TaskID, Instant>,
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
            });
        },
        SortOrder::TaskID => {
            tasks.sort_by_key(|(tid, _)| *tid);
        },
        SortOrder::StartTime => {
            let time = |tid: &TaskID| task_start_times.get(tid);
            tasks.sort_by(|a, b| time(&a.0).cmp(&time(&b.0)));
        },
    }
}

fn section_shows_time(filters: &[TaskFilter]) -> bool {
    filters.iter().any(|f| {
        matches!(
            f,
            TaskFilter::Running | TaskFilter::Preempting | TaskFilter::Ready
        )
    })
}

/* LAYOUT */

fn layout(area: Rect, sections: &[SectionConfig]) -> std::rc::Rc<[Rect]> {
    let total: usize = sections
        .iter()
        .map(|s| s.weight)
        .sum();
    let available = area.height.saturating_sub(4) as usize;

    let mut constraints = vec![Constraint::Length(3)];
    for section in sections {
        let proportion =
            (section.weight as f64 / total as f64 * available as f64) as u16;
        let height = proportion.max(5);
        constraints.push(Constraint::Length(height));
    }
    constraints.push(Constraint::Length(1));

    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area)
}

/* RENDERING - TOP LEVEL */

fn render_header(
    frame: &mut ratatui::Frame,
    area: Rect,
    snapshot: &SchedulerSnapshot,
    _throughput: &HashMap<TaskID, f64>,
) {
    let counts = count_states(snapshot);
    let bar_line =
        render_state_bar(&counts, area.width.saturating_sub(2) as usize);

    let lines = vec![
        Line::from(Span::styled(
            format!(
                " Nova Scheduler │ Tick: {} │ Total: {} tasks ",
                snapshot.tick,
                snapshot.tasks.len()
            ),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Running: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{} ", counts.running)),
            Span::styled("• Ready: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{} ", counts.ready)),
            Span::styled("• Waiting: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{} ", counts.waiting)),
            Span::styled("• Suspended: ", Style::default().fg(Color::Blue)),
            Span::raw(format!("{} ", counts.suspended)),
            Span::styled("• Errors: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}", counts.errors)),
        ]),
        bar_line,
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::White)),
    );

    frame.render_widget(paragraph, area);
}

fn render_section(
    frame: &mut ratatui::Frame,
    area: Rect,
    title: &str,
    tasks: &[(TaskID, &TaskContextSnapshot)],
    spinner: usize,
    max_deps: usize,
    throughput: &HashMap<TaskID, f64>,
    show_time: bool,
) {
    let available = area.height.saturating_sub(2) as usize;
    let per_task = 4;
    let capacity = available / per_task;

    let (visible, overflow) = if tasks.len() <= capacity {
        (tasks, &[][..])
    } else {
        let split = capacity.saturating_sub(1);
        (&tasks[..split], &tasks[split..])
    };

    let mut lines = Vec::new();
    for (tid, ctx) in visible {
        lines.extend(render_task(
            *tid, ctx, spinner, max_deps, throughput,
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
            .border_style(Style::default().fg(Color::White)),
    );

    frame.render_widget(paragraph, area);
}

/* RENDERING - TASK DETAILS */

fn render_task<'a>(
    tid: TaskID,
    ctx: &'a TaskContextSnapshot,
    spinner: usize,
    max_deps: usize,
    throughput: &'a HashMap<TaskID, f64>,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    lines.push(task_header(tid, ctx, spinner));
    if let Some(line) = render_progress(ctx, throughput.get(&tid).copied()) {
        lines.push(line);
    }

    if let Some(line) = render_deps(ctx, max_deps) {
        lines.push(line);
    }

    lines.push(Line::from(""));
    lines
}

fn task_header(
    tid: TaskID,
    ctx: &TaskContextSnapshot,
    spinner: usize,
) -> Line<'_> {
    let (mut icon, color) = DashboardLogger::task_icon(&ctx.state);
    if matches!(
        ctx.state,
        TaskState::Running | TaskState::Preempting
    ) {
        icon = DashboardLogger::spinner_char(spinner);
    }

    let (badge, badge_color) = DashboardLogger::task_badge(&ctx.state);
    Line::from(vec![
        Span::raw(" "),
        Span::styled(icon, Style::default().fg(color)),
        Span::raw(" "),
        Span::styled(
            format!("[{}]", badge),
            Style::default()
                .fg(badge_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{}: {}", tid, &ctx.about),
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

fn render_deps(ctx: &TaskContextSnapshot, max_deps: usize) -> Option<Line<'_>> {
    if let TaskState::Waiting(deps) = &ctx.state {
        let list: Vec<TaskID> = deps.iter().copied().collect();
        let text = format_deps(&list, max_deps);
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

/* RENDERING - COMPONENTS */

fn calculate_bar_segments(counts: &StateCounts, width: usize) -> BarSegments {
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

    let ready =
        ((counts.ready as f64 / total as f64) * width as f64).round() as usize;

    let waiting = ((counts.waiting as f64 / total as f64) * width as f64)
        .round() as usize;

    let suspended = ((counts.suspended as f64 / total as f64) * width as f64)
        .round() as usize;

    let errors =
        ((counts.errors as f64 / total as f64) * width as f64).round() as usize;

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

fn build_bar_spans(segments: &BarSegments) -> Vec<Span<'static>> {
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
            Style::default().fg(Color::Yellow),
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
            Style::default().fg(Color::Blue),
        ));
    }

    if segments.errors > 0 {
        spans.push(Span::styled(
            "━".repeat(segments.errors),
            Style::default().fg(Color::Red),
        ));
    }

    spans
}

fn render_state_bar(counts: &StateCounts, width: usize) -> Line<'static> {
    if width == 0 {
        return Line::from(" ");
    }

    let segments = calculate_bar_segments(counts, width);
    let spans = build_bar_spans(&segments);
    Line::from(spans)
}

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
        Span::styled(bar, Style::default().fg(Color::Green)),
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
            Style::default().fg(Color::Cyan),
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
        Style::default().fg(Color::Magenta),
    ));
}

fn add_rate(spans: &mut Vec<Span<'static>>, rate: f64) {
    spans.push(Span::raw(" │ "));
    spans.push(Span::styled(
        format!("{:.1} ops/s", rate),
        Style::default().fg(Color::Yellow),
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

/* FORMATTING */

fn format_task_ids(ids: &[TaskID]) -> String {
    ids.iter()
        .map(|id| format!("#{}", id))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_deps(deps: &[TaskID], max_deps_display: usize) -> String {
    if deps.is_empty() {
        return String::new();
    }

    if deps.len() <= max_deps_display {
        format!("Waits on: {}", format_task_ids(deps))
    } else {
        let shown = &deps[..max_deps_display];
        let remaining = deps.len() - max_deps_display;
        format!(
            "Waits on: {}, ... (+{} more)",
            format_task_ids(shown),
            remaining
        )
    }
}

fn format_duration(seconds: f64) -> String {
    if seconds < 60.0 {
        format!("{:.0}s", seconds)
    } else if seconds < 3600.0 {
        let mins = (seconds / 60.0) as u64;
        let secs = (seconds % 60.0) as u64;
        if secs > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}m", mins)
        }
    } else if seconds < 86400.0 {
        let hours = (seconds / 3600.0) as u64;
        let mins = ((seconds % 3600.0) / 60.0) as u64;
        if mins > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}h", hours)
        }
    } else {
        let days = (seconds / 86400.0) as u64;
        let hours = ((seconds % 86400.0) / 3600.0) as u64;
        if hours > 0 {
            format!("{}d {}h", days, hours)
        } else {
            format!("{}d", days)
        }
    }
}
