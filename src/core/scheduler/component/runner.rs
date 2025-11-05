//! # Runner Implementations
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use async_trait::async_trait;

use crate::traits::scheduler::Executable;
use crate::traits::scheduler::Runner;
use crate::types::scheduler::TaskID;
use crate::types::scheduler::TaskOutcomes;
use crate::types::scheduler::YieldUpdate;
use crate::types::scheduler::runner::SyncRunner;

/* SYNCHRONOUS RUNNER */

#[async_trait]
impl Runner for SyncRunner {
    async fn execute(
        &mut self,
        tid: TaskID,
        mut task: Box<dyn Executable>,
        deps: TaskOutcomes,
    ) -> Result<()> {
        if self.running.contains_key(&tid) {
            bail!("Task {} is already running.", tid);
        }

        let mut result = task.tick(deps);
        while result.ready() {
            result = task.tick(TaskOutcomes::new());
        }

        self.running.insert(tid, task);
        self.results
            .insert(tid, Ok(result));

        Ok(())
    }

    fn poll(&mut self, tid: TaskID) -> Option<Result<YieldUpdate>> {
        self.results.remove(&tid)
    }

    async fn collect(&mut self, tid: TaskID) -> Result<Box<dyn Executable>> {
        self.running
            .remove(&tid)
            .context(format!("Task {} is not running.", tid))
    }
}
