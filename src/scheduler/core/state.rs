//! # Scheduler State Management
//!
//! Central registry with type-safe task context storage.

use std::collections::HashMap;
use std::collections::HashSet;

use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::TaskContextSnapshot;
use crate::scheduler::TaskID;
use crate::scheduler::TaskState;
use crate::scheduler::core::context::AnyContext;
use crate::scheduler::traits::Executable;
use crate::scheduler::traits::Policy;
use crate::scheduler::traits::Runner;

/* TYPE ALIASES */

type Deps = HashSet<TaskID>;

/* STRUCTURES */

/// Deferred merge for offshore tasks.
pub(crate) struct MergeContext {
    pub(in crate::scheduler) executable: Box<dyn Executable>,
    pub(in crate::scheduler) _dependencies: Deps,
}

/// Scheduler state with task registry.
pub struct State {
    merges: HashMap<TaskID, MergeContext>,
    buffer: HashMap<TaskID, AnyContext>,
    ticks: u64,
}

/* IMPLEMENTATIONS */

impl State {
    pub fn new() -> Self {
        Self {
            buffer: HashMap::new(),
            merges: HashMap::new(),
            ticks: 0,
        }
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    pub fn increment(&mut self) {
        self.ticks += 1;
    }

    pub fn get(&self, id: &TaskID) -> Option<&AnyContext> {
        self.buffer.get(id)
    }

    pub fn get_mut(&mut self, id: &TaskID) -> Option<&mut AnyContext> {
        self.buffer.get_mut(id)
    }

    pub fn remove(&mut self, id: &TaskID) -> Option<AnyContext> {
        self.buffer.remove(id)
    }

    pub fn insert(&mut self, id: TaskID, ctx: AnyContext) {
        self.buffer.insert(id, ctx);
    }

    pub fn contains(&self, id: &TaskID) -> bool {
        self.buffer.contains_key(id)
    }

    pub fn active(&self) -> bool {
        self.buffer
            .values()
            .any(|ctx| ctx.is_active())
    }

    pub(in crate::scheduler) fn take_merge(
        &mut self,
        id: &TaskID,
    ) -> Option<MergeContext> {
        self.merges.remove(id)
    }

    pub(in crate::scheduler) fn defer_merge(
        &mut self,
        id: TaskID,
        merge: MergeContext,
    ) {
        self.merges.insert(id, merge);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&TaskID, &AnyContext)> {
        self.buffer.iter()
    }

    pub fn keys(&self) -> impl Iterator<Item = &TaskID> {
        self.buffer.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &AnyContext> {
        self.buffer.values()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn ready_ids(&self) -> impl Iterator<Item = &TaskID> {
        self.buffer
            .iter()
            .filter_map(|(id, ctx)| {
                matches!(ctx, AnyContext::Ready(_)).then_some(id)
            })
    }

    pub fn running_ids(&self) -> impl Iterator<Item = &TaskID> {
        self.buffer
            .iter()
            .filter_map(|(id, ctx)| {
                matches!(ctx, AnyContext::Running(_)).then_some(id)
            })
    }

    pub fn waiting_ids(&self) -> impl Iterator<Item = &TaskID> {
        self.buffer
            .iter()
            .filter_map(|(id, ctx)| {
                matches!(ctx, AnyContext::Waiting(_)).then_some(id)
            })
    }

    pub fn preempting_ids(&self) -> impl Iterator<Item = &TaskID> {
        self.buffer
            .iter()
            .filter_map(|(id, ctx)| {
                matches!(ctx, AnyContext::Preempting(_)).then_some(id)
            })
    }

    pub fn error_ids(&self) -> impl Iterator<Item = &TaskID> {
        self.buffer
            .iter()
            .filter_map(|(id, ctx)| {
                matches!(ctx, AnyContext::Error(_)).then_some(id)
            })
    }

    pub fn suspended_ids(&self) -> impl Iterator<Item = &TaskID> {
        self.buffer
            .iter()
            .filter_map(|(id, ctx)| {
                matches!(ctx, AnyContext::Suspended(_)).then_some(id)
            })
    }

    pub(in crate::scheduler) fn snapshot(
        &self,
        runner: &mut dyn Runner,
        _policy: &dyn Policy,
    ) -> SchedulerSnapshot {
        let tasks: HashMap<TaskID, TaskContextSnapshot> = self
            .buffer
            .iter()
            .map(|(id, ctx)| {
                let snapshot = Self::context_snapshot(id, ctx, runner);
                (*id, snapshot)
            })
            .collect();

        SchedulerSnapshot {
            tasks,
            tick: self.ticks,
            runner: runner.snapshot(),
            policy: None,
            transitions: Vec::new(),
        }
    }

    fn context_snapshot(
        id: &TaskID,
        ctx: &AnyContext,
        runner: &dyn Runner,
    ) -> TaskContextSnapshot {
        let state = Self::snapshot_state(ctx);
        let progress = Self::snapshot_progress(id, ctx, runner);
        let size = Self::snapshot_size(ctx);
        let about = Self::snapshot_about(ctx);

        TaskContextSnapshot {
            category: id.category,
            component: id.component,
            progress,
            about,
            state,
            size,
        }
    }

    fn snapshot_state(ctx: &AnyContext) -> TaskState {
        match ctx {
            AnyContext::Ready(_) => TaskState::Ready,
            AnyContext::Running(_) => TaskState::Running,
            AnyContext::Waiting(c) => {
                TaskState::Waiting(c.dependencies().clone())
            },
            AnyContext::Preempting(_) => TaskState::Preempting,
            AnyContext::Suspended(c) => TaskState::Suspended(*c.outcome()),
            AnyContext::Error(_) => TaskState::Error,
        }
    }

    fn snapshot_progress(
        id: &TaskID,
        ctx: &AnyContext,
        runner: &dyn Runner,
    ) -> Option<u64> {
        match ctx {
            AnyContext::Running(c) => runner
                .progress(*id)
                .or_else(|| c.progress()),
            AnyContext::Preempting(c) => runner
                .progress(*id)
                .or_else(|| c.progress()),
            AnyContext::Ready(c) => c.progress(),
            AnyContext::Waiting(c) => c.progress(),
            AnyContext::Suspended(c) => c.progress(),
            AnyContext::Error(c) => c.progress(),
        }
    }

    fn snapshot_size(ctx: &AnyContext) -> Option<u64> {
        match ctx {
            AnyContext::Ready(c) => c.size(),
            AnyContext::Running(c) => c.size(),
            AnyContext::Waiting(c) => c.size(),
            AnyContext::Preempting(c) => c.size(),
            AnyContext::Suspended(c) => c.size(),
            AnyContext::Error(c) => c.size(),
        }
    }

    fn snapshot_about(ctx: &AnyContext) -> String {
        match ctx {
            AnyContext::Ready(c) => c.about().to_string(),
            AnyContext::Running(c) => c.about().to_string(),
            AnyContext::Waiting(c) => c.about().to_string(),
            AnyContext::Preempting(c) => c.about().to_string(),
            AnyContext::Suspended(c) => c.about().to_string(),
            AnyContext::Error(c) => c.about().to_string(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl MergeContext {
    pub(in crate::scheduler) fn new(
        executable: Box<dyn Executable>,
        dependencies: Deps,
    ) -> Self {
        Self {
            _dependencies: dependencies,
            executable,
        }
    }
}
