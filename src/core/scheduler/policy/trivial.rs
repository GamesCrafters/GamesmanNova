//! # Trivial Policy Implementation
//!
//! TODO

use crate::core::scheduler::SchedulerState;
use crate::core::scheduler::TaskID;
use crate::traits::scheduler::Policy;

/* STRUCTURES */

/// No-preemption policy that always picks the task with lowest ID.
#[derive(Default)]
pub struct TrivialPolicy;

/* IMPL TRAIT FOR TYPE */

impl Policy for TrivialPolicy {
    fn retry(&mut self, _state: &SchedulerState) -> Option<TaskID> {
        None
    }

    fn preempt(&mut self, _state: &SchedulerState) -> Option<TaskID> {
        None
    }

    fn execute(&mut self, state: &SchedulerState) -> Option<TaskID> {
        state
            .tasks_ready()
            .map(|(tid, _ctx)| *tid)
            .min()
    }
}
