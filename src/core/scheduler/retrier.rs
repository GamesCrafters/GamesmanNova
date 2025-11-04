//! # Retrier Implementations
//!
//! TODO

use crate::traits::scheduler::Retrier;
use crate::types::scheduler::LimitedRetrier;
use crate::types::scheduler::NoRetrier;
use crate::types::scheduler::Progress;
use crate::types::scheduler::SchedulerState;
use crate::types::scheduler::TaskID;

/* RETRIER IMPLEMENTATIONS */

impl Retrier for NoRetrier {
    fn retry(&mut self, _state: &SchedulerState) -> Option<TaskID> {
        None
    }
}

impl Retrier for LimitedRetrier {
    fn retry(&mut self, state: &SchedulerState) -> Option<TaskID> {
        let tid = state
            .registry
            .iter()
            .filter(|(_, ctx)| ctx.retriable)
            .filter(|(_, ctx)| matches!(ctx.progress, Progress::Error))
            .map(|(&tid, _)| tid)
            .find(|&tid| {
                self.counts
                    .get(&tid)
                    .copied()
                    .unwrap_or(0)
                    < self.limit
            })?;

        *self.counts.entry(tid).or_insert(0) += 1;
        Some(tid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::scheduler::utils::test_utils::*;
    use crate::types::scheduler::LimitedRetrierBuilder;

    #[test]
    fn test_no_retrier_never_retries() {
        let mut retrier = NoRetrier;
        let mut state = SchedulerState::default();
        state
            .registry
            .insert(1, retriable_task_ctx(Progress::Error));

        assert_eq!(retrier.retry(&state), None);
        assert_eq!(retrier.retry(&state), None);
    }

    #[test]
    fn test_limited_retrier_respects_limit() {
        let mut retrier = LimitedRetrierBuilder::default()
            .limit(3)
            .build()
            .unwrap();

        let mut state = SchedulerState::default();
        state
            .registry
            .insert(1, retriable_task_ctx(Progress::Error));

        assert_eq!(retrier.retry(&state), Some(1));
        assert_eq!(retrier.retry(&state), Some(1));
        assert_eq!(retrier.retry(&state), Some(1));
        assert_eq!(retrier.retry(&state), None);
        assert_eq!(retrier.retry(&state), None);
        assert_eq!(retrier.counts.get(&1), Some(&3));
    }

    #[test]
    fn test_limited_retrier_only_retries_retriable_tasks() {
        let mut retrier = LimitedRetrierBuilder::default()
            .limit(3)
            .build()
            .unwrap();

        let mut state = SchedulerState::default();
        state
            .registry
            .insert(1, task_ctx(Progress::Error));

        assert_eq!(retrier.retry(&state), None);
        assert_eq!(retrier.counts.get(&1), None);
    }

    #[test]
    fn test_limited_retrier_only_retries_error_tasks() {
        let mut retrier = LimitedRetrierBuilder::default()
            .limit(3)
            .build()
            .unwrap();

        let mut state = SchedulerState::default();
        state
            .registry
            .insert(1, retriable_task_ctx(Progress::Ready));

        assert_eq!(retrier.counts.get(&1), None);
        assert_eq!(retrier.retry(&state), None);
    }
}
