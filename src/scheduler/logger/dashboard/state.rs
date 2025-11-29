//! Internal state management for dashboard logger.

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use std::collections::HashMap;
use std::io::Stdout;
use std::time::Instant;

use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::TaskContextSnapshot;
use crate::scheduler::TaskID;
use crate::scheduler::TaskState;
use crate::scheduler::logger::dashboard::histogram::TickSketch;

/* CONSTANTS */

const MIN_INTERVAL: f64 = 1.0;
const SMOOTHING: f64 = 0.3;

/* STRUCTURES */

/// Internal state for TUI logger.
pub struct TuiLoggerState {
    pub task_start_times: HashMap<TaskID, Instant>,
    pub task_throughput: HashMap<TaskID, f64>,
    pub task_progress: HashMap<TaskID, (Instant, u64)>,
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    pub tick_sketch: TickSketch,
    pub start_time: Instant,
    pub observe_count: usize,
}

/// State count breakdown by task state.
pub struct StateCounts {
    pub suspended: usize,
    pub running: usize,
    pub waiting: usize,
    pub errors: usize,
    pub ready: usize,
}

/// Bar segment sizes for state visualization.
#[derive(Default)]
pub struct BarSegments {
    pub suspended: usize,
    pub running: usize,
    pub waiting: usize,
    pub errors: usize,
    pub ready: usize,
}

/* IMPLEMENTATIONS */

pub(super) fn count_states(snapshot: &SchedulerSnapshot) -> StateCounts {
    let mut counts = StateCounts {
        suspended: 0,
        running: 0,
        waiting: 0,
        errors: 0,
        ready: 0,
    };

    for ctx in snapshot.tasks.values() {
        match ctx.state {
            TaskState::Suspended(_) => counts.suspended += 1,
            TaskState::Preempting | TaskState::Running => counts.running += 1,
            TaskState::Waiting(_) => counts.waiting += 1,
            TaskState::Error => counts.errors += 1,
            TaskState::Ready => counts.ready += 1,
        }
    }

    counts
}

pub fn track_times(state: &mut TuiLoggerState, snapshot: &SchedulerSnapshot) {
    let dominated = |ctx: &TaskContextSnapshot| {
        matches!(
            ctx.state,
            TaskState::Suspended(_) | TaskState::Error
        )
    };

    let active = |ctx: &TaskContextSnapshot| {
        matches!(
            ctx.state,
            TaskState::Running | TaskState::Preempting
        )
    };

    snapshot
        .tasks
        .iter()
        .filter(|(_, ctx)| dominated(ctx))
        .for_each(|(tid, _)| {
            state.task_start_times.remove(tid);
            state.task_throughput.remove(tid);
            state.task_progress.remove(tid);
        });

    snapshot
        .tasks
        .iter()
        .filter(|(_, ctx)| active(ctx))
        .for_each(|(tid, _)| {
            state
                .task_start_times
                .entry(*tid)
                .or_insert_with(Instant::now);
        });
}

pub fn update_throughput(
    state: &mut TuiLoggerState,
    snapshot: &SchedulerSnapshot,
) {
    let now = Instant::now();

    for (tid, ctx) in &snapshot.tasks {
        let Some(current) = ctx.progress else {
            continue;
        };

        let Some((last_time, last_progress)) = state.task_progress.get(tid)
        else {
            state
                .task_progress
                .insert(*tid, (now, current));
            continue;
        };

        let elapsed = now
            .duration_since(*last_time)
            .as_secs_f64();
        if elapsed < MIN_INTERVAL {
            continue;
        }

        let rate = compute_smoothed_rate(
            current,
            *last_progress,
            elapsed,
            state.task_throughput.get(tid),
        );

        state
            .task_throughput
            .insert(*tid, rate);
        state
            .task_progress
            .insert(*tid, (now, current));
    }
}

fn compute_smoothed_rate(
    current: u64,
    last: u64,
    elapsed: f64,
    prev_rate: Option<&f64>,
) -> f64 {
    let delta = current.saturating_sub(last);
    let instant = delta as f64 / elapsed;

    match prev_rate {
        Some(&prev) => SMOOTHING * instant + (1.0 - SMOOTHING) * prev,
        None => instant,
    }
}
