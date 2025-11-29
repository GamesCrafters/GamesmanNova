//! # Scheduler Traits
//!
//! Core extensibility traits for the Nova scheduler system.
//!
//! ## Traits
//!
//! - **Runner**: Execution backend (sync, threaded, distributed)
//! - **Policy**: Decision making (execute, preempt, retry)
//! - **Executable**: Task implementation (tick, size, progress)
//! - **Logger**: Observability (snapshots, metrics)
//!
//! ## Policy Lifetime Parameters
//!
//! Policy methods use generic lifetime `<'a>` to allow both
//! `&mut self` (mutable policy state) and `&DecisionContext<'a>`
//! (borrowed scheduler state) in the same method signature.

#[cfg(test)]
use mockall::automock;

use anyhow::Result;
use anyhow::bail;
use std::any::Any;

use crate::scheduler::DecisionContext;
use crate::scheduler::PolicySnapshot;
use crate::scheduler::PollStatus;
use crate::scheduler::RunnerSnapshot;
use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::TaskID;
use crate::scheduler::TaskOutcomes;
use crate::scheduler::YieldUpdate;

/* SCHEDULER INTERFACES */

#[cfg_attr(test, automock)]
pub(super) trait Runner {
    /// Returns the number of parallel execution units available. None indicates
    /// unlimited capacity (synchronous execution where tasks run to completion).
    fn capacity(&self) -> Option<usize>;

    /// Initiates execution of a task with its dependencies' outcomes. Transfers
    /// ownership of the executable to the runner. The task switches to Running
    /// state in the scheduler before this call. May return immediately or block
    /// until first yield (synchronous execution).
    fn execute(
        &mut self,
        id: TaskID,
        awaited: TaskOutcomes,
        executable: Box<dyn Executable>,
    ) -> Result<()>;

    /// Checks if running task has yielded since the last poll. Returns Pending
    /// if still executing, Ready if completed with update, or Panic if the task
    /// panicked. This method must not block. Returns error for runner failures
    /// (invalid task ID, etc).
    fn poll(&mut self, id: TaskID) -> Result<PollStatus>;

    /// Signals a running task to preempt (stop execution and yield control).
    /// For synchronous runners, this may be a no-op. For concurrent runners,
    /// this sets the preemption signal. Does not block. Returns an error for
    /// infrastructure failures (task not found, not running, etc).
    fn preempt(&mut self, id: TaskID) -> Result<()>;

    /// Retrieves a completed task's executable. Only succeeds if the task has
    /// finished executing (poll returned Ready or Panic). Returns an error if
    /// the task is not found, still executing, or not ready to collect.
    fn collect(&mut self, id: TaskID) -> Result<Box<dyn Executable>>;

    /// Samples the progress of a running task. Returns None if the task is not
    /// found, not running, or doesn't report progress. This method queries the
    /// last known progress value without blocking. For concurrent runners, this
    /// reflects progress sampled after the most recent tick() call.
    fn progress(&self, id: TaskID) -> Option<u64>;

    /// Returns a snapshot of runner state for observability. Returns None if
    /// the runner doesn't support observability. Snapshots represent current
    /// state only; loggers are responsible for any accumulation.
    fn snapshot(&mut self) -> Option<RunnerSnapshot> {
        None
    }
}

#[cfg_attr(test, automock)]
pub(super) trait Executable: Send + Any {
    /// Executes one tick of work (bounded time quantum). Receives outcomes of
    /// all dependencies and returns update indicating whether task finished, is
    /// waiting for new dependencies, or is ready to continue. Must checkpoint
    /// internal state via &mut self to support resurrection.
    ///
    /// Returns None to continue ticking without yielding to scheduler, or
    /// Some(YieldUpdate) to yield control back to scheduler with the given update.
    fn tick(&mut self, deps: TaskOutcomes) -> Option<YieldUpdate>;

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

    /// Returns the unique identifier of this task. The ID includes both the
    /// task category (type) and component, uniquely identifying the task within
    /// the scheduler. Used to namespace tasks by type and prevent inappropriate
    /// merging between different task types operating on the same component.
    /// Must be implemented by all task types.
    fn id(&self) -> TaskID;
}

#[cfg_attr(test, automock)]
pub(super) trait Policy {
    /// Identifies a failed task that should be retried. Returns a TaskID from
    /// the provided candidates (Error tasks), or None if no retries are needed.
    /// Must be idempotent - repeated calls without state changes should return
    /// the same result.
    fn retry<'a>(&mut self, ctx: &DecisionContext<'a>) -> Option<TaskID>;

    /// Identifies running task that should be preempted. Returns a TaskID from
    /// the provided candidates (Running tasks), or None if no preemption is
    /// needed. Must be idempotent.
    fn preempt<'a>(&mut self, ctx: &DecisionContext<'a>) -> Option<TaskID>;

    /// Selects the next task to execute from candidates (Ready or satisfied
    /// Waiting tasks depending on phase). Returns a TaskID that should be
    /// dispatched to the runner, or None if no tasks should run. Must be
    /// idempotent.
    fn execute<'a>(&mut self, ctx: &DecisionContext<'a>) -> Option<TaskID>;

    /// Returns a snapshot of policy state for observability. Returns None if
    /// the policy doesn't support observability. Snapshots represent current
    /// state only; loggers are responsible for any accumulation.
    fn snapshot(&mut self) -> Option<PolicySnapshot> {
        None
    }
}

#[cfg_attr(test, automock)]
pub(super) trait Logger {
    /// Observes a scheduler snapshot at each tick. The `changed` flag indicates
    /// whether any state transitions occurred during the tick.
    fn report(
        &mut self,
        snapshot: &SchedulerSnapshot,
        changed: bool,
    ) -> Result<()>;
}
