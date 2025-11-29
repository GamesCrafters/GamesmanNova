//! # Critical Path Scheduling Policy
//!
//! Weighted critical path scheduling with preemption support.
//!
//! ## Algorithm
//!
//! Selects tasks with highest critical weight:
//!   weight(task) = size(task) + max(weight(child) for children)
//!
//! This is competitive to ≤ (2 + 1/cores) * optimal under certain
//! assumptions.
//!
//! ## Preemption Strategy
//!
//! Preempts running tasks when ready tasks have significantly
//! higher critical weight. Threshold determined by sigma parameter
//! (difference in standard deviations).

use derive_builder::Builder;

use std::collections::HashMap;
use std::mem::take;

use crate::scheduler::PolicyAction;
use crate::scheduler::PolicyDecision;
use crate::scheduler::PolicySnapshot;
use crate::scheduler::TaskID;
use crate::scheduler::core::AnyContext;
use crate::scheduler::core::State;
use crate::scheduler::traits::Policy;

/* TYPE ALIASES */

/// Generic component of a scheduler policy in charge of retrying tasks.
type RetryPolicy = Box<dyn FnMut(&[TaskID]) -> Option<TaskID>>;

/* STRUCTURES */

/// Weighted critical path scheduling policy with preemption.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct CriticalPathPolicy {
    /// The difference in standard deviations of longest blocked critical path
    /// size among two tasks that will cause the one blocking the lighter path
    /// to be immediately preempted (without necessarily scheduling the other).
    #[builder(default = "1.0f64")]
    sigma: f64,

    /// Custom retry policy, determining which tasks the scheduler will mark as
    /// ready for execution (immediately and regardless of their state).
    #[builder(default = "threshold(0)")]
    retry: RetryPolicy,

    #[builder(default, setter(skip))]
    decisions: Vec<PolicyDecision>,
}

/* IMPLEMENTATIONS */

impl Policy for CriticalPathPolicy {
    fn retry(
        &mut self,
        candidates: &[TaskID],
        _state: &State,
        _capacity: usize,
    ) -> Option<TaskID> {
        let result = (self.retry)(candidates);
        if let Some(tid) = result {
            self.decisions
                .push(PolicyDecision {
                    action: PolicyAction::Retry,
                    weight: None,
                    task: tid,
                });
        }

        result
    }

    fn preempt(
        &mut self,
        candidates: &[TaskID],
        state: &State,
        capacity: usize,
    ) -> Option<TaskID> {
        if candidates.len() < capacity {
            return None;
        }

        let incoming = build_incoming_map(state);
        let stats = compute_size_stats(state);
        let threshold = (self.sigma * stats.stddev) as u64;

        let max_ready = state
            .ready_ids()
            .map(|id| critical_weight(*id, state, &incoming, stats.mean))
            .max()?;

        let (victim, weight) = candidates
            .iter()
            .map(|id| {
                (
                    *id,
                    critical_weight(*id, state, &incoming, stats.mean),
                )
            })
            .min_by_key(|(_, w)| *w)?;

        if max_ready > weight + threshold {
            self.decisions
                .push(PolicyDecision {
                    action: PolicyAction::Preempt,
                    weight: Some(weight),
                    task: victim,
                });
            Some(victim)
        } else {
            None
        }
    }

    fn execute(
        &mut self,
        candidates: &[TaskID],
        state: &State,
        _capacity: usize,
    ) -> Option<TaskID> {
        let incoming = build_incoming_map(state);
        let stats = compute_size_stats(state);

        let (tid, weight) = candidates
            .iter()
            .map(|id| {
                (
                    *id,
                    critical_weight(*id, state, &incoming, stats.mean),
                )
            })
            .max_by_key(|(_, w)| *w)?;

        self.decisions
            .push(PolicyDecision {
                action: PolicyAction::Execute,
                weight: Some(weight),
                task: tid,
            });

        Some(tid)
    }

    fn snapshot(&mut self) -> Option<PolicySnapshot> {
        Some(PolicySnapshot {
            decisions: take(&mut self.decisions),
        })
    }
}

/* FUNCTIONS */

/// Provide each task up to `limit` retry opportunities.
pub fn threshold(limit: usize) -> RetryPolicy {
    let mut counts: HashMap<TaskID, usize> = HashMap::new();
    let policy = move |candidates: &[TaskID]| {
        let tid = *candidates.iter().next()?;
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

/// Build a map of incoming dependencies: for each task, which tasks depend on it.
fn build_incoming_map(state: &State) -> HashMap<TaskID, Vec<TaskID>> {
    let mut incoming: HashMap<TaskID, Vec<TaskID>> = HashMap::new();

    for (id, ctx) in state.iter() {
        if let AnyContext::Waiting(waiting) = ctx {
            for dep in waiting.dependencies() {
                incoming
                    .entry(*dep)
                    .or_default()
                    .push(*id);
            }
        }
    }

    incoming
}

/// Size statistics for computing thresholds and defaults.
struct SizeStats {
    mean: f64,
    stddev: f64,
}

/// Compute task size statistics across all tasks in state.
fn compute_size_stats(state: &State) -> SizeStats {
    let sizes: Vec<u64> = state
        .iter()
        .filter_map(|(_, ctx)| {
            let size = match ctx {
                AnyContext::Ready(c) => c.size(),
                AnyContext::Running(c) => c.size(),
                AnyContext::Waiting(c) => c.size(),
                AnyContext::Preempting(c) => c.size(),
                AnyContext::Suspended(c) => c.size(),
                AnyContext::Error(c) => c.size(),
            };
            size
        })
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

/// Returns size(task) + max{critical_path(child) : child depends on task}.
/// Scheduling this way is competitive to <= (2 + 1/cores) * optimal. (Under
/// certain unrealistic but pretty well-posed assumptions.)
fn critical_weight(
    tid: TaskID,
    state: &State,
    incoming: &HashMap<TaskID, Vec<TaskID>>,
    avg_size: f64,
) -> u64 {
    let avg_size = avg_size as u64;
    let mut memo = HashMap::new();
    let mut stack = vec![(tid, false)];

    while let Some((current, children_pushed)) = stack.pop() {
        if memo.contains_key(&current) {
            continue;
        }

        let Some(ctx) = state.get(&current) else {
            memo.insert(current, 0);
            continue;
        };

        if !children_pushed {
            stack.push((current, true));
            push_unvisited_children(&mut stack, &mut memo, incoming, &current);
        } else {
            let size = context_size(ctx, avg_size);
            let child_weight = max_child_weight(incoming, &current, &memo);
            memo.insert(current, size + child_weight);
        }
    }

    memo.get(&tid)
        .copied()
        .unwrap_or(0)
}

fn context_size(ctx: &AnyContext, avg: u64) -> u64 {
    let size = match ctx {
        AnyContext::Ready(c) => c.size(),
        AnyContext::Running(c) => c.size(),
        AnyContext::Waiting(c) => c.size(),
        AnyContext::Preempting(c) => c.size(),
        AnyContext::Suspended(c) => c.size(),
        AnyContext::Error(c) => c.size(),
    };
    size.unwrap_or(avg)
}

fn max_child_weight(
    incoming: &HashMap<TaskID, Vec<TaskID>>,
    current: &TaskID,
    memo: &HashMap<TaskID, u64>,
) -> u64 {
    incoming
        .get(current)
        .and_then(|children| {
            children
                .iter()
                .filter_map(|child| memo.get(child))
                .max()
                .copied()
        })
        .unwrap_or(0)
}

fn push_unvisited_children(
    stack: &mut Vec<(TaskID, bool)>,
    memo: &HashMap<TaskID, u64>,
    incoming: &HashMap<TaskID, Vec<TaskID>>,
    current: &TaskID,
) {
    if let Some(children) = incoming.get(current) {
        for child in children {
            if !memo.contains_key(child) {
                stack.push((*child, false));
            }
        }
    }
}
