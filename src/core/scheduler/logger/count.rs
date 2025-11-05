//! # Count Logger Implementation
//!
//! TODO

use anyhow::Result;

use crate::traits::scheduler::Logger;
use crate::types::scheduler::SchedulerState;
use crate::types::scheduler::TaskState;
use crate::types::scheduler::logger::count::CountLogger;

/* IMPLEMENTATION */

impl Logger for CountLogger {
    fn log(&mut self, state: &SchedulerState) -> Result<()> {
        let mut error = 0;
        let mut ready = 0;
        let mut running = 0;
        let mut waiting = 0;
        let mut finished = 0;
        let mut preempting = 0;

        for ctx in state.registry.values() {
            match ctx.progress {
                TaskState::Error => error += 1,
                TaskState::Ready => ready += 1,
                TaskState::Running => running += 1,
                TaskState::Waiting(_) => waiting += 1,
                TaskState::Finished(_) => finished += 1,
                TaskState::Preempting => preempting += 1,
            }
        }

        println!(
            "[Tick {}] Ready: {} | Running: {} | Preempting: {} | Waiting: {} \
            | Finished: {} | Error: {}",
            state.ticks, ready, running, preempting, waiting, finished, error
        );

        Ok(())
    }
}
