//! # Count Logger
//!
//! Records task state counts at each scheduler tick for metrics.

use anyhow::Result;
use derive_builder::Builder;

use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::TaskState;
use crate::scheduler::traits::Logger;

/* STRUCTURES */

/// Simple command-line logger that prints task progress counts.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct CountLogger {
    #[builder(default = "1")]
    frequency: usize,

    #[builder(default = "true")]
    lazy: bool,

    #[builder(default)]
    #[builder(setter(skip))]
    changes: usize,
}

/* IMPL TRAIT FOR TYPE */

impl Logger for CountLogger {
    fn observe(
        &mut self,
        snapshot: &SchedulerSnapshot,
        changed: bool,
    ) -> Result<()> {
        let record = match (self.lazy, changed) {
            (true, false) => false,
            (false, _) => snapshot
                .tick
                .is_multiple_of(self.frequency as u64),
            (true, true) => {
                self.changes += 1;
                self.changes
                    .is_multiple_of(self.frequency)
            },
        };

        if !record {
            return Ok(());
        }

        let mut error = 0;
        let mut ready = 0;
        let mut running = 0;
        let mut waiting = 0;
        let mut suspended = 0;
        let mut preempting = 0;

        for ctx in snapshot.tasks.values() {
            match ctx.state {
                TaskState::Error => error += 1,
                TaskState::Ready => ready += 1,
                TaskState::Running => running += 1,
                TaskState::Waiting(_) => waiting += 1,
                TaskState::Suspended(_) => suspended += 1,
                TaskState::Preempting => preempting += 1,
            }
        }

        println!(
            "[Tick {}] Ready: {} | Running: {} | Preempting: {} | Waiting: {} \
            | Suspended: {} | Error: {}",
            snapshot.tick,
            ready,
            running,
            preempting,
            waiting,
            suspended,
            error
        );

        Ok(())
    }
}
