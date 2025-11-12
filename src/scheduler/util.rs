//! # Scheduler Utilities
//!
//! Helper functions for SchedulerState queries and task context
//! management.
//!
//! ## Note on DecisionContext
//!
//! The iterator methods here (tasks_ready, tasks_running, etc.)
//! are for SCHEDULER INTERNALS, not policy decisions. Policies
//! receive DecisionContext with pre-filtered candidates.

use anyhow::Context;
use anyhow::Result;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::scheduler::DecisionContext;
use crate::scheduler::Dependencies;
use crate::scheduler::Logger;
use crate::scheduler::MergeContext;
use crate::scheduler::Policy;
use crate::scheduler::Runner;
use crate::scheduler::SchedulerContextBuilder;
use crate::scheduler::SchedulerState;
use crate::scheduler::SchedulerStateBuilder;
use crate::scheduler::Task;
use crate::scheduler::TaskContext;
use crate::scheduler::TaskID;
use crate::scheduler::TaskOutcome;
use crate::scheduler::TaskRegistry;
use crate::scheduler::TaskState;

/* BUILDER PATTERN */

impl SchedulerStateBuilder {
    pub fn task(mut self, task: Task) -> Self {
        let state = if task.requires.is_empty() {
            TaskState::Ready
        } else {
            TaskState::Waiting(task.requires.clone())
        };

        let buffer = self
            .buffer
            .get_or_insert_with(TaskRegistry::new);

        let ctx = TaskContext {
            executable: Some(task.executable),
            retriable: task.retriable,
            incoming: task.requires.clone(),
            progress: None,
            about: task.about,
            size: task.size,
            state,
        };

        buffer.insert(task.tid, ctx);
        self
    }
}

#[allow(private_bounds)]
impl SchedulerContextBuilder {
    pub fn policy(mut self, policy: impl Policy + 'static) -> Self {
        self.policy = Some(Box::new(policy));
        self
    }

    pub fn logger(mut self, logger: impl Logger + 'static) -> Self {
        self.logger = Some(Box::new(logger));
        self
    }

    pub fn runner(mut self, runner: impl Runner + 'static) -> Self {
        self.runner = Some(Box::new(runner));
        self
    }
}

/* UTILITY IMPLEMENTATIONS */

impl MergeContext {
    pub fn merge_into(&mut self, other: MergeContext) -> Result<()> {
        self.executable
            .merge(other.executable)
            .context("Failed to merge pending executables")?;

        let merged: Dependencies = self
            .dependencies
            .union(&other.dependencies)
            .copied()
            .collect();

        self.dependencies = merged;
        Ok(())
    }
}

impl<'a> DecisionContext<'a> {
    pub fn new(
        candidates: HashMap<TaskID, &'a TaskContext>,
        buffer: &'a TaskRegistry,
        ticks: u64,
        units: Option<usize>,
    ) -> Self {
        Self {
            candidates,
            buffer,
            ticks,
            units,
        }
    }

    /// Candidates for resolution phase: Waiting tasks with satisfied deps
    pub fn for_resolution(
        state: &'a SchedulerState,
        capacity: Option<usize>,
    ) -> Self {
        let candidates = state
            .buffer
            .iter()
            .filter(|(tid, ctx)| {
                matches!(ctx.state, TaskState::Waiting(_))
                    && state.dependencies_satisfied(**tid)
            })
            .map(|(tid, ctx)| (*tid, ctx))
            .collect();

        Self::new(candidates, &state.buffer, state.ticks, capacity)
    }

    /// Candidates for execution phase: Ready tasks
    pub fn for_execution(
        state: &'a SchedulerState,
        capacity: Option<usize>,
    ) -> Self {
        let candidates = state
            .buffer
            .iter()
            .filter(|(_, ctx)| ctx.ready())
            .map(|(tid, ctx)| (*tid, ctx))
            .collect();

        Self::new(candidates, &state.buffer, state.ticks, capacity)
    }

    /// Candidates for preemption: Running tasks
    pub fn for_preemption(
        state: &'a SchedulerState,
        capacity: Option<usize>,
    ) -> Self {
        let candidates = state
            .buffer
            .iter()
            .filter(|(_, ctx)| ctx.running())
            .map(|(tid, ctx)| (*tid, ctx))
            .collect();

        Self::new(candidates, &state.buffer, state.ticks, capacity)
    }

    /// Candidates for retry: Error tasks
    pub fn for_retry(
        state: &'a SchedulerState,
        capacity: Option<usize>,
    ) -> Self {
        let candidates = state
            .buffer
            .iter()
            .filter(|(_, ctx)| matches!(ctx.state, TaskState::Error))
            .map(|(tid, ctx)| (*tid, ctx))
            .collect();

        Self::new(candidates, &state.buffer, state.ticks, capacity)
    }

    /// Iterator over valid candidates
    pub fn candidates(
        &self,
    ) -> impl Iterator<Item = (&TaskID, &TaskContext)> + '_ {
        self.candidates
            .iter()
            .map(|(tid, ctx)| (tid, *ctx))
    }
}

impl TaskContext {
    pub fn active(&self) -> bool {
        match &self.state {
            TaskState::Ready
            | TaskState::Running
            | TaskState::Waiting(_)
            | TaskState::Preempting => true,
            TaskState::Error | TaskState::Suspended(_) => false,
        }
    }

    pub fn ready(&self) -> bool {
        match &self.state {
            TaskState::Error
            | TaskState::Running
            | TaskState::Waiting(_)
            | TaskState::Suspended(_)
            | TaskState::Preempting => false,
            TaskState::Ready => true,
        }
    }

    pub fn running(&self) -> bool {
        match &self.state {
            TaskState::Error
            | TaskState::Waiting(_)
            | TaskState::Suspended(_)
            | TaskState::Preempting
            | TaskState::Ready => false,
            TaskState::Running => true,
        }
    }

    pub fn errored(&self) -> bool {
        match &self.state {
            TaskState::Ready
            | TaskState::Running
            | TaskState::Waiting(_)
            | TaskState::Suspended(_)
            | TaskState::Preempting => false,
            TaskState::Error => true,
        }
    }

    pub fn preempting(&self) -> bool {
        match &self.state {
            TaskState::Error
            | TaskState::Running
            | TaskState::Waiting(_)
            | TaskState::Suspended(_)
            | TaskState::Ready => false,
            TaskState::Preempting => true,
        }
    }

    pub fn dependencies(&self) -> Option<&Dependencies> {
        match &self.state {
            TaskState::Ready
            | TaskState::Running
            | TaskState::Preempting
            | TaskState::Suspended(_)
            | TaskState::Error => None,
            TaskState::Waiting(deps) => Some(deps),
        }
    }

    pub fn outcome(&self) -> Option<&TaskOutcome> {
        match &self.state {
            TaskState::Ready
            | TaskState::Running
            | TaskState::Preempting
            | TaskState::Waiting(_)
            | TaskState::Error => None,
            TaskState::Suspended(outcome) => Some(outcome),
        }
    }

    /// Check if task is missing its executable (offshore task).
    /// Used to determine if task merge should be deferred.
    pub fn missing_executable(&self) -> bool {
        self.executable.is_none()
    }

    /// Check if task has pending dependencies to wait for.
    pub fn has_pending_dependencies(&self) -> bool {
        matches!(self.state, TaskState::Waiting(_))
    }
}

impl SchedulerState {
    pub(super) fn tasks_active(
        &self,
    ) -> impl Iterator<Item = (&TaskID, &TaskContext)> {
        self.buffer
            .iter()
            .filter(|(_, ctx)| ctx.active())
    }

    pub(super) fn tasks_ready(
        &self,
    ) -> impl Iterator<Item = (&TaskID, &TaskContext)> {
        self.buffer
            .iter()
            .filter(|(_, ctx)| ctx.ready())
    }

    pub(super) fn tasks_running(
        &self,
    ) -> impl Iterator<Item = (&TaskID, &TaskContext)> {
        self.buffer
            .iter()
            .filter(|(_, ctx)| ctx.running())
    }

    pub(super) fn tasks_errored(
        &self,
    ) -> impl Iterator<Item = (&TaskID, &TaskContext)> {
        self.buffer
            .iter()
            .filter(|(_, ctx)| ctx.errored())
    }

    pub(super) fn runner_tasks(
        &self,
    ) -> impl Iterator<Item = (&TaskID, &TaskContext)> {
        self.buffer
            .iter()
            .filter(|(_, ctx)| {
                matches!(
                    ctx.state,
                    TaskState::Running | TaskState::Preempting
                )
            })
    }

    pub(super) fn get_dependencies(
        &self,
        tid: TaskID,
    ) -> Option<&Dependencies> {
        self.buffer
            .get(&tid)?
            .dependencies()
    }

    pub(super) fn take_merge(&mut self, tid: &TaskID) -> Option<MergeContext> {
        self.merges.remove(tid)
    }

    /// Check if a task's dependencies are all satisfied (completed/suspended).
    /// Returns false if task has no dependencies registered.
    pub fn dependencies_satisfied(&self, tid: TaskID) -> bool {
        self.get_dependencies(tid)
            .is_some_and(|deps| {
                deps.iter().all(|dep_tid| {
                    self.buffer
                        .get(dep_tid)
                        .and_then(|ctx| ctx.outcome())
                        .is_some()
                })
            })
    }

    /// Collect all currently running/preempting task IDs into a Vec.
    /// For operations that need to iterate over runner tasks with mutations.
    pub fn collect_runner_task_ids(&self) -> Vec<TaskID> {
        self.runner_tasks()
            .map(|(tid, _)| *tid)
            .collect()
    }
}

impl Display for TaskState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            TaskState::Preempting => "preempting",
            TaskState::Suspended(_) => "suspended",
            TaskState::Waiting(_) => "waiting",
            TaskState::Running => "running",
            TaskState::Error => "error",
            TaskState::Ready => "ready",
        };

        write!(f, "{content}")
    }
}

/* HELPER FUNCTIONS */

/// Format a cycle path into a human-readable error message.
/// Shows each task ID and its description in the cycle.
pub fn format_cycle_path(path: &[TaskID], registry: &TaskRegistry) -> String {
    let mut message = format!(
        "These {} tasks wait for each other cyclically:\n",
        path.len() - 1
    );

    for tid in path {
        if let Some(ctx) = registry.get(tid) {
            message.push_str(&format!("-> {:?}: {}\n", tid, ctx.about));
        }
    }

    message
}

// Check each task for connected dependcency cycles using DFS.
pub fn find_cycle_path(registry: &TaskRegistry) -> Option<Vec<TaskID>> {
    let mut seen = HashSet::new();
    registry
        .keys()
        .find_map(|tid| {
            (!seen.contains(tid)).then(|| {
                let mut stack = Vec::new();
                find_cycle(*tid, registry, &mut seen, &mut stack)
            })
        })
        .flatten()
}

// DFS to find a dependcency cycle connected to a given task.
fn find_cycle(
    start: TaskID,
    registry: &TaskRegistry,
    seen: &mut HashSet<TaskID>,
    stack: &mut Vec<TaskID>,
) -> Option<Vec<TaskID>> {
    stack.push(start);
    seen.insert(start);
    let cycle = registry
        .get(&start)
        .and_then(TaskContext::dependencies)
        .and_then(|deps| {
            deps.iter().find_map(|dep| {
                if !seen.contains(dep) {
                    find_cycle(*dep, registry, seen, stack)
                } else if stack.contains(dep) {
                    let cycle_start = stack
                        .iter()
                        .position(|tid| tid == dep)
                        .unwrap();
                    let mut cycle = stack[cycle_start..].to_vec();
                    cycle.push(*dep);
                    Some(cycle)
                } else {
                    None
                }
            })
        });

    stack.pop();
    cycle
}

/* TEST UTILITIES */

#[cfg(test)]
pub mod test_utils {

    use crate::scheduler::TaskContextBuilder;
    use crate::scheduler::TaskID;
    use crate::scheduler::TaskState;

    use super::*;

    /// Create a simple task context for testing with minimal boilerplate.
    pub fn task_ctx(progress: TaskState) -> TaskContext {
        TaskContextBuilder::default()
            .state(progress)
            .retriable(false)
            .incoming(HashSet::new())
            .about(String::new())
            .size(None)
            .progress(None)
            .build()
            .unwrap()
    }

    /// Create a task context with a specific size.
    pub fn task_ctx_with_size(progress: TaskState, size: u64) -> TaskContext {
        TaskContextBuilder::default()
            .state(progress)
            .retriable(false)
            .incoming(HashSet::new())
            .about(String::new())
            .size(Some(size))
            .progress(None)
            .build()
            .unwrap()
    }

    /// Create a task context with dependents (incoming dependencies).
    pub fn task_ctx_with_dependents(
        progress: TaskState,
        size: Option<u64>,
        dependents: Vec<TaskID>,
    ) -> TaskContext {
        let incoming: HashSet<TaskID> = dependents.into_iter().collect();
        TaskContextBuilder::default()
            .state(progress)
            .retriable(false)
            .incoming(incoming)
            .about(String::new())
            .size(size)
            .progress(None)
            .build()
            .unwrap()
    }

    /// Create a retriable task context.
    pub fn retriable_task_ctx(progress: TaskState) -> TaskContext {
        TaskContextBuilder::default()
            .state(progress)
            .retriable(true)
            .incoming(HashSet::new())
            .about(String::new())
            .size(None)
            .progress(None)
            .build()
            .unwrap()
    }
}
