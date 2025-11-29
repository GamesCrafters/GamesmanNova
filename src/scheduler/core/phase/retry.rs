//! # Retry Phase
//!
//! Moves Error tasks back to Ready based on policy decisions.

use anyhow::Context as _;
use anyhow::Result;
use anyhow::bail;

use crate::scheduler::TaskID;
use crate::scheduler::core::context::AnyContext;
use crate::scheduler::core::phase::Phase;
use crate::scheduler::core::phase::PhaseResult;
use crate::scheduler::core::state::State;
use crate::scheduler::traits::Policy;

/* STRUCTURES */

pub(in crate::scheduler) struct Retry<'a> {
    state: &'a mut State,
    policy: &'a mut dyn Policy,
}

/* IMPLEMENTATIONS */

impl<'a> Retry<'a> {
    pub(in crate::scheduler) fn new(
        state: &'a mut State,
        policy: &'a mut dyn Policy,
    ) -> Self {
        Self { state, policy }
    }

    fn transition(&mut self, id: TaskID) -> Result<()> {
        let ctx = self
            .state
            .remove(&id)
            .context("Task not in state")?
            .into_error()
            .context("Task not in Error state")?;

        let ready = ctx.retry();
        self.state
            .insert(id, AnyContext::Ready(ready));

        Ok(())
    }
}

impl<'a> Phase for Retry<'a> {
    fn execute(&mut self) -> Result<PhaseResult> {
        let mut affected = 0;
        let capacity = 0;

        loop {
            let error_ids: Vec<TaskID> = self
                .state
                .error_ids()
                .copied()
                .collect();

            if error_ids.is_empty() {
                break;
            }

            let Some(id) = self
                .policy
                .retry(&error_ids, &self.state, capacity)
            else {
                break;
            };

            if !error_ids.contains(&id) {
                bail!(
                    "Policy selected non-candidate task {} in retry",
                    id
                );
            }

            self.transition(id)?;
            affected += 1;
        }

        let changed = affected > 0;
        Ok(PhaseResult { changed, affected })
    }

    fn name(&self) -> &'static str {
        "Retry"
    }
}
