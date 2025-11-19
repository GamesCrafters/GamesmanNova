//! # Composed Logger
//!
//! Logger that composes multiple loggers, calling each in sequence.

use anyhow::Result;
use derive_builder::Builder;

use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::traits::Logger;

/* STRUCTURES */

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct ComposeLogger {
    #[builder(setter(each = "logger"))]
    loggers: Vec<Box<dyn Logger>>,
}

/* IMPLEMENTATIONS */

impl Logger for ComposeLogger {
    fn report(
        &mut self,
        snapshot: &SchedulerSnapshot,
        changed: bool,
    ) -> Result<()> {
        for logger in &mut self.loggers {
            logger.report(snapshot, changed)?;
        }

        Ok(())
    }
}
