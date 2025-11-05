//! # Sync Runner Implementation
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;

use crate::traits::scheduler::Executable;
use crate::traits::scheduler::Runner;
use crate::types::scheduler::PollStatus;
use crate::types::scheduler::TaskID;
use crate::types::scheduler::TaskOutcomes;
use crate::types::scheduler::runner::sync::SyncRunner;

/* IMPLEMENTATION */

impl Runner for SyncRunner {
    fn units(&self) -> usize {
        0
    }

    fn execute(
        &mut self,
        tid: TaskID,
        awaited: TaskOutcomes,
        mut executable: Box<dyn Executable>,
    ) -> Result<()> {
        if self.running.contains_key(&tid) {
            bail!("Task {} is already running.", tid);
        }

        let mut result = executable.tick(awaited);
        while result.ready() {
            result = executable.tick(TaskOutcomes::new());
        }

        self.running
            .insert(tid, executable);

        self.results
            .insert(tid, Ok(result));

        Ok(())
    }

    fn poll(&mut self, tid: TaskID) -> Result<PollStatus> {
        match self.results.remove(&tid) {
            Some(Ok(update)) => Ok(PollStatus::Ready(update)),
            Some(Err(e)) => Ok(PollStatus::Panic(e.to_string())),
            None => Ok(PollStatus::Pending),
        }
    }

    fn preempt(&mut self, _tid: TaskID) -> Result<()> {
        Ok(())
    }

    fn collect(&mut self, tid: TaskID) -> Result<Box<dyn Executable>> {
        self.running
            .remove(&tid)
            .context(format!(
                "Task {} is not running or not ready to collect",
                tid
            ))
    }
}
