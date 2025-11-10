//! # Scheduler Traits
//!
//! TODO

#[cfg(test)]
use mockall::automock;

use anyhow::Result;
use anyhow::bail;
use std::any::Any;

use crate::core::scheduler::PollStatus;
use crate::core::scheduler::SchedulerState;
use crate::core::scheduler::TaskID;
use crate::core::scheduler::TaskOutcomes;
use crate::core::scheduler::YieldUpdate;

/* INTERFACES */

#[cfg_attr(test, automock)]
pub trait Runner {
    /// Returns the number of parallel execution units available. Zero indicates
    /// synchronous execution where tasks run to completion before returning.
    fn units(&self) -> usize;

    /// Initiates execution of a task with its dependencies' outcomes. Transfers
    /// ownership of the executable to the runner. The task switches to Running
    /// state in the scheduler before this call. May return immediately or block
    /// until first yield (synchronous execution).
    fn execute(
        &mut self,
        tid: TaskID,
        awaited: TaskOutcomes,
        executable: Box<dyn Executable>,
    ) -> Result<()>;

    /// Checks if running task has yielded since the last poll. Returns Pending
    /// if still executing, Ready if completed with update, or Panic if the task
    /// panicked. This method must not block. Returns error for runner failures
    /// (invalid task ID, etc).
    fn poll(&mut self, tid: TaskID) -> Result<PollStatus>;

    /// Signals a running task to preempt (stop execution and yield control).
    /// For synchronous runners, this may be a no-op. For concurrent runners,
    /// this sets the preemption signal. Does not block. Returns an error for
    /// infrastructure failures (task not found, not running, etc).
    fn preempt(&mut self, tid: TaskID) -> Result<()>;

    /// Retrieves a completed task's executable. Only succeeds if the task has
    /// finished executing (poll returned Ready or Panic). Returns an error if
    /// the task is not found, still executing, or not ready to collect.
    fn collect(&mut self, tid: TaskID) -> Result<Box<dyn Executable>>;

    /// Samples the progress of a running task. Returns None if the task is not
    /// found, not running, or doesn't report progress. This method queries the
    /// last known progress value without blocking. For concurrent runners, this
    /// reflects progress sampled after the most recent tick() call.
    fn progress(&self, tid: TaskID) -> Option<u64>;
}

#[cfg_attr(test, automock)]
pub trait Executable: Send + Any {
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

    /// Returns the task's current progress using the same metric as size(). For
    /// example, if size() returns all operations, progress() returns operations
    /// completed. Used for live progress tracking and UI updates.
    fn progress(&self) -> Option<u64> {
        None
    }

    /// Merges another task of the same type into this one. Used when a task is
    /// discovered multiple times - the new version is merged with the existing
    /// version. Implementations should downcast to verify type compatibility
    /// and return an error if types don't match. Default implementation returns
    /// an error.
    fn merge(&mut self, _other: Box<dyn Executable>) -> Result<()> {
        bail!("Merging not supported for this task type")
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
    /// Observes a scheduler snapshot at each tick. The `changed` flag indicates
    /// whether any state transitions occurred during the tick.
    fn observe(
        &mut self,
        snapshot: &crate::core::scheduler::SchedulerSnapshot,
        changed: bool,
    ) -> Result<()>;
}
