//! # Critical Path Policy Implementation
//!
//! TODO

use std::collections::HashMap;

use derive_builder::Builder;

use crate::core::scheduler::SchedulerState;
use crate::core::scheduler::SizeStats;
use crate::core::scheduler::TaskID;
use crate::core::scheduler::TaskRegistry;
use crate::traits::scheduler::Policy;

/* TYPE ALIASES */

/// Generic component of a scheduler policy in charge of retrying tasks.
pub type RetryPolicy = Box<dyn FnMut(&SchedulerState) -> Option<TaskID>>;

/* STRUCTURES */

/// Weighted critical path scheduling policy with preemption.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct CriticalPathPolicy {
    /// The difference in standard deviations of longest blocked critical path
    /// size among two tasks that will cause the one blocking the lighter path
    /// to be immediately preempted (without necessarily scheduling the other).
    #[builder(default = "1.0f64")]
    pub sigma: f64,

    /// Custom retry policy, determining which tasks the scheduler will mark as
    /// ready for execution (immediately and regardless of their state).
    #[builder(default = "threshold(0)")]
    pub retry: RetryPolicy,
}

/* IMPLEMENTATIONS */

impl Policy for CriticalPathPolicy {
    fn retry(&mut self, state: &SchedulerState) -> Option<TaskID> {
        (self.retry)(state)
    }

    fn preempt(&mut self, state: &SchedulerState) -> Option<TaskID> {
        let running_count = state.tasks_running().count();
        if state.units == 0 || running_count < state.units {
            return None;
        }

        let stats = compute_size_stats(&state.buffer);
        let threshold = (self.sigma * stats.stddev) as u64;
        let max_ready_depth = state
            .tasks_ready()
            .map(|(tid, _ctx)| critical_weight(*tid, &state.buffer))
            .max()?;

        let (min_running_tid, min_running_depth) = state
            .tasks_running()
            .map(|(tid, _ctx)| (*tid, critical_weight(*tid, &state.buffer)))
            .min_by_key(|(_, depth)| *depth)?;

        if max_ready_depth > min_running_depth + threshold {
            Some(min_running_tid)
        } else {
            None
        }
    }

    fn execute(&mut self, state: &SchedulerState) -> Option<TaskID> {
        state
            .tasks_ready()
            .map(|(tid, _ctx)| (*tid, critical_weight(*tid, &state.buffer)))
            .max_by_key(|(_, depth)| *depth)
            .map(|(tid, _)| tid)
    }
}

/* FUNCTIONS */

/// Provide each task up to `limit` retry opportunities.
pub fn threshold(limit: usize) -> RetryPolicy {
    let mut counts: HashMap<TaskID, usize> = HashMap::new();
    let policy = move |state: &SchedulerState| {
        let tid = *state
            .tasks_errored()
            .map(|(tid, _)| tid)
            .next()?;

        let count = counts.entry(tid).or_insert(0);
        if *count < limit {
            *count += 1;
            Some(tid)
        } else {
            None
        }
    };

    Box::new(policy)
}

/* HELPER FUNCTIONS */

/// Returns size(task) + max{critical_path(child) : child depends on task}.
/// Scheduling this way is competitive to <= (2 + 1/cores) * optimal. (Under
/// certain unrealistic but pretty well-posed assumptions.)
fn critical_weight(tid: TaskID, registry: &TaskRegistry) -> u64 {
    let avg_size = compute_size_stats(registry).mean as u64;
    let mut memo = HashMap::new();
    let mut stack = vec![(tid, false)];

    while let Some((current, children_pushed)) = stack.pop() {
        if memo.contains_key(&current) {
            continue;
        }

        let Some(ctx) = registry.get(&current) else {
            memo.insert(current, 0);
            continue;
        };

        if !children_pushed {
            stack.push((current, true));
            for child in &ctx.incoming {
                if !memo.contains_key(child) {
                    stack.push((*child, false));
                }
            }
        } else {
            let task_size = ctx.size.unwrap_or(avg_size);
            let max_child = ctx
                .incoming
                .iter()
                .filter_map(|child| memo.get(child))
                .max()
                .copied()
                .unwrap_or(0);

            memo.insert(current, task_size + max_child);
        }
    }

    memo.get(&tid)
        .copied()
        .unwrap_or(0)
}

/// Compute task size statistics across the registry.
fn compute_size_stats(registry: &TaskRegistry) -> SizeStats {
    let sizes: Vec<u64> = registry
        .values()
        .filter_map(|ctx| ctx.size)
        .collect();

    if sizes.is_empty() {
        return SizeStats {
            stddev: 0.0,
            mean: 1.0,
        };
    }

    let mean = sizes.iter().sum::<u64>() as f64 / sizes.len() as f64;
    let variance = sizes
        .iter()
        .map(|&s| {
            let diff = s as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / sizes.len() as f64;

    SizeStats {
        stddev: variance.sqrt(),
        mean,
    }
}

/* TESTS */

#[cfg(test)]
mod tests {}
