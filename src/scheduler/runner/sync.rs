//! # Synchronous Runner
//!
//! Single-threaded blocking execution. Tasks run to completion
//! before returning from execute().

use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;

use crate::scheduler::DispatchOutcome;
use crate::scheduler::TaskID;
use crate::scheduler::TaskOutcomes;
use crate::scheduler::traits::Executable;
use crate::scheduler::traits::PollStatus;
use crate::scheduler::traits::Runner;
use crate::scheduler::traits::YieldUpdate;

/* STRUCTURES */

/// Synchronous runner that just blocks on task spawns.
#[derive(Default)]
pub struct SyncRunner {
    running: HashMap<TaskID, Box<dyn Executable>>,
    results: HashMap<TaskID, Result<YieldUpdate>>,
}

/* TRAIT IMPLEMENTATIONS */

impl Runner for SyncRunner {
    fn capacity(&self) -> Option<usize> {
        None
    }

    fn execute(
        &mut self,
        tid: TaskID,
        awaited: TaskOutcomes,
        mut executable: Box<dyn Executable>,
    ) -> Result<crate::scheduler::DispatchOutcome> {
        if self.running.contains_key(&tid) {
            bail!("Task {} is already running.", tid);
        }

        let mut result = executable.tick(awaited);
        while result.is_none() {
            result = executable.tick(TaskOutcomes::new());
        }

        let result = result.expect("guaranteed");
        self.running
            .insert(tid, executable);
        self.results
            .insert(tid, Ok(result));

        Ok(DispatchOutcome::Accepted)
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

    fn progress(&self, tid: TaskID) -> Option<u64> {
        self.running.get(&tid)?.progress()
    }
}
