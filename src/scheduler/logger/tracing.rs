//! # Tracing Logger
//!
//! Structured logging via the `tracing` crate for scheduler observability.

use anyhow::Result;
use derive_builder::Builder;
use tracing::debug;
use tracing::info;
use tracing::trace;

use std::time::Duration;
use std::time::Instant;

use crate::scheduler::PolicyAction;
use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::TaskState;
use crate::scheduler::traits::Logger;

/* STRUCTURES */

/// Logger emitting structured tracing events for scheduler observability.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct TracingLogger {
    #[builder(default = "true")]
    lazy: bool,

    #[builder(default = "1")]
    frequency: usize,

    #[builder(default, setter(skip))]
    last_tick: Option<Instant>,

    #[builder(default, setter(skip))]
    changes: usize,
}

#[derive(Default)]
struct StateCounts {
    preempting: usize,
    suspended: usize,
    running: usize,
    waiting: usize,
    ready: usize,
    error: usize,
}

/* IMPLEMENTATIONS */

impl Logger for TracingLogger {
    fn report(
        &mut self,
        snapshot: &SchedulerSnapshot,
        changed: bool,
    ) -> Result<()> {
        let elapsed = self.last_tick.map(|t| t.elapsed());
        self.last_tick = Some(Instant::now());
        self.emit_transitions(snapshot);

        if self.should_emit(changed) {
            self.emit_tick(snapshot, elapsed);
        }

        self.emit_runner(snapshot);
        self.emit_policy(snapshot);
        Ok(())
    }
}

/* UTILITIES */

impl TracingLogger {
    fn should_emit(&mut self, changed: bool) -> bool {
        if self.lazy && !changed {
            return false;
        }

        self.changes += 1;
        self.changes
            .is_multiple_of(self.frequency)
    }

    fn emit_transitions(&self, snapshot: &SchedulerSnapshot) {
        for t in &snapshot.transitions {
            info!(
                task = %t.task,
                from = ?t.from,
                to = ?t.to,
                phase = ?t.phase,
                "task_transition"
            );
        }
    }

    fn emit_tick(
        &self,
        snapshot: &SchedulerSnapshot,
        elapsed: Option<Duration>,
    ) {
        let counts = self.count_states(snapshot);

        debug!(
            tick_duration_us = elapsed.map(|d| d.as_micros() as u64),
            suspended = counts.suspended,
            preempting = counts.preempting,
            waiting = counts.waiting,
            running = counts.running,
            ready = counts.ready,
            error = counts.error,
            tick = snapshot.tick,
            "scheduler_tick"
        );
    }

    fn emit_runner(&self, snapshot: &SchedulerSnapshot) {
        let Some(ref runner) = snapshot.runner else {
            return;
        };

        if runner.ticks.is_empty() {
            trace!(
                capacity = runner.capacity,
                ticks_collected = 0u64,
                "runner_stats"
            );
            return;
        }

        let len = runner.ticks.len() as u64;
        let sum: u64 = runner.ticks.iter().sum();
        let max = runner
            .ticks
            .iter()
            .max()
            .copied()
            .unwrap_or(0);

        trace!(
            ticks_collected = len,
            avg_tick_us = sum / len,
            capacity = runner.capacity,
            max_tick_us = max,
            "runner_stats"
        );
    }

    fn emit_policy(&self, snapshot: &SchedulerSnapshot) {
        let Some(ref policy) = snapshot.policy else {
            return;
        };

        let (mut execs, mut preempts, mut retries) = (0, 0, 0);
        for d in &policy.decisions {
            match d.action {
                PolicyAction::Execute => execs += 1,
                PolicyAction::Preempt => preempts += 1,
                PolicyAction::Retry => retries += 1,
            }
        }

        trace!(
            decisions = policy.decisions.len(),
            preemptions = preempts,
            executions = execs,
            retries = retries,
            "policy_stats"
        );
    }

    fn count_states(&self, snapshot: &SchedulerSnapshot) -> StateCounts {
        let mut counts = StateCounts::default();

        for ctx in snapshot.tasks.values() {
            match ctx.state {
                TaskState::Suspended(_) => counts.suspended += 1,
                TaskState::Preempting => counts.preempting += 1,
                TaskState::Waiting(_) => counts.waiting += 1,
                TaskState::Running => counts.running += 1,
                TaskState::Ready => counts.ready += 1,
                TaskState::Error => counts.error += 1,
            }
        }

        counts
    }
}
