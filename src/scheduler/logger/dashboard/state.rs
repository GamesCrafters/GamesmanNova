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

/* CONSTANTS */

// EMA parameters for throughput calculation.
const MIN_INTERVAL: f64 = 1.0;
const ALPHA: f64 = 0.3;

/* STRUCTURES */

/// Internal state for TUI logger.
pub struct TuiLoggerState {
    /// When the logger started (for time-based spinner).
    pub start_time: Instant,

    /// When each task started running (for duration tracking).
    pub task_start_times: HashMap<TaskID, Instant>,

    /// Last progress sample for each task (timestamp, progress).
    pub task_progress: HashMap<TaskID, (Instant, u64)>,

    /// Calculated throughput for each task (ops per second).
    pub task_throughput: HashMap<TaskID, f64>,

    /// Terminal interface.
    pub terminal: Terminal<CrosstermBackend<Stdout>>,

    /// Number of observe calls (for update frequency).
    pub observe_count: usize,
}

/// State count breakdown by task state.
pub struct StateCounts {
    pub running: usize,
    pub ready: usize,
    pub waiting: usize,
    pub suspended: usize,
    pub errors: usize,
}

/// Bar segment sizes for state visualization.
pub struct BarSegments {
    pub running: usize,
    pub ready: usize,
    pub waiting: usize,
    pub suspended: usize,
    pub errors: usize,
}

/* STATE MANAGEMENT */

pub fn count_states(snapshot: &SchedulerSnapshot) -> StateCounts {
    let mut counts = StateCounts {
        running: 0,
        ready: 0,
        waiting: 0,
        suspended: 0,
        errors: 0,
    };

    for ctx in snapshot.tasks.values() {
        match ctx.state {
            TaskState::Running | TaskState::Preempting => counts.running += 1,
            TaskState::Ready => counts.ready += 1,
            TaskState::Waiting(_) => counts.waiting += 1,
            TaskState::Suspended(_) => counts.suspended += 1,
            TaskState::Error => counts.errors += 1,
        }
    }

    counts
}

pub fn track_times(state: &mut TuiLoggerState, snapshot: &SchedulerSnapshot) {
    let active = |(_, ctx): &(&TaskID, &TaskContextSnapshot)| {
        matches!(
            ctx.state,
            TaskState::Running | TaskState::Preempting
        )
    };

    let tasks = snapshot
        .tasks
        .iter()
        .filter(active);
    for (tid, _) in tasks {
        state
            .task_start_times
            .entry(*tid)
            .or_insert_with(Instant::now);
    }
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

        let delta = current.saturating_sub(*last_progress);
        let instantaneous = delta as f64 / elapsed;
        let smoothed = if let Some(&previous) = state.task_throughput.get(tid) {
            ALPHA * instantaneous + (1.0 - ALPHA) * previous
        } else {
            instantaneous
        };

        state
            .task_throughput
            .insert(*tid, smoothed);

        state
            .task_progress
            .insert(*tid, (now, current));
    }
}
