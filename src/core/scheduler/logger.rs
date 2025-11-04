//! # Logger Implementations
//!
//! TODO

use anyhow::Result;

use crate::traits::scheduler::Logger;
use crate::types::scheduler::CountLogger;
use crate::types::scheduler::Progress;
use crate::types::scheduler::SchedulerState;

/* CLI LOGGER */

impl Logger for CountLogger {
    fn log(&mut self, state: &SchedulerState) -> Result<()> {
        let mut error = 0;
        let mut ready = 0;
        let mut running = 0;
        let mut waiting = 0;
        let mut finished = 0;

        for ctx in state.registry.values() {
            match ctx.progress {
                Progress::Error => error += 1,
                Progress::Ready => ready += 1,
                Progress::Running => running += 1,
                Progress::Waiting(_) => waiting += 1,
                Progress::Finished(_) => finished += 1,
            }
        }

        println!(
            "[Tick {}] Ready: {} | Running: {} | Waiting: {} | Finished: {} \
            | Error: {}",
            state.ticks, ready, running, waiting, finished, error
        );

        Ok(())
    }
}
