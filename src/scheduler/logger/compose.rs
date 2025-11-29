//! # Composed Logger
//!
//! Logger that composes multiple loggers, calling each in sequence.

use anyhow::Result;

use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::traits::Logger;

/* STRUCTURES */

pub struct ComposeLogger {
    loggers: Vec<Box<dyn Logger>>,
}

#[derive(Default)]
pub struct ComposeLoggerBuilder {
    loggers: Vec<Box<dyn Logger>>,
}

impl ComposeLoggerBuilder {
    #[allow(private_bounds)]
    pub fn logger(mut self, logger: impl Logger + 'static) -> Self {
        self.loggers.push(Box::new(logger));
        self
    }

    pub fn build(self) -> Result<ComposeLogger> {
        Ok(ComposeLogger {
            loggers: self.loggers,
        })
    }
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
