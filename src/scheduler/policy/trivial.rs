//! # Trivial Scheduling Policy
//!
//! Simplest possible policy: no preemption, no retries, always
//! selects task with lowest ID.

use crate::scheduler::TaskID;
use crate::scheduler::core::State;
use crate::scheduler::traits::Policy;

/* STRUCTURES */

/// No-preemption policy that always picks the task with lowest ID.
#[derive(Default)]
pub struct TrivialPolicy;

/* TRAIT IMPLEMENTATIONS */

impl Policy for TrivialPolicy {
    fn retry(
        &mut self,
        _candidates: &[TaskID],
        _state: &State,
        _capacity: usize,
    ) -> Option<TaskID> {
        None
    }

    fn preempt(
        &mut self,
        _candidates: &[TaskID],
        _state: &State,
        _capacity: usize,
    ) -> Option<TaskID> {
        None
    }

    fn execute(
        &mut self,
        candidates: &[TaskID],
        _state: &State,
        _capacity: usize,
    ) -> Option<TaskID> {
        candidates.iter().min().copied()
    }
}
