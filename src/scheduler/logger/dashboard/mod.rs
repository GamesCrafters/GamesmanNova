//! # TUI Logger Implementation
//!
//! Scheduler dashboard.

use std::collections::HashMap;
use std::time::Instant;
use std::time::Duration;
use std::io::Stdout;
use std::io::stdout;

use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::event::KeyCode;
use crossterm::event::Event;
use crossterm::event::poll;
use crossterm::event::read;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Block;
use ratatui::style::Style;
use ratatui::Terminal;
use derive_builder::Builder;
use crossterm::execute;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;

use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::TaskID;

use super::super::traits::Logger;

/* SUBMODULES */

mod collect;
mod components;
mod render;
mod support;
mod tree;

use tree::ComponentKind;
use tree::SizeRequest;
use tree::WidthRequest;
use tree::Direction;
use tree::Container;
use tree::Component;
use tree::LayoutNode;
use tree::Padding;
use tree::Padded;
use tree::Border;

use components::histogram;
use support::state;

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

/// TUI logger that displays beautiful real-time scheduler state.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct DashboardLogger {
    #[builder(default)]
    #[builder(setter(skip))]
    state: Option<state::TuiLoggerState>,

    #[builder(default = "5")]
    task_dependencies: usize,

    #[builder(default = "SortOrder::Progress")]
    task_sorting: SortOrder,

    #[builder(default = "true")]
    policy_weights: bool,

    #[builder(default = "2500")]
    render_period: usize,

    #[builder(default = "50")]
    sample_period: usize,
}

/* HELPER STRUCTURES */

struct RenderContext {
    throughput: HashMap<TaskID, f64>,
    weights: HashMap<TaskID, String>,
    times: HashMap<TaskID, Instant>,
    centroids: Vec<u64>,
    counts: Vec<u64>,
    elapsed: usize,
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
        let width = terminal.size()?.width as usize;

        self.state = Some(state::TuiLoggerState {
            terminal,
            task_progress: Default::default(),
            task_start_times: Default::default(),
            task_throughput: Default::default(),
            tick_sketch: histogram::TickSketch::new(width),
            start_time: Instant::now(),
            observe_count: 0,
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

        state::update_throughput(state, snapshot);
        state::track_times(state, snapshot);
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
        let width = state.terminal.size()?.width as usize;
        state.tick_sketch.resize(width);

        Ok(RenderContext {
            times: state.task_start_times.clone(),
            weights: if self.policy_weights {
                collect::extract_weights(snapshot.policy.as_ref())
            } else {
                HashMap::new()
            },
            throughput: state.task_throughput.clone(),
            centroids: state.tick_sketch.centroid_values(),
            counts: state.tick_sketch.centroid_counts(),
            elapsed: state
                .start_time
                .elapsed()
                .as_millis() as usize,
        })
    }

    fn render_frame(
        &mut self,
        snapshot: &SchedulerSnapshot,
        ctx: &RenderContext,
    ) -> Result<()> {
        let state = self.state.as_mut().unwrap();
        let task_sorting = self.task_sorting;
        let task_dependencies = self.task_dependencies;
        state.terminal.draw(|frame| {
            let area = frame.area();
            let layout_tree = build_dashboard_layout();
            let request_ctx = tree::RequestContext { snapshot };
            let total_weight = compute_total_weight(&layout_tree, &request_ctx);
            let layout_ctx = tree::LayoutContext {
                request_ctx,
                total_weight,
            };

            let layout_result = layout_tree.layout(area, &layout_ctx);
            let render_ctx = tree::RenderContext {
                times: &ctx.times,
                weights: &ctx.weights,
                runner: snapshot.runner.as_ref(),
                throughput: &ctx.throughput,
                snapshot,
                task_sorting,
                task_dependencies,
                centroids: &ctx.centroids,
                counts: &ctx.counts,
                elapsed: ctx.elapsed,
            };

            render_tree(frame, &layout_result, &render_ctx);
        })?;

        Ok(())
    }
}

/* HELPER FUNCTIONS */

fn compute_total_weight(
    node: &tree::LayoutNode,
    ctx: &tree::RequestContext,
) -> usize {
    match node {
        tree::LayoutNode::Component(comp) => {
            let req = comp.kind.request_size(ctx);
            match req {
                tree::SizeRequest::Flexible { height, .. }
                | tree::SizeRequest::FixedWidth { height, .. } => {
                    if let tree::HeightRequest::Weight(w) = height {
                        w
                    } else {
                        0
                    }
                },
                _ => 0,
            }
        },
        tree::LayoutNode::Container(container) => container
            .children
            .iter()
            .map(|child| compute_total_weight(child, ctx))
            .sum(),
        tree::LayoutNode::Padded(padded) => {
            compute_total_weight(&padded.child, ctx)
        },
    }
}

fn render_tree(
    frame: &mut ratatui::Frame,
    result: &tree::LayoutResult,
    ctx: &tree::RenderContext,
) {
    match result {
        tree::LayoutResult::Leaf {
            area,
            content_area,
            component,
            border,
        } => {
            if let Some(border_info) = border {
                let block = Block::default()
                    .border_style(
                        Style::default().fg(ratatui::style::Color::White),
                    )
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .title(format!(" {} ", border_info.title.unwrap_or("")));
                frame.render_widget(block, *area);
            }

            component.render(frame, *content_area, ctx);
        },
        tree::LayoutResult::Branch { children, .. } => {
            for child in children {
                render_tree(frame, child, ctx);
            }
        },
    }
}

fn build_dashboard_layout() -> LayoutNode {
    Container::builder()
        .direction(Direction::Vertical)
        .child(
            Component::builder()
                .kind(ComponentKind::Title)
                .size(SizeRequest::FixedHeight {
                    width: WidthRequest::Fill,
                    height: 1,
                })
                .build()
                .unwrap(),
        )
        .child(LayoutNode::Padded(Padded {
            padding: Padding {
                top: 1,
                bottom: 0,
                left: 0,
                right: 0,
            },
            child: Box::new(LayoutNode::Component(
                Component::builder()
                    .kind(ComponentKind::Stats)
                    .size(SizeRequest::FixedHeight {
                        width: WidthRequest::Fill,
                        height: 2,
                    })
                    .build()
                    .unwrap(),
            )),
        }))
        .child(LayoutNode::Padded(Padded {
            padding: Padding {
                top: 1,
                bottom: 0,
                left: 0,
                right: 0,
            },
            child: Box::new(LayoutNode::Component(
                Component::builder()
                    .kind(ComponentKind::BreakdownBar)
                    .size(SizeRequest::FixedHeight {
                        width: WidthRequest::Fill,
                        height: 1,
                    })
                    .build()
                    .unwrap(),
            )),
        }))
        .child(LayoutNode::Padded(Padded {
            padding: Padding {
                top: 1,
                bottom: 0,
                left: 0,
                right: 0,
            },
            child: Box::new(LayoutNode::Component(
                Component::builder()
                    .kind(ComponentKind::Histogram { label_every_n: 20 })
                    .size(SizeRequest::FixedHeight {
                        width: WidthRequest::Fill,
                        height: 2,
                    })
                    .build()
                    .unwrap(),
            )),
        }))
        .child(
            Component::builder()
                .kind(ComponentKind::TaskList {
                    filters: vec![TaskFilter::Preempting, TaskFilter::Running],
                    max_tasks: Some(10),
                })
                .border(Some(Border {
                    title: Some("Owned by Workers"),
                    style: BorderType::Rounded,
                }))
                .build()
                .unwrap(),
        )
        .child(
            Component::builder()
                .kind(ComponentKind::TaskList {
                    filters: vec![TaskFilter::Waiting, TaskFilter::Ready],
                    max_tasks: Some(15),
                })
                .border(Some(Border {
                    title: Some("Waiting in Scheduler"),
                    style: BorderType::Rounded,
                }))
                .build()
                .unwrap(),
        )
        .child(
            Component::builder()
                .kind(ComponentKind::TaskList {
                    filters: vec![TaskFilter::Suspended, TaskFilter::Error],
                    max_tasks: Some(10),
                })
                .border(Some(Border {
                    title: Some("Making no Progress"),
                    style: BorderType::Rounded,
                }))
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
}
