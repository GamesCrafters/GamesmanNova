//! # Count Logger Implementation
//!
//! TODO

use anyhow::Result;

use crate::core::scheduler::SchedulerSnapshot;
use crate::core::scheduler::TaskState;
use crate::traits::scheduler::Logger;

/* STRUCTURES */

/// Simple command-line logger that prints task progress counts.
#[derive(Default)]
pub struct CountLogger;

/* IMPL TRAIT FOR TYPE */

impl Logger for CountLogger {
    fn observe(
        &mut self,
        snapshot: &SchedulerSnapshot,
        changed: bool,
    ) -> Result<()> {
        if !changed {
            return Ok(());
        }

        let mut error = 0;
        let mut ready = 0;
        let mut running = 0;
        let mut waiting = 0;
        let mut suspended = 0;
        let mut preempting = 0;

        for ctx in snapshot.tasks.values() {
            match ctx.progress {
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
