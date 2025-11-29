//! # Decision Context
//!
//! Filtered view of scheduler state for policy decisions.

use crate::scheduler::TaskID;
use crate::scheduler::core::context::AnyContext;
use crate::scheduler::core::state::State;

/* STRUCTURES */

/// Restricted view of scheduler state for policy decisions.
pub struct DecisionContext<'a> {
    pub candidates: Vec<TaskID>,
    pub state: &'a State,
    pub capacity: usize,
}

/* IMPLEMENTATIONS */

impl<'a> DecisionContext<'a> {
    pub fn for_retry(state: &'a State, capacity: usize) -> Self {
        let candidates: Vec<TaskID> = state
            .error_ids()
            .copied()
            .collect();
        Self {
            candidates,
            state,
            capacity,
        }
    }

    pub fn for_preemption(state: &'a State, capacity: usize) -> Self {
        let candidates: Vec<TaskID> = state
            .running_ids()
            .copied()
            .collect();
        Self {
            candidates,
            state,
            capacity,
        }
    }

    pub fn for_execution(state: &'a State, capacity: usize) -> Self {
        let candidates: Vec<TaskID> = state
            .ready_ids()
            .copied()
            .collect();
        Self {
            candidates,
            state,
            capacity,
        }
    }

    pub fn for_resolution(state: &'a State, capacity: usize) -> Self {
        let candidates: Vec<TaskID> = state
            .waiting_ids()
            .filter_map(|id| {
                if let Some(AnyContext::Waiting(waiting)) = state.get(id) {
                    let deps_satisfied =
                        waiting
                            .dependencies()
                            .iter()
                            .all(|dep_id| {
                                matches!(
                                    state.get(dep_id),
                                    Some(AnyContext::Suspended(_))
                                )
                            });

                    if deps_satisfied { Some(*id) } else { None }
                } else {
                    None
                }
            })
            .collect();

        Self {
            candidates,
            state,
            capacity,
        }
    }

    pub fn ticks(&self) -> u64 {
        self.state.ticks()
    }

    pub fn size(&self, id: &TaskID) -> Option<u64> {
        self.state
            .get(id)
            .and_then(|ctx| match ctx {
                AnyContext::Ready(c) => c.size(),
                AnyContext::Running(c) => c.size(),
                AnyContext::Waiting(c) => c.size(),
                AnyContext::Preempting(c) => c.size(),
                AnyContext::Error(c) => c.size(),
                AnyContext::Suspended(c) => c.size(),
            })
    }

    pub fn progress(&self, id: &TaskID) -> Option<u64> {
        self.state
            .get(id)
            .and_then(|ctx| match ctx {
                AnyContext::Ready(c) => c.progress(),
                AnyContext::Running(c) => c.progress(),
                AnyContext::Waiting(c) => c.progress(),
                AnyContext::Preempting(c) => c.progress(),
                AnyContext::Error(c) => c.progress(),
                AnyContext::Suspended(c) => c.progress(),
            })
    }
}
