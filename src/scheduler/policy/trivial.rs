//! # Trivial Scheduling Policy
//!
//! Simplest possible policy: no preemption, no retries, always
//! selects task with lowest ID.

use crate::scheduler::DecisionContext;
use crate::scheduler::TaskID;
use crate::scheduler::traits::Policy;

/* STRUCTURES */

/// No-preemption policy that always picks the task with lowest ID.
#[derive(Default)]
pub struct TrivialPolicy;

/* TRAIT IMPLEMENTATIONS */

impl Policy for TrivialPolicy {
    fn retry<'a>(&mut self, _ctx: &DecisionContext<'a>) -> Option<TaskID> {
        None
    }

    fn preempt<'a>(&mut self, _ctx: &DecisionContext<'a>) -> Option<TaskID> {
        None
    }

    fn execute<'a>(&mut self, ctx: &DecisionContext<'a>) -> Option<TaskID> {
        ctx.candidates
            .keys()
            .copied()
            .min()
    }
}
