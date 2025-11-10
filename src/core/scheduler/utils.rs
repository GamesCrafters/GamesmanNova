//! # Scheduler Utility Implementations
//!
//! TODO    

use anyhow::Context;
use anyhow::Result;

use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::core::scheduler::Dependencies;
use crate::core::scheduler::MergeContext;
use crate::core::scheduler::SchedulerState;
use crate::core::scheduler::TaskContext;
use crate::core::scheduler::TaskID;
use crate::core::scheduler::TaskOutcome;
use crate::core::scheduler::TaskRegistry;
use crate::core::scheduler::TaskState;
use crate::core::scheduler::YieldIntention;
use crate::core::scheduler::YieldUpdate;

/* UTILITY IMPLEMENTATIONS */

impl YieldUpdate {
    pub fn ready(&self) -> bool {
        match self.intention {
            YieldIntention::Suspended(_) | YieldIntention::Waiting(_) => false,
            YieldIntention::Ready => true,
        }
    }
}

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
}

impl SchedulerState {
    pub fn tasks_active(
        &self,
    ) -> impl Iterator<Item = (&TaskID, &TaskContext)> {
        self.buffer
            .iter()
            .filter(|(_, ctx)| ctx.active())
    }

    pub fn tasks_ready(&self) -> impl Iterator<Item = (&TaskID, &TaskContext)> {
        self.buffer
            .iter()
            .filter(|(_, ctx)| ctx.ready())
    }

    pub fn tasks_running(
        &self,
    ) -> impl Iterator<Item = (&TaskID, &TaskContext)> {
        self.buffer
            .iter()
            .filter(|(_, ctx)| ctx.running())
    }

    pub fn tasks_errored(
        &self,
    ) -> impl Iterator<Item = (&TaskID, &TaskContext)> {
        self.buffer
            .iter()
            .filter(|(_, ctx)| ctx.errored())
    }

    pub fn runner_tasks(
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

    pub fn get_dependencies(&self, tid: TaskID) -> Option<&Dependencies> {
        self.buffer
            .get(&tid)?
            .dependencies()
    }

    pub fn take_merge(&mut self, tid: &TaskID) -> Option<MergeContext> {
        self.merges.remove(tid)
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
    use super::*;
    use crate::core::scheduler::TaskContextBuilder;

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
