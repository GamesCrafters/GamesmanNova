//! # Scheduler Utility Implementations
//!
//! TODO    

use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::types::scheduler::Dependencies;
use crate::types::scheduler::Progress;
use crate::types::scheduler::SchedulerState;
use crate::types::scheduler::TaskContext;
use crate::types::scheduler::TaskID;
use crate::types::scheduler::TaskRegistry;

/* UTILITY IMPLEMENTATIONS */

impl TaskContext {
    pub fn active(&self) -> bool {
        match &self.progress {
            Progress::Ready | Progress::Running | Progress::Waiting(_) => true,
            Progress::Error | Progress::Finished(_) => false,
        }
    }

    pub fn dependencies(&self) -> Option<&Dependencies> {
        match &self.progress {
            Progress::Ready
            | Progress::Running
            | Progress::Finished(_)
            | Progress::Error => None,
            Progress::Waiting(deps) => Some(deps),
        }
    }
}

impl SchedulerState {
    pub fn ready_tasks(&self) -> impl Iterator<Item = (&TaskID, &TaskContext)> {
        self.registry
            .iter()
            .filter(|(_, ctx)| match ctx.progress {
                Progress::Error
                | Progress::Waiting(_)
                | Progress::Finished(_)
                | Progress::Running => false,
                Progress::Ready => true,
            })
    }

    pub fn running_tasks(
        &self,
    ) -> impl Iterator<Item = (&TaskID, &TaskContext)> {
        self.registry
            .iter()
            .filter(|(_, ctx)| match ctx.progress {
                Progress::Error
                | Progress::Waiting(_)
                | Progress::Finished(_)
                | Progress::Ready => false,
                Progress::Running => true,
            })
    }

    pub fn get_dependencies(&self, tid: TaskID) -> Option<&Dependencies> {
        self.registry
            .get(&tid)?
            .dependencies()
    }
}

impl Display for Progress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            Progress::Finished(_) => "finished",
            Progress::Waiting(_) => "waiting",
            Progress::Running => "running",
            Progress::Error => "error",
            Progress::Ready => "ready",
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
    use crate::types::scheduler::TaskContextBuilder;

    /// Create a simple task context for testing with minimal boilerplate.
    pub fn task_ctx(progress: Progress) -> TaskContext {
        TaskContextBuilder::default()
            .progress(progress)
            .retriable(false)
            .incoming(HashSet::new())
            .about(String::new())
            .size(None)
            .build()
            .unwrap()
    }

    /// Create a task context with a specific size.
    pub fn task_ctx_with_size(progress: Progress, size: u64) -> TaskContext {
        TaskContextBuilder::default()
            .progress(progress)
            .retriable(false)
            .incoming(HashSet::new())
            .about(String::new())
            .size(Some(size))
            .build()
            .unwrap()
    }

    /// Create a task context with dependents (incoming dependencies).
    pub fn task_ctx_with_dependents(
        progress: Progress,
        size: Option<u64>,
        dependents: Vec<TaskID>,
    ) -> TaskContext {
        let incoming: HashSet<TaskID> = dependents.into_iter().collect();
        TaskContextBuilder::default()
            .progress(progress)
            .retriable(false)
            .incoming(incoming)
            .about(String::new())
            .size(size)
            .build()
            .unwrap()
    }

    /// Create a retriable task context.
    pub fn retriable_task_ctx(progress: Progress) -> TaskContext {
        TaskContextBuilder::default()
            .progress(progress)
            .retriable(true)
            .incoming(HashSet::new())
            .about(String::new())
            .size(None)
            .build()
            .unwrap()
    }
}
