//! # Scheduler Traits
//!
//! TODO

#[cfg(test)]
use mockall::automock;

use anyhow::Result;
use async_trait::async_trait;

use crate::types::scheduler::SchedulerState;
use crate::types::scheduler::TaskID;
use crate::types::scheduler::TaskOutcomes;
use crate::types::scheduler::YieldUpdate;

/* INTERFACES */

#[async_trait]
#[cfg_attr(test, automock)]
pub trait Runner {
    /// Initiates execution of a task with its dependencies' outcomes. Transfers
    /// ownership of the executable to the runner. The task switches to Running
    /// state in the scheduler before this call. May return immediately or block
    /// until first yield (synchronous execution).
    async fn execute(
        &mut self,
        tid: TaskID,
        task: Box<dyn Executable>,
        deps: TaskOutcomes,
    ) -> Result<()>;

    /// Checks if a running task has yielded since the last poll. Returns None
    /// if the task is still executing, Some(Ok(update)) if yielded successfully
    /// or Some(Err(e)) if it failed. This method must not block.
    fn poll(&mut self, tid: TaskID) -> Option<Result<YieldUpdate>>;

    /// Prepares for preemption if task has not yet yielded, otherwise collects
    /// its yield update and retrieves the executable. Transfers ownership of
    /// executable back to the scheduler. May block briefly if the task is mid
    /// tick, bounded by tick duration.
    async fn collect(&mut self, tid: TaskID) -> Result<Box<dyn Executable>>;
}

#[cfg_attr(test, automock)]
pub trait Executable: Send {
    /// Executes one tick of work (bounded time quantum). Receives outcomes of
    /// all dependencies and returns update indicating whether task finished, is
    /// waiting for new dependencies, or is ready to continue. Must checkpoint
    /// internal state via &mut self to support resurrection.
    fn tick(&mut self, deps: TaskOutcomes) -> YieldUpdate;

    /// Returns the task's internal estimate of its own relative size. Used by
    /// policies to make weighted scheduling decisions.
    fn size(&self) -> Option<u64> {
        None
    }
}

#[cfg_attr(test, automock)]
pub trait Policy {
    /// Identifies a failed task that should be retried. Returns a TaskID with
    /// TaskState::Error that should transition to Ready, or None if no retries
    /// are needed. Must be idempotent - repeated calls without state changes
    /// should return the same result.
    fn retry(&mut self, state: &SchedulerState) -> Option<TaskID>;

    /// Identifies running task that should be preempted. Returns a TaskID with
    /// TaskState::Running that should be fetched and transitioned to Ready, or
    /// None if no preemption is needed. Must be idempotent.
    fn preempt(&mut self, state: &SchedulerState) -> Option<TaskID>;

    /// Selects the next ready task to execute. Returns TaskID with ready state
    /// that should be dispatched to the runner, or None if no tasks should run.
    /// Must be idempotent.
    fn execute(&mut self, state: &SchedulerState) -> Option<TaskID>;
}

#[cfg_attr(test, automock)]
pub trait Logger {
    /// Does anything related to observability upon observing `state`. Called
    /// only when a scheduling tick results in state changes.
    fn log(&mut self, state: &SchedulerState) -> Result<()>;
}
