//! # Execution Phase
//!
//! Selects and dispatches Ready tasks to runners based on policy.

use anyhow::Context as _;
use anyhow::Result;
use anyhow::bail;

use crate::scheduler::TaskID;
use crate::scheduler::TaskOutcomes;
use crate::scheduler::core::context::AnyContext;
use crate::scheduler::core::phase::Phase;
use crate::scheduler::core::phase::PhaseResult;
use crate::scheduler::core::state::State;
use crate::scheduler::traits::Policy;
use crate::scheduler::traits::Runner;

/* STRUCTURES */

pub(in crate::scheduler) struct Execution<'a> {
    state: &'a mut State,
    runner: &'a mut dyn Runner,
    policy: &'a mut dyn Policy,
    capacity: usize,
}

/* IMPLEMENTATIONS */

impl<'a> Execution<'a> {
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

    fn at_capacity(&self) -> bool {
        let running = self.state.running_ids().count();
        running >= self.capacity
    }

    fn dispatch(&mut self, id: TaskID) -> Result<bool> {
        use crate::scheduler::DispatchOutcome;

        let ctx = self
            .state
            .remove(&id)
            .context("Task not in state")?
            .into_ready()
            .context("Task not in Ready state")?;

        let (running, executable) = ctx.dispatch();
        let awaited = self.collect_awaited()?;

        match self
            .runner
            .execute(id, awaited, executable)?
        {
            DispatchOutcome::Accepted => {
                self.state
                    .insert(id, AnyContext::Running(running));
                Ok(true)
            },
            DispatchOutcome::CapacityExhausted(executable) => {
                self.state
                    .insert(id, AnyContext::Ready(running.restore(executable)));
                Ok(false)
            },
        }
    }

    fn collect_awaited(&self) -> Result<TaskOutcomes> {
        let mut outcomes = TaskOutcomes::new();
        for (dep_id, ctx) in self.state.iter() {
            if let AnyContext::Suspended(suspended) = ctx {
                outcomes.insert(*dep_id, *suspended.outcome());
            }
        }

        Ok(outcomes)
    }
}

impl<'a> Phase for Execution<'a> {
    fn execute(&mut self) -> Result<PhaseResult> {
        let mut affected = 0;

        while !self.at_capacity() {
            let ready_ids: Vec<TaskID> = self
                .state
                .ready_ids()
                .copied()
                .collect();

            if ready_ids.is_empty() {
                break;
            }

            let id =
                self.policy
                    .execute(&ready_ids, &self.state, self.capacity);

            let Some(id) = id else {
                break;
            };

            if !ready_ids.contains(&id) {
                bail!(
                    "Policy selected non-candidate task {} in execution",
                    id
                );
            }

            if self.dispatch(id)? {
                affected += 1;
            } else {
                break;
            }
        }

        let changed = affected > 0;
        Ok(PhaseResult { changed, affected })
    }

    fn name(&self) -> &'static str {
        "Execution"
    }
}
