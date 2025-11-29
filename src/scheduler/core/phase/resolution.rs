//! # Resolution Phase
//!
//! Immediately executes satisfied Waiting tasks without policy involvement.

use anyhow::Context as _;
use anyhow::Result;

use crate::scheduler::TaskID;
use crate::scheduler::TaskOutcomes;
use crate::scheduler::core::context::AnyContext;
use crate::scheduler::core::phase::Phase;
use crate::scheduler::core::phase::PhaseResult;
use crate::scheduler::core::state::State;
use crate::scheduler::traits::Policy;
use crate::scheduler::traits::Runner;

/* STRUCTURES */

pub(in crate::scheduler) struct Resolution<'a> {
    state: &'a mut State,
    runner: &'a mut dyn Runner,
    policy: &'a mut dyn Policy,
    capacity: usize,
}

/* IMPLEMENTATIONS */

impl<'a> Resolution<'a> {
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

    fn find_satisfied(&self) -> Vec<TaskID> {
        self.state
            .waiting_ids()
            .filter_map(|id| {
                if let Some(AnyContext::Waiting(waiting)) = self.state.get(id) {
                    let satisfied =
                        waiting
                            .dependencies()
                            .iter()
                            .all(|dep_id| {
                                matches!(
                                    self.state.get(dep_id),
                                    Some(AnyContext::Suspended(_))
                                )
                            });

                    if satisfied { Some(*id) } else { None }
                } else {
                    None
                }
            })
            .collect()
    }

    fn resolve(&mut self, id: TaskID) -> Result<bool> {
        use crate::scheduler::DispatchOutcome;

        let ctx = self
            .state
            .remove(&id)
            .context("Task not in state")?
            .into_waiting()
            .context("Task not in Waiting state")?;

        let dependencies = ctx.dependencies().clone();
        let (running, executable) = ctx.resolve();
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
                self.state.insert(
                    id,
                    AnyContext::Waiting(running.wait(executable, dependencies)),
                );
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

impl<'a> Phase for Resolution<'a> {
    fn execute(&mut self) -> Result<PhaseResult> {
        let mut affected = 0;

        while !self.at_capacity() {
            let satisfied = self.find_satisfied();

            if satisfied.is_empty() {
                break;
            }

            let id =
                self.policy
                    .execute(&satisfied, &self.state, self.capacity);

            let Some(id) = id else {
                break;
            };

            if self.resolve(id)? {
                affected += 1;
            } else {
                break;
            }
        }

        let changed = affected > 0;
        Ok(PhaseResult { changed, affected })
    }

    fn name(&self) -> &'static str {
        "Resolution"
    }
}
