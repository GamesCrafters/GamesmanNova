//! Task collection, filtering, and sorting.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::Instant;

use crate::scheduler::PolicySnapshot;
use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::TaskContextSnapshot;
use crate::scheduler::TaskID;
use crate::scheduler::TaskState;

use super::SortOrder;
use super::TaskFilter;
use super::format::format_weight;

/* IMPLEMENTATIONS */

pub fn collect<'a>(
    snapshot: &'a SchedulerSnapshot,
    filters: &[TaskFilter],
) -> Vec<(TaskID, &'a TaskContextSnapshot)> {
    snapshot
        .tasks
        .iter()
        .filter(|(_, ctx)| matches(&ctx.state, filters))
        .map(|(tid, ctx)| (*tid, ctx))
        .collect()
}

pub fn sort(
    tasks: &mut [(TaskID, &TaskContextSnapshot)],
    order: SortOrder,
    times: &HashMap<TaskID, Instant>,
) {
    match order {
        SortOrder::StartTime => {
            tasks.sort_by(|a, b| {
                times
                    .get(&a.0)
                    .cmp(&times.get(&b.0))
                    .then_with(|| a.0.cmp(&b.0))
            });
        },
        SortOrder::Progress => {
            let pct = |ctx: &TaskContextSnapshot| {
                ctx.progress.and_then(|p| {
                    ctx.size
                        .map(|s| p as f64 / s as f64)
                })
            };
            tasks.sort_by(|a, b| {
                pct(b.1)
                    .partial_cmp(&pct(a.1))
                    .unwrap_or(Ordering::Equal)
                    .then_with(|| a.0.cmp(&b.0))
            });
        },
        SortOrder::TaskID => {
            tasks.sort_by_key(|(tid, _)| *tid);
        },
    }
}

pub fn extract_weights(
    policy: Option<&PolicySnapshot>,
) -> HashMap<TaskID, String> {
    let mut result = HashMap::new();

    let Some(snap) = policy else {
        return result;
    };

    let all: Vec<u64> = snap
        .decisions
        .iter()
        .filter_map(|d| d.weight)
        .collect();

    for decision in &snap.decisions {
        let Some(w) = decision.weight else {
            continue;
        };

        if let Some(fmt) = format_weight(w, &all) {
            result.insert(decision.task, fmt);
        }
    }

    result
}

fn matches(state: &TaskState, filters: &[TaskFilter]) -> bool {
    filters.iter().any(|f| {
        matches!(
            (f, state),
            (TaskFilter::Suspended, TaskState::Suspended(_))
                | (TaskFilter::Preempting, TaskState::Preempting)
                | (TaskFilter::Waiting, TaskState::Waiting(_))
                | (TaskFilter::Running, TaskState::Running)
                | (TaskFilter::Ready, TaskState::Ready)
                | (TaskFilter::Error, TaskState::Error)
        )
    })
}
