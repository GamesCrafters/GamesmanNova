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
    /// Initiates concurrent execution of a task with its dependencies' outcomes
    /// transitioning it to running state.
    async fn spawn(
        &mut self,
        tid: TaskID,
        task: Box<dyn Executable>,
        deps: TaskOutcomes,
    ) -> Result<()>;

    /// Checks if a running task has yielded since the last poll. Returns `None`
    /// if no yield has occurred yet.
    fn poll(&mut self, tid: TaskID) -> Option<Result<YieldUpdate>>;

    /// Stops running task and retrieves its executable for rescheduling. Used
    /// for pausing and yielding.
    async fn stop(&mut self, tid: TaskID) -> Result<Box<dyn Executable>>;
}

#[cfg_attr(test, automock)]
pub trait Executable: Send {
    /// Executes arbitrary logic and finally yields by returning, modifying its
    /// state so that it may continue were it left off (if necessary). Returns
    /// an update about the task's operational metadata, such as its priority.
    fn execute(&mut self, deps: TaskOutcomes) -> YieldUpdate;

    /// Returns the task's internal estimate of its own relative size. Used by
    /// policies to make weighted scheduling decisions.
    fn size(&self) -> Option<u64> {
        None
    }
}

#[cfg_attr(test, automock)]
pub trait Policy {
    /// If some [`TaskID`] is returned, instructs the scheduler to immediately
    /// preempt it and then mark it as [`Progress::Ready`]. Must be idempotent,
    /// and any task returned must be [`Progress::Running`] in `state`.
    fn pause(&mut self, state: &SchedulerState) -> Option<TaskID>;

    /// If some [`TaskID`] is returned, instructs the scheduler to immediately
    /// mark its state as [`Progress::Running`] (and then run it right after).
    /// Must be idempotent, and any task returned must be [`Progress::Ready`]
    /// in the scheduler `state`.
    fn next(&mut self, state: &SchedulerState) -> Option<TaskID>;
}

#[cfg_attr(test, automock)]
pub trait Retrier {
    /// If some [`TaskID`] is returned, instructs the scheduler to immediately
    /// set its state to [`Progress::Ready`] (restoring its original state and
    /// priority). Must be idempotent, and any task returned must have already
    /// [`Progress::Error`] (in the scheduler `state`).
    fn retry(&mut self, state: &SchedulerState) -> Option<TaskID>;
}

#[cfg_attr(test, automock)]
pub trait Logger {
    /// Does anything related to observability upon observing `state`. Called
    /// only when a scheduling tick results in state changes.
    fn log(&mut self, state: &SchedulerState) -> Result<()>;
}
