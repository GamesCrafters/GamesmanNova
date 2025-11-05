//! # Policy Implementations
//!
//! TODO

use std::collections::HashMap;

use crate::traits::scheduler::Policy;
use crate::types::scheduler::SchedulerState;
use crate::types::scheduler::SizeStats;
use crate::types::scheduler::TaskID;
use crate::types::scheduler::TaskRegistry;
use crate::types::scheduler::policy::CriticalPathPolicy;
use crate::types::scheduler::policy::RetryPolicy;
use crate::types::scheduler::policy::TrivialPolicy;

/* POLICY IMPLEMENTATIONS */

impl Policy for TrivialPolicy {
    fn retry(&mut self, _state: &SchedulerState) -> Option<TaskID> {
        None
    }

    fn preempt(&mut self, _state: &SchedulerState) -> Option<TaskID> {
        None
    }

    fn execute(&mut self, state: &SchedulerState) -> Option<TaskID> {
        state
            .tasks_ready()
            .map(|(tid, _ctx)| *tid)
            .min()
    }
}

impl Policy for CriticalPathPolicy {
    fn retry(&mut self, state: &SchedulerState) -> Option<TaskID> {
        (self.retry)(state)
    }

    fn preempt(&mut self, state: &SchedulerState) -> Option<TaskID> {
        let running_count = state.tasks_running().count();
        if state.units == 0 || running_count < state.units {
            return None;
        }

        let stats = compute_size_stats(&state.registry);
        let threshold = (self.sigma * stats.stddev) as u64;
        let max_ready_depth = state
            .tasks_ready()
            .map(|(tid, _ctx)| calculate_depth(*tid, &state.registry))
            .max()?;

        let (min_running_tid, min_running_depth) = state
            .tasks_running()
            .map(|(tid, _ctx)| (*tid, calculate_depth(*tid, &state.registry)))
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
            .map(|(tid, _ctx)| (*tid, calculate_depth(*tid, &state.registry)))
            .max_by_key(|(_, depth)| *depth)
            .map(|(tid, _)| tid)
    }
}

/* PRESET RETRY POLICIES */

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
fn calculate_depth(tid: TaskID, registry: &TaskRegistry) -> u64 {
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::core::scheduler::utils::test_utils::*;
    use crate::types::scheduler::TaskState;
    use crate::types::scheduler::policy::CriticalPathPolicyBuilder;
    use std::collections::HashSet;

    #[test]
    fn test_trivial_never_preempts() {
        let mut policy = TrivialPolicy;
        let mut state = SchedulerState::default();

        state
            .registry
            .insert(1, task_ctx(TaskState::Running));

        assert_eq!(policy.preempt(&state), None);
    }

    #[test]
    fn test_critical_path_selects_longest_path() {
        let mut state = SchedulerState::default();
        let mut policy = CriticalPathPolicyBuilder::default()
            .sigma(0.0)
            .build()
            .unwrap();

        state.units = 0;
        state.registry.insert(
            2,
            task_ctx_with_dependents(TaskState::Ready, Some(5), vec![3]),
        );

        state
            .registry
            .insert(1, task_ctx_with_size(TaskState::Ready, 10));

        let mut deps = HashSet::new();
        deps.insert(2);
        state.registry.insert(
            3,
            task_ctx_with_size(TaskState::Waiting(deps), 20),
        );

        assert_eq!(policy.execute(&state), Some(2));
    }

    #[test]
    fn test_critical_path_no_preemption_without_workers() {
        let mut state = SchedulerState::default();
        let mut policy = CriticalPathPolicyBuilder::default()
            .sigma(0.0)
            .build()
            .unwrap();

        state.units = 0;
        state
            .registry
            .insert(1, task_ctx_with_size(TaskState::Running, 5));

        state
            .registry
            .insert(2, task_ctx_with_size(TaskState::Ready, 100));

        assert_eq!(policy.preempt(&state), None);
    }

    #[test]
    fn test_critical_path_preempts_at_capacity() {
        let mut state = SchedulerState::default();
        let mut policy = CriticalPathPolicyBuilder::default()
            .sigma(0.0)
            .build()
            .unwrap();

        state.units = 1;
        state
            .registry
            .insert(1, task_ctx_with_size(TaskState::Running, 5));

        state
            .registry
            .insert(2, task_ctx_with_size(TaskState::Ready, 100));

        assert_eq!(policy.preempt(&state), Some(1));
    }

    #[test]
    fn test_critical_path_respects_sigma_threshold() {
        let mut state = SchedulerState::default();
        let mut policy = CriticalPathPolicyBuilder::default()
            .sigma(10.0)
            .build()
            .unwrap();

        state.units = 1;
        state
            .registry
            .insert(1, task_ctx_with_size(TaskState::Running, 50));

        state
            .registry
            .insert(2, task_ctx_with_size(TaskState::Ready, 55));

        assert_eq!(policy.preempt(&state), None);
    }
}
