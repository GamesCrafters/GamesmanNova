//! # Critical Path Policy
//!
//! Weighted critical path scheduling policy with preemption.

use derive_builder::Builder;

use crate::core::scheduler::policy::critical::threshold;
use crate::types::scheduler::SchedulerState;
use crate::types::scheduler::TaskID;

/* TYPE ALIASES */

/// Generic component of a scheduler policy in charge of retrying tasks.
pub type RetryPolicy = Box<dyn FnMut(&SchedulerState) -> Option<TaskID>>;

/* TYPE */

/// Weighted critical path scheduling policy with preemption.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct CriticalPathPolicy {
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
