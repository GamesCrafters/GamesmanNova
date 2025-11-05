//! # Trivial Policy Implementation
//!
//! TODO

use crate::traits::scheduler::Policy;
use crate::types::scheduler::SchedulerState;
use crate::types::scheduler::TaskID;
use crate::types::scheduler::policy::trivial::TrivialPolicy;

/* IMPLEMENTATION */

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
