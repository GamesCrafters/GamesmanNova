//! Tree-based layout system for dashboard components.
//!
//! Eliminates magic numbers by making all spacing and sizing explicit
//! through a declarative tree structure.

use std::collections::HashMap;
use std::time::Instant;

use ratatui::layout::Direction as RatatuiDirection;
use ratatui::widgets::BorderType;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use derive_builder::Builder;
use ratatui::Frame;

use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::RunnerSnapshot;
use crate::scheduler::TaskID;

use super::support::style;
use super::collect::collect;
use super::collect::sort;
use super::TaskFilter;
use super::SortOrder;
use super::components;
use super::render;

/* ENUMERATIONS */

#[derive(Clone, Debug)]
pub enum LayoutNode {
    Component(Component),
    Container(Container),
    Padded(Padded),
}

#[derive(Clone, Debug)]
pub enum ComponentKind {
    TaskList {
        filters: Vec<TaskFilter>,
        max_tasks: Option<usize>,
    },
    Histogram {
        label_every_n: usize,
    },
    BreakdownBar,
    Title,
    Stats,
}

#[derive(Clone, Debug)]
pub enum SizeRequest {
    FixedHeight {
        width: WidthRequest,
        height: u16,
    },
    FixedWidth {
        width: u16,
        height: HeightRequest,
    },
    Flexible {
        width: WidthRequest,
        height: HeightRequest,
    },
    Fixed {
        width: u16,
        height: u16,
    },
}

#[derive(Clone, Debug)]
pub enum WidthRequest {
    Intrinsic,
    Percent(u16),
    Fill,
    Min(u16),
    Max(u16),
}

#[derive(Clone, Debug)]
pub enum HeightRequest {
    Intrinsic,
    Percent(u16),
    Weight(usize),
    Fill,
    Min(u16),
    Max(u16),
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Vertical,
    Horizontal,
}

#[derive(Clone, Debug)]
pub enum LayoutResult {
    Leaf {
        area: Rect,
        content_area: Rect,
        component: ComponentKind,
        border: Option<Border>,
    },
    Branch {
        area: Rect,
        children: Vec<LayoutResult>,
    },
}

/* STRUCTURES */

#[derive(Clone, Debug, Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct Component {
    #[builder(default)]
    pub border: Option<Border>,

    pub kind: ComponentKind,

    #[builder(default = "SizeRequest::Flexible {
        width: WidthRequest::Fill,
        height: HeightRequest::Fill
    }")]
    pub size: SizeRequest,
}

#[derive(Clone, Debug, Builder)]
#[builder(pattern = "owned", setter(into), build_fn(name = "build_inner"))]
pub struct Container {
    #[builder(default, setter(each(name = "child", into)))]
    pub children: Vec<LayoutNode>,

    pub direction: Direction,

    #[builder(default = "Padding::zero()")]
    pub padding: Padding,
}

#[derive(Clone, Debug)]
pub struct Padded {
    pub child: Box<LayoutNode>,
    pub padding: Padding,
}

#[derive(Clone, Copy, Debug, Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct Padding {
    #[builder(default = "0")]
    pub bottom: u16,

    #[builder(default = "0")]
    pub right: u16,

    #[builder(default = "0")]
    pub left: u16,

    #[builder(default = "0")]
    pub top: u16,
}

#[derive(Clone, Debug, Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct Border {
    #[builder(default)]
    pub title: Option<&'static str>,

    #[builder(default = "BorderType::Rounded")]
    pub style: BorderType,
}

pub struct RequestContext<'a> {
    pub snapshot: &'a SchedulerSnapshot,
}

pub struct RenderContext<'a> {
    pub times: &'a HashMap<TaskID, Instant>,
    pub weights: &'a HashMap<TaskID, String>,
    pub runner: Option<&'a RunnerSnapshot>,
    pub throughput: &'a HashMap<TaskID, f64>,
    pub snapshot: &'a SchedulerSnapshot,
    pub task_sorting: SortOrder,
    pub task_dependencies: usize,
    pub centroids: &'a [u64],
    pub counts: &'a [u64],
    pub elapsed: usize,
}

pub struct LayoutContext<'a> {
    pub request_ctx: RequestContext<'a>,
    pub total_weight: usize,
}

/* IMPLEMENTATIONS */

impl From<Component> for LayoutNode {
    fn from(comp: Component) -> Self {
        LayoutNode::Component(comp)
    }
}

impl Component {
    pub fn builder() -> ComponentBuilder {
        ComponentBuilder::default()
    }
}

impl Container {
    pub fn builder() -> ContainerBuilder {
        ContainerBuilder::default()
    }
}

impl Padding {
    pub fn zero() -> Self {
        Self {
            bottom: 0,
            right: 0,
            left: 0,
            top: 0,
        }
    }

    pub fn uniform(n: u16) -> Self {
        Self {
            bottom: n,
            right: n,
            left: n,
            top: n,
        }
    }
}

impl ComponentKind {
    pub fn request_size(&self, ctx: &RequestContext) -> SizeRequest {
        match self {
            Self::Title => SizeRequest::FixedHeight {
                width: WidthRequest::Fill,
                height: 1,
            },
            Self::Stats => SizeRequest::FixedHeight {
                width: WidthRequest::Fill,
                height: 1,
            },
            Self::BreakdownBar => SizeRequest::FixedHeight {
                width: WidthRequest::Fill,
                height: 2,
            },
            Self::Histogram { .. } => SizeRequest::FixedHeight {
                width: WidthRequest::Fill,
                height: 2,
            },
            Self::TaskList { filters, max_tasks } => {
                let tasks = collect(ctx.snapshot, filters);
                let display_count = match max_tasks {
                    Some(max) => tasks.len().min(*max),
                    None => tasks.len(),
                };
                let mut height = tasks
                    .iter()
                    .take(display_count)
                    .map(|(_, ctx)| render::count_task_lines(ctx))
                    .sum::<usize>() as u16;
                if !tasks.is_empty() {
                    height += 1;
                }
                if display_count < tasks.len() {
                    height += 1;
                }
                SizeRequest::FixedHeight {
                    width: WidthRequest::Fill,
                    height,
                }
            },
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, ctx: &RenderContext) {
        match self {
            Self::Title => components::title::render(frame, area),
            Self::Stats => {
                components::stats::render(frame, area, ctx.snapshot, ctx.runner)
            },
            Self::BreakdownBar => {
                components::breakdown::render(frame, area, ctx.snapshot)
            },
            Self::Histogram { label_every_n } => render::render_histogram(
                frame,
                area,
                ctx.centroids,
                ctx.counts,
                *label_every_n,
            ),
            Self::TaskList { filters, max_tasks } => {
                let mut tasks = collect(ctx.snapshot, filters);
                sort(&mut tasks, ctx.task_sorting, &ctx.times);

                let params = render::TaskRenderParams {
                    throughput: ctx.throughput,
                    weights: &ctx.weights,
                    snapshot: ctx.snapshot,
                    max_deps: ctx.task_dependencies,
                    max_tasks: *max_tasks,
                    spinner: ctx.elapsed / 100,
                };

                render::section(
                    frame,
                    area,
                    &tasks,
                    &params,
                    style::spinner,
                    style::icon,
                    style::badge,
                );
            },
        }
    }
}

impl LayoutNode {
    pub fn request_size(&self, ctx: &RequestContext) -> SizeRequest {
        match self {
            Self::Component(comp) => {
                let mut req = comp.kind.request_size(ctx);

                if comp.border.is_some() {
                    req = Self::add_border_overhead(req);
                }

                req
            },

            Self::Padded(padded) => {
                let child_req = padded.child.request_size(ctx);
                Self::add_padding_overhead(child_req, &padded.padding)
            },

            Self::Container(container) => {
                let children_reqs: Vec<SizeRequest> = container
                    .children
                    .iter()
                    .map(|child| child.request_size(ctx))
                    .collect();

                Self::aggregate_requests(
                    &children_reqs,
                    container.direction,
                    &container.padding,
                )
            },
        }
    }

    pub fn layout(&self, area: Rect, ctx: &LayoutContext) -> LayoutResult {
        match self {
            Self::Component(comp) => Self::layout_component(comp, area),
            Self::Padded(padded) => Self::layout_padded(padded, area, ctx),
            Self::Container(container) => {
                Self::layout_container(container, area, ctx)
            },
        }
    }

    fn layout_component(comp: &Component, area: Rect) -> LayoutResult {
        let content_area = if comp.border.is_some() {
            Rect {
                x: area.x + 1,
                y: area.y + 1,
                width: area.width.saturating_sub(2),
                height: area.height.saturating_sub(2),
            }
        } else {
            area
        };

        LayoutResult::Leaf {
            area,
            content_area,
            component: comp.kind.clone(),
            border: comp.border.clone(),
        }
    }

    fn layout_padded(
        padded: &Padded,
        area: Rect,
        ctx: &LayoutContext,
    ) -> LayoutResult {
        let inner_area = Self::apply_padding(area, &padded.padding);
        padded
            .child
            .layout(inner_area, ctx)
    }

    fn layout_container(
        container: &Container,
        area: Rect,
        ctx: &LayoutContext,
    ) -> LayoutResult {
        let content_area = Self::apply_padding(area, &container.padding);

        let constraints = Self::build_constraints(&container.children, ctx);

        let ratatui_dir = match container.direction {
            Direction::Vertical => RatatuiDirection::Vertical,
            Direction::Horizontal => RatatuiDirection::Horizontal,
        };

        let chunks = Layout::default()
            .direction(ratatui_dir)
            .constraints(constraints)
            .split(content_area);

        let children_results: Vec<LayoutResult> = container
            .children
            .iter()
            .zip(chunks.iter())
            .map(|(child, &chunk)| child.layout(chunk, ctx))
            .collect();

        LayoutResult::Branch {
            area,
            children: children_results,
        }
    }

    fn apply_padding(area: Rect, padding: &Padding) -> Rect {
        Rect {
            x: area.x + padding.left,
            y: area.y + padding.top,
            width: area
                .width
                .saturating_sub(padding.left + padding.right),
            height: area
                .height
                .saturating_sub(padding.top + padding.bottom),
        }
    }

    fn build_constraints(
        children: &[LayoutNode],
        ctx: &LayoutContext,
    ) -> Vec<Constraint> {
        children
            .iter()
            .map(|child| {
                let req = child.request_size(&ctx.request_ctx);
                Self::constraint_from_request(&req, ctx)
            })
            .collect()
    }

    fn constraint_from_request(
        req: &SizeRequest,
        ctx: &LayoutContext,
    ) -> Constraint {
        match req {
            SizeRequest::FixedHeight { height, .. } => {
                Constraint::Length(*height)
            },
            SizeRequest::Fixed { height, .. } => Constraint::Length(*height),
            SizeRequest::FixedWidth { height, .. } => {
                Self::height_to_constraint(height, ctx)
            },
            SizeRequest::Flexible { height, .. } => {
                Self::height_to_constraint(height, ctx)
            },
        }
    }

    fn height_to_constraint(
        height_req: &HeightRequest,
        ctx: &LayoutContext,
    ) -> Constraint {
        match height_req {
            HeightRequest::Fill => Constraint::Min(0),
            HeightRequest::Min(h) => Constraint::Min(*h),
            HeightRequest::Max(h) => Constraint::Max(*h),
            HeightRequest::Percent(p) => Constraint::Percentage(*p),
            HeightRequest::Weight(w) => {
                if ctx.total_weight > 0 {
                    Constraint::Ratio(*w as u32, ctx.total_weight as u32)
                } else {
                    Constraint::Min(0)
                }
            },
            HeightRequest::Intrinsic => Constraint::Min(0),
        }
    }

    fn add_border_overhead(req: SizeRequest) -> SizeRequest {
        match req {
            SizeRequest::Fixed { width, height } => SizeRequest::Fixed {
                width: width.saturating_add(2),
                height: height.saturating_add(2),
            },
            SizeRequest::FixedWidth { width, height } => {
                SizeRequest::FixedWidth {
                    width: width.saturating_add(2),
                    height: Self::add_height_overhead(height, 2),
                }
            },
            SizeRequest::FixedHeight { width, height } => {
                SizeRequest::FixedHeight {
                    width: Self::add_width_overhead(width, 2),
                    height: height.saturating_add(2),
                }
            },
            SizeRequest::Flexible { width, height } => SizeRequest::Flexible {
                width: Self::add_width_overhead(width, 2),
                height: Self::add_height_overhead(height, 2),
            },
        }
    }

    fn add_padding_overhead(
        req: SizeRequest,
        padding: &Padding,
    ) -> SizeRequest {
        let h_overhead = padding.top + padding.bottom;
        let w_overhead = padding.left + padding.right;

        match req {
            SizeRequest::Fixed { width, height } => SizeRequest::Fixed {
                width: width.saturating_add(w_overhead),
                height: height.saturating_add(h_overhead),
            },
            SizeRequest::FixedWidth { width, height } => {
                SizeRequest::FixedWidth {
                    width: width.saturating_add(w_overhead),
                    height: Self::add_height_overhead(height, h_overhead),
                }
            },
            SizeRequest::FixedHeight { width, height } => {
                SizeRequest::FixedHeight {
                    width: Self::add_width_overhead(width, w_overhead),
                    height: height.saturating_add(h_overhead),
                }
            },
            SizeRequest::Flexible { width, height } => SizeRequest::Flexible {
                width: Self::add_width_overhead(width, w_overhead),
                height: Self::add_height_overhead(height, h_overhead),
            },
        }
    }

    fn add_width_overhead(req: WidthRequest, overhead: u16) -> WidthRequest {
        match req {
            WidthRequest::Min(w) => {
                WidthRequest::Min(w.saturating_add(overhead))
            },
            WidthRequest::Max(w) => {
                WidthRequest::Max(w.saturating_add(overhead))
            },
            other => other,
        }
    }

    fn add_height_overhead(req: HeightRequest, overhead: u16) -> HeightRequest {
        match req {
            HeightRequest::Min(h) => {
                HeightRequest::Min(h.saturating_add(overhead))
            },
            HeightRequest::Max(h) => {
                HeightRequest::Max(h.saturating_add(overhead))
            },
            other => other,
        }
    }

    fn aggregate_requests(
        children_reqs: &[SizeRequest],
        direction: Direction,
        padding: &Padding,
    ) -> SizeRequest {
        if children_reqs.is_empty() {
            return SizeRequest::Fixed {
                width: 0,
                height: 0,
            };
        }

        match direction {
            Direction::Vertical => {
                Self::aggregate_vertical(children_reqs, padding)
            },
            Direction::Horizontal => {
                Self::aggregate_horizontal(children_reqs, padding)
            },
        }
    }

    fn aggregate_vertical(
        children_reqs: &[SizeRequest],
        padding: &Padding,
    ) -> SizeRequest {
        let total_height: u16 = children_reqs
            .iter()
            .filter_map(|req| match req {
                SizeRequest::Fixed { height, .. } => Some(*height),
                SizeRequest::FixedHeight { height, .. } => Some(*height),
                _ => None,
            })
            .sum();

        let max_width = children_reqs
            .iter()
            .filter_map(|req| match req {
                SizeRequest::Fixed { width, .. } => Some(*width),
                SizeRequest::FixedWidth { width, .. } => Some(*width),
                _ => None,
            })
            .max()
            .unwrap_or(0);

        let h_overhead = padding.top + padding.bottom;
        let w_overhead = padding.left + padding.right;

        SizeRequest::Flexible {
            width: WidthRequest::Min(max_width.saturating_add(w_overhead)),
            height: HeightRequest::Min(total_height.saturating_add(h_overhead)),
        }
    }

    fn aggregate_horizontal(
        children_reqs: &[SizeRequest],
        padding: &Padding,
    ) -> SizeRequest {
        let total_width: u16 = children_reqs
            .iter()
            .filter_map(|req| match req {
                SizeRequest::Fixed { width, .. } => Some(*width),
                SizeRequest::FixedWidth { width, .. } => Some(*width),
                _ => None,
            })
            .sum();

        let max_height = children_reqs
            .iter()
            .filter_map(|req| match req {
                SizeRequest::Fixed { height, .. } => Some(*height),
                SizeRequest::FixedHeight { height, .. } => Some(*height),
                _ => None,
            })
            .max()
            .unwrap_or(0);

        let h_overhead = padding.top + padding.bottom;
        let w_overhead = padding.left + padding.right;

        SizeRequest::Flexible {
            width: WidthRequest::Min(total_width.saturating_add(w_overhead)),
            height: HeightRequest::Min(max_height.saturating_add(h_overhead)),
        }
    }
}

/* BUILDER IMPLEMENTATIONS */

impl ContainerBuilder {
    pub fn build(self) -> anyhow::Result<LayoutNode> {
        let container = self.build_inner()?;
        Ok(LayoutNode::Container(container))
    }
}
