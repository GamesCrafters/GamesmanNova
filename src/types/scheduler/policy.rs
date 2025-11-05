//! # Scheduler Policy Types
//!
//! TODO

use derive_builder::Builder;

use crate::core::scheduler::component::policy::threshold;
use crate::types::scheduler::SchedulerState;
use crate::types::scheduler::TaskID;

/* TYPE ALIASES */

/// Generic component of a scheduler policy in charge of retrying tasks.
pub type RetryPolicy = Box<dyn FnMut(&SchedulerState) -> Option<TaskID>>;

/* POLICY STRUCTURES */

/// No-preemption policy that always picks the task with lowest ID.
#[derive(Default)]
pub struct TrivialPolicy;

/// Weighted critical path scheduling policy with preemption.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct CriticalPathPolicy {
    /// The number of parallel units being used for the runner component, which
    /// will greatly impact policy. If zero, preemptions will never be scheduled
    /// (because they cannot be beneficial without parallelism).
    pub units: usize,

    /// The difference in standard deviations of longest blocked critical path
    /// size among two tasks that will cause the one blocking the lighter path
    /// to be immediately preempted (without necessarily scheduling the other).
    #[builder(default = "1.0f64")]
    pub sigma: f64,

    /// Custom retry policy, determining which tasks the scheduler will mark as
    /// ready for execution (immediately and regardless of their state).
    #[builder(default = "threshold(0)")]
    pub retry: RetryPolicy,
}
