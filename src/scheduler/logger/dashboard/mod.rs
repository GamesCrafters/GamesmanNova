//! # TUI Logger Implementation
//!
//! Scheduler dashboard.

use std::collections::HashMap;
use std::io::Stdout;
use std::io::stdout;
use std::time::Duration;
use std::time::Instant;

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
use ratatui::layout::Rect;

use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::TaskID;

use super::super::traits::Logger;

mod collect;
mod format;
mod histogram;
mod layout;
mod progress;
mod render;
mod state;
mod style;

use collect::collect;
use collect::extract_weights;
use collect::sort;
use histogram::TickSketch;
use layout::HEADER_PADDING;
use layout::layout;
use layout::shows_time;
use render::header;
use render::section;
use state::TuiLoggerState;
use state::track_times;
use state::update_throughput;

/* SUBMODULES (declared above) */

/* ENUMERATIONS */

#[derive(Clone, Copy, Debug)]
pub enum SortOrder {
    StartTime,
    Progress,
    TaskID,
}

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
    #[builder(setter(each(name = "filter")))]
    pub filters: Vec<TaskFilter>,
    pub name: &'static str,

    #[builder(default = "1")]
    pub weight: usize,
}

/// TUI logger that displays beautiful real-time scheduler state.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct DashboardLogger {
    #[builder(default)]
    #[builder(setter(skip))]
    state: Option<TuiLoggerState>,

    #[builder(default = "default_sections()")]
    #[builder(setter(each(name = "section", into)))]
    sections: Vec<SectionConfig>,

    #[builder(default = "SortOrder::Progress")]
    task_sorting: SortOrder,

    #[builder(default = "5")]
    task_dependencies: usize,

    #[builder(default = "true")]
    policy_weights: bool,

    #[builder(default = "2500")]
    render_period: usize,

    #[builder(default = "50")]
    sample_period: usize,

    #[builder(default = "20")]
    label_period: usize,
}

/* Helper Structures */

struct RenderContext {
    throughput: HashMap<TaskID, f64>,
    times: HashMap<TaskID, Instant>,
    centroids: Vec<u64>,
    counts: Vec<u64>,
    elapsed: usize,
    sections: Vec<SectionConfig>,
    weights: HashMap<TaskID, String>,
}

/* IMPLEMENTATIONS */

impl Logger for DashboardLogger {
    fn report(&mut self, snapshot: &SchedulerSnapshot, _: bool) -> Result<()> {
        self.init()?;

        let state = self.state.as_mut().unwrap();
        state.observe_count += 1;
        let tick = state.observe_count;

        if tick.is_multiple_of(self.sample_period) {
            self.update(snapshot);
        }

        if tick.is_multiple_of(self.render_period) {
            self.exit_check()
                .context("Checking for user exit")?;

            self.draw(snapshot)
                .context("Rendering TUI frame")?;
        }

        Ok(())
    }
}

impl Drop for DashboardLogger {
    fn drop(&mut self) {
        if let Some(state) = &mut self.state {
            let _ = Self::restore(&mut state.terminal);
        }
    }
}

impl DashboardLogger {
    fn terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(terminal)
    }

    fn restore(term: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            term.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        term.show_cursor()?;
        Ok(())
    }

    fn exit_check(&self) -> Result<()> {
        if poll(Duration::from_millis(0))?
            && let Event::Key(key) = read()?
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            return Err(anyhow!("User requested exit"));
        }

        Ok(())
    }

    fn init(&mut self) -> Result<()> {
        if self.state.is_some() {
            return Ok(());
        }

        let terminal = Self::terminal().context("Preparing TUI terminal")?;
        let width = terminal
            .size()?
            .width
            .saturating_sub(HEADER_PADDING) as usize;

        self.state = Some(TuiLoggerState {
            task_start_times: Default::default(),
            task_throughput: Default::default(),
            task_progress: Default::default(),
            tick_sketch: TickSketch::new(width),
            start_time: Instant::now(),
            observe_count: 0,
            terminal,
        });

        Ok(())
    }

    fn update(&mut self, snapshot: &SchedulerSnapshot) {
        let state = self.state.as_mut().unwrap();

        if let Some(runner) = snapshot.runner.as_ref() {
            for &tick in &runner.ticks {
                state.tick_sketch.record(tick);
            }
        }

        update_throughput(state, snapshot);
        track_times(state, snapshot);
    }

    fn draw(&mut self, snapshot: &SchedulerSnapshot) -> Result<()> {
        let ctx = self.prepare_render_context(snapshot)?;
        self.render_frame(snapshot, &ctx)?;
        Ok(())
    }

    fn prepare_render_context(
        &mut self,
        snapshot: &SchedulerSnapshot,
    ) -> Result<RenderContext> {
        let state = self.state.as_mut().unwrap();
        let width = state
            .terminal
            .size()?
            .width
            .saturating_sub(HEADER_PADDING) as usize;
        state.tick_sketch.resize(width);

        Ok(RenderContext {
            throughput: state.task_throughput.clone(),
            times: state.task_start_times.clone(),
            centroids: state.tick_sketch.centroid_values(),
            counts: state.tick_sketch.centroid_counts(),
            elapsed: state
                .start_time
                .elapsed()
                .as_millis() as usize,
            sections: self.sections.clone(),
            weights: if self.policy_weights {
                extract_weights(snapshot.policy.as_ref())
            } else {
                HashMap::new()
            },
        })
    }

    fn render_frame(
        &mut self,
        snapshot: &SchedulerSnapshot,
        ctx: &RenderContext,
    ) -> Result<()> {
        let state = self.state.as_mut().unwrap();
        let label_period = self.label_period;
        let task_sorting = self.task_sorting;
        let task_dependencies = self.task_dependencies;

        state.terminal.draw(|frame| {
            let area = frame.area();
            let chunks = layout(area, &ctx.sections);

            render_header(frame, chunks[0], snapshot, ctx, label_period);
            render_sections(
                frame,
                &chunks[1..],
                snapshot,
                ctx,
                task_sorting,
                task_dependencies,
            );
        })?;

        Ok(())
    }
}

/* Helper Functions */

fn render_header(
    frame: &mut ratatui::Frame,
    area: Rect,
    snapshot: &SchedulerSnapshot,
    ctx: &RenderContext,
    label_period: usize,
) {
    header(
        frame,
        area,
        snapshot,
        snapshot.runner.as_ref(),
        &ctx.centroids,
        &ctx.counts,
        label_period,
    );
}

fn render_sections(
    frame: &mut ratatui::Frame,
    chunks: &[Rect],
    snapshot: &SchedulerSnapshot,
    ctx: &RenderContext,
    task_sorting: SortOrder,
    task_dependencies: usize,
) {
    for (i, sec) in ctx.sections.iter().enumerate() {
        let mut tasks = collect(snapshot, &sec.filters);
        sort(&mut tasks, task_sorting, &ctx.times);
        let show_time = shows_time(&sec.filters);

        section(
            frame,
            chunks[i],
            sec.name,
            &tasks,
            &ctx.weights,
            snapshot,
            ctx.elapsed / 100,
            task_dependencies,
            &ctx.throughput,
            show_time,
            style::spinner,
            style::icon,
            style::badge,
        );
    }
}

/* Helpers */

fn default_sections() -> Vec<SectionConfig> {
    vec![
        SectionConfig {
            filters: vec![TaskFilter::Running, TaskFilter::Preempting],
            name: "Workers",
            weight: 2,
        },
        SectionConfig {
            filters: vec![TaskFilter::Waiting, TaskFilter::Ready],
            name: "Buffered",
            weight: 2,
        },
        SectionConfig {
            filters: vec![TaskFilter::Suspended],
            name: "Suspended",
            weight: 2,
        },
        SectionConfig {
            filters: vec![TaskFilter::Error],
            name: "Errors",
            weight: 1,
        },
    ]
}
