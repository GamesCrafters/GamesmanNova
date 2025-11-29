//! # Preemption Phase
//!
//! Signals running tasks to stop based on policy decisions.

use anyhow::Context as _;
use anyhow::Result;
use anyhow::bail;

use crate::scheduler::TaskID;
use crate::scheduler::core::context::AnyContext;
use crate::scheduler::core::phase::Phase;
use crate::scheduler::core::phase::PhaseResult;
use crate::scheduler::core::state::State;
use crate::scheduler::traits::Policy;
use crate::scheduler::traits::Runner;

/* STRUCTURES */

pub(in crate::scheduler) struct Preemption<'a> {
    state: &'a mut State,
    runner: &'a mut dyn Runner,
    policy: &'a mut dyn Policy,
    capacity: usize,
}

/* IMPLEMENTATIONS */

impl<'a> Preemption<'a> {
    pub(in crate::scheduler) fn new(
        state: &'a mut State,
        runner: &'a mut dyn Runner,
        policy: &'a mut dyn Policy,
        capacity: usize,
    ) -> Self {
        Self {
            state,
            runner,
            policy,
            capacity,
        }
    }

    fn transition(&mut self, id: TaskID) -> Result<()> {
        let ctx = self
            .state
            .remove(&id)
            .context("Task not in state")?
            .into_running()
            .context("Task not in Running state")?;

        self.runner.preempt(id)?;

        let preempting = ctx.preempt();
        self.state
            .insert(id, AnyContext::Preempting(preempting));

        Ok(())
    }
}

impl<'a> Phase for Preemption<'a> {
    fn execute(&mut self) -> Result<PhaseResult> {
        let mut affected = 0;

        loop {
            let running_ids: Vec<TaskID> = self
                .state
                .running_ids()
                .copied()
                .collect();

            if running_ids.is_empty() {
                break;
            }

            let Some(id) =
                self.policy
                    .preempt(&running_ids, &self.state, self.capacity)
            else {
                break;
            };

            if !running_ids.contains(&id) {
                bail!(
                    "Policy selected non-candidate task {} in preemption",
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
        "Preemption"
    }
}
