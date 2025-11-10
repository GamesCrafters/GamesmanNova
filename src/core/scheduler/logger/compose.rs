//! # Composed Logger
//!
//! Logger that composes multiple loggers, calling each in sequence.

use anyhow::Result;
use derive_builder::Builder;

use crate::core::scheduler::SchedulerSnapshot;
use crate::traits::scheduler::Logger;

/* STRUCTURES */

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct ComposedLogger {
    #[builder(setter(each = "logger"))]
    loggers: Vec<Box<dyn Logger>>,
}

/* IMPLEMENTATIONS */

impl Logger for ComposedLogger {
    fn observe(
        &mut self,
        snapshot: &SchedulerSnapshot,
        changed: bool,
    ) -> Result<()> {
        for logger in &mut self.loggers {
            logger.observe(snapshot, changed)?;
        }

        Ok(())
    }
}
