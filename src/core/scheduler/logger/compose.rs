//! # Composed Logger
//!
//! Logger that composes multiple loggers, calling each in sequence.

use anyhow::Result;

use crate::core::scheduler::SchedulerSnapshot;
use crate::traits::scheduler::Logger;

pub struct ComposedLogger {
    loggers: Vec<Box<dyn Logger>>,
}

impl ComposedLogger {
    pub fn new() -> Self {
        Self {
            loggers: Vec::new(),
        }
    }

    pub fn with(mut self, logger: Box<dyn Logger>) -> Self {
        self.loggers.push(logger);
        self
    }
}

impl Logger for ComposedLogger {
    fn observe(&mut self, snapshot: &SchedulerSnapshot, changed: bool) -> Result<()> {
        for logger in &mut self.loggers {
            logger.observe(snapshot, changed)?;
        }
        Ok(())
    }
}

impl Default for ComposedLogger {
    fn default() -> Self {
        Self::new()
    }
}
