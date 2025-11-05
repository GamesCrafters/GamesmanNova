//! # Trivial Policy
//!
//! No-preemption policy that always picks the task with lowest ID.

/* TYPE */

/// No-preemption policy that always picks the task with lowest ID.
#[derive(Default)]
pub struct TrivialPolicy;
