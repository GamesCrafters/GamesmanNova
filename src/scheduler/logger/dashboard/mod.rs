//! # TUI Logger Implementation
//!
//! Scheduler dashboard.

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::poll;
use crossterm::event::read;
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use derive_builder::Builder;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use std::io::Stdout;
use std::time::Duration;
use std::time::Instant;

use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::TaskState;
use crate::scheduler::logger::dashboard::layout::layout;
use crate::scheduler::logger::dashboard::layout::section_shows_time;
use crate::scheduler::logger::dashboard::render::collect_tasks;
use crate::scheduler::logger::dashboard::render::render_header;
use crate::scheduler::logger::dashboard::render::render_section;
use crate::scheduler::logger::dashboard::render::sort_tasks;
use crate::scheduler::logger::dashboard::state::TuiLoggerState;
use crate::scheduler::logger::dashboard::state::track_times;
use crate::scheduler::logger::dashboard::state::update_throughput;
use crate::scheduler::traits::Logger;

/* SUBMODULES */

mod format;
mod layout;
mod render;
mod state;

/* CONSTANTS */

const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

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

/// TUI logger that displays beautiful real-time scheduler state.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct DashboardLogger {
    /// Sections to display with their filters.
    #[builder(default = "default_sections()")]
    #[builder(setter(each(name = "section", into)))]
    sections: Vec<SectionConfig>,

    /// Sort order within sections.
    #[builder(default = "SortOrder::Progress")]
    order: SortOrder,

    /// Redraw every N observe calls.
    #[builder(default = "100")]
    frequency: usize,

    /// Max dependencies to display before abbreviation.
    #[builder(default = "5")]
    dependencies: usize,

    /// Internal TUI state
    #[builder(default)]
    #[builder(setter(skip))]
    state: Option<TuiLoggerState>,
}

/* IMPLEMENTATIONS */

impl Logger for DashboardLogger {
    fn report(
        &mut self,
        snapshot: &SchedulerSnapshot,
        _changed: bool,
    ) -> Result<()> {
        self.ensure_init()?;
        self.check_exit_request()
            .context("Failure while checking for manual user exit")?;

        let should_update = {
            let state = self.state.as_mut().unwrap();
            state.observe_count += 1;
            state
                .observe_count
                .is_multiple_of(self.frequency)
        };

        if !should_update {
            return Ok(());
        }

        self.update_state(snapshot);
        self.render(snapshot)
            .context("Failure while rendering TUI frame")?;

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

    /// Get icon for task state.
    fn task_icon(state: &TaskState) -> &'static str {
        match state {
            TaskState::Suspended(_) => "✓",
            TaskState::Preempting => Self::spinner_char(0),
            TaskState::Waiting(_) => "⏸",
            TaskState::Running => Self::spinner_char(0),
            TaskState::Ready => "◯",
            TaskState::Error => "✗",
        }
    }

    /// Get badge text and color for task state.
    fn task_badge(state: &TaskState) -> (&'static str, ratatui::style::Color) {
        use ratatui::style::Color;
        match state {
            TaskState::Suspended(_) => ("SUSPENDED", Color::Gray),
            TaskState::Preempting => ("PREEMPT", Color::Yellow),
            TaskState::Waiting(_) => ("WAITING", Color::Cyan),
            TaskState::Running => ("RUNNING", Color::Green),
            TaskState::Ready => ("READY", Color::Blue),
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

    /// Check for user exit request via keyboard.
    fn check_exit_request(&self) -> Result<()> {
        if poll(Duration::from_millis(0))?
            && let Event::Key(key) = read()?
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            return Err(anyhow!("User requested exit"));
        }

        Ok(())
    }

    /// Initialize terminal state if needed.
    fn ensure_init(&mut self) -> Result<()> {
        if self.state.is_some() {
            return Ok(());
        }

        let terminal = Self::init_terminal()
            .context("Failed to prepare terminal for TUI reporting")?;

        self.state = Some(TuiLoggerState {
            start_time: Instant::now(),
            task_start_times: Default::default(),
            task_throughput: Default::default(),
            task_progress: Default::default(),
            observe_count: 0,
            terminal,
        });

        Ok(())
    }

    /// Update internal state tracking.
    fn update_state(&mut self, snapshot: &SchedulerSnapshot) {
        let state = self.state.as_mut().unwrap();
        track_times(state, snapshot);
        update_throughput(state, snapshot);
    }

    /// Render the TUI display.
    fn render(&mut self, snapshot: &SchedulerSnapshot) -> Result<()> {
        let state = self.state.as_mut().unwrap();
        let throughput = state.task_throughput.clone();
        let sections = self.sections.clone();
        let max_deps = self.dependencies;
        let elapsed = state
            .start_time
            .elapsed()
            .as_millis() as usize;

        let spinner = elapsed / 100;
        let times = state.task_start_times.clone();
        let order = self.order;

        state.terminal.draw(|frame| {
            let area = frame.area();
            let chunks = layout(area, &sections);
            render_header(frame, chunks[0], snapshot, &throughput);
            for (i, section) in sections.iter().enumerate() {
                let mut tasks = collect_tasks(
                    snapshot,
                    &section.filters,
                    Self::matches_filter,
                );
                sort_tasks(&mut tasks, order, &times);
                let show_time = section_shows_time(&section.filters);
                render_section(
                    frame,
                    chunks[i + 1],
                    section.name,
                    &tasks,
                    snapshot,
                    spinner,
                    max_deps,
                    &throughput,
                    show_time,
                    Self::spinner_char,
                    Self::task_icon,
                    Self::task_badge,
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
            weight: 1,
        },
        SectionConfig {
            name: "Waiting",
            filters: vec![TaskFilter::Waiting],
            weight: 1,
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
