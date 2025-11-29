//! # Collection Phase
//!
//! Polls runners and processes task yields, discoveries, and merges.

use anyhow::Context as _;
use anyhow::Result;
use anyhow::bail;

use std::collections::HashSet;

use crate::scheduler::TaskID;
use crate::scheduler::TaskOutcome;
use crate::scheduler::core::context::AnyContext;
use crate::scheduler::core::phase::Phase;
use crate::scheduler::core::phase::PhaseResult;
use crate::scheduler::core::state::State;
use crate::scheduler::orchestration::Task;
use crate::scheduler::traits::Executable;
use crate::scheduler::traits::PollStatus;
use crate::scheduler::traits::Runner;
use crate::scheduler::traits::YieldIntention;

/* STRUCTURES */

pub(in crate::scheduler) struct Collection<'a> {
    state: &'a mut State,
    runner: &'a mut dyn Runner,
}

/* IMPLEMENTATIONS */

impl<'a> Collection<'a> {
    pub(in crate::scheduler) fn new(
        state: &'a mut State,
        runner: &'a mut dyn Runner,
    ) -> Self {
        Self { state, runner }
    }

    fn collect_task(&mut self, id: TaskID) -> Result<bool> {
        match self.runner.poll(id)? {
            PollStatus::Pending => Ok(false),
            PollStatus::Ready(update) => {
                let executable = self.runner.collect(id)?;
                self.handle_ready(id, executable, update)?;
                Ok(true)
            },
            PollStatus::Panic(_msg) => {
                let executable = self.runner.collect(id)?;
                self.handle_panic(id, executable)?;
                Ok(true)
            },
        }
    }

    fn handle_ready(
        &mut self,
        id: TaskID,
        mut executable: Box<dyn Executable>,
        update: crate::scheduler::traits::YieldUpdate,
    ) -> Result<()> {
        if let Some(merge) = self.state.take_merge(&id) {
            executable
                .merge(merge.executable)
                .context("Failed to apply pending merge")?;
        }

        self.register_discovered(update.discovered)?;

        match update.intention {
            YieldIntention::Suspended(outcome) => {
                self.handle_suspended(id, executable, outcome)?;
            },
            YieldIntention::Waiting(deps) => {
                self.handle_waiting(id, executable, deps)?;
            },
            YieldIntention::Ready => {
                self.handle_restored(id, executable)?;
            },
        }

        Ok(())
    }

    fn handle_suspended(
        &mut self,
        id: TaskID,
        executable: Box<dyn Executable>,
        outcome: TaskOutcome,
    ) -> Result<()> {
        use crate::scheduler::core::context::Either;

        let ctx = self
            .state
            .remove(&id)
            .context("Task not in state")?
            .into_running_or_preempting()
            .context("Task not in Running or Preempting state")?;

        let suspended = match ctx {
            Either::Running(running) => running.complete(executable, outcome),
            Either::Preempting(preempting) => {
                preempting.complete(executable, outcome)
            },
        };

        self.state
            .insert(id, AnyContext::Suspended(suspended));

        Ok(())
    }

    fn handle_waiting(
        &mut self,
        id: TaskID,
        executable: Box<dyn Executable>,
        deps: HashSet<TaskID>,
    ) -> Result<()> {
        use crate::scheduler::core::context::Either;

        let ctx = self
            .state
            .remove(&id)
            .context("Task not in state")?
            .into_running_or_preempting()
            .context("Task not in Running or Preempting state")?;

        let waiting = match ctx {
            Either::Running(running) => running.wait(executable, deps),
            Either::Preempting(preempting) => preempting.wait(executable, deps),
        };

        self.state
            .insert(id, AnyContext::Waiting(waiting));

        self.validate_cycles()
            .context("Yield created dependency cycle")?;

        Ok(())
    }

    fn validate_cycles(&self) -> Result<()> {
        for (id, ctx) in self.state.iter() {
            if let AnyContext::Waiting(waiting) = ctx {
                for dep in waiting.dependencies() {
                    if self.has_cycle(*id, *dep) {
                        bail!("Dependency cycle detected");
                    }
                }
            }
        }

        Ok(())
    }

    fn has_cycle(&self, from: TaskID, to: TaskID) -> bool {
        if from == to {
            return true;
        }

        let mut visited = HashSet::new();
        let mut stack = vec![to];

        while let Some(current) = stack.pop() {
            if current == from {
                return true;
            }

            if visited.contains(&current) {
                continue;
            }

            visited.insert(current);

            if let Some(AnyContext::Waiting(waiting)) = self.state.get(&current)
            {
                for dep in waiting.dependencies() {
                    stack.push(*dep);
                }
            }
        }

        false
    }

    fn handle_restored(
        &mut self,
        id: TaskID,
        executable: Box<dyn Executable>,
    ) -> Result<()> {
        use crate::scheduler::core::context::Either;

        let ctx = self
            .state
            .remove(&id)
            .context("Task not in state")?
            .into_running_or_preempting()
            .context("Task not in Running or Preempting state")?;

        let ready = match ctx {
            Either::Running(running) => running.restore(executable),
            Either::Preempting(preempting) => preempting.restore(executable),
        };

        self.state
            .insert(id, AnyContext::Ready(ready));

        Ok(())
    }

    fn handle_panic(
        &mut self,
        id: TaskID,
        executable: Box<dyn Executable>,
    ) -> Result<()> {
        use crate::scheduler::core::context::Either;

        let ctx = self
            .state
            .remove(&id)
            .context("Task not in state")?
            .into_running_or_preempting()
            .context("Task not in Running or Preempting state")?;

        let error = match ctx {
            Either::Running(running) => running.crash(executable),
            Either::Preempting(preempting) => preempting.crash(executable),
        };

        self.state
            .insert(id, AnyContext::Error(error));

        Ok(())
    }

    fn register_discovered(&mut self, tasks: Vec<Task>) -> Result<()> {
        for task in tasks {
            self.register_task(task)?;
        }

        Ok(())
    }

    fn register_task(&mut self, task: Task) -> Result<()> {
        let id = task.id();

        if self.state.contains(&id) {
            if self.should_defer(&id) {
                self.defer_merge(id, task)?;
            } else {
                self.attempt_merge(id, task)?;
            }
        } else {
            self.register_new(task)?;
        }

        self.validate_cycles()
            .context("Registration created cycle")?;

        Ok(())
    }

    fn should_defer(&self, id: &TaskID) -> bool {
        self.state
            .get(id)
            .map(|ctx| !ctx.has_executable())
            .unwrap_or(false)
    }

    fn defer_merge(&mut self, id: TaskID, task: Task) -> Result<()> {
        let merge = self
            .state
            .take_merge(&id)
            .unwrap_or_else(|| {
                crate::scheduler::core::state::MergeContext::new(
                    task.executable,
                    task.dependencies,
                )
            });

        self.state.defer_merge(id, merge);

        Ok(())
    }

    fn attempt_merge(&mut self, id: TaskID, task: Task) -> Result<()> {
        let mut ctx = self
            .state
            .remove(&id)
            .context("Task not in state during merge")?;

        match &mut ctx {
            AnyContext::Ready(ready) => {
                ready
                    .merge_with(task.executable)
                    .context("Failed to merge Ready task")?;
            },
            AnyContext::Waiting(waiting) => {
                waiting
                    .merge_with(task.executable, task.dependencies)
                    .context("Failed to merge Waiting task")?;
            },
            AnyContext::Error(error) => {
                error
                    .merge_with(task.executable)
                    .context("Failed to merge Error task")?;
            },
            AnyContext::Suspended(_) => {
                self.state.insert(id, ctx);
                return Ok(());
            },
            AnyContext::Running(_) | AnyContext::Preempting(_) => {
                bail!("Attempted immediate merge on task without executable");
            },
        }

        self.state.insert(id, ctx);

        Ok(())
    }

    fn register_new(&mut self, task: Task) -> Result<()> {
        let id = task.id();

        let ctx = if task.dependencies.is_empty() {
            AnyContext::Ready(
                crate::scheduler::core::context::TaskContext::ready(
                    task.executable,
                    task.retriable,
                    task.about,
                    task.size,
                ),
            )
        } else {
            AnyContext::Waiting(
                crate::scheduler::core::context::TaskContext::waiting(
                    task.executable,
                    task.dependencies,
                    task.retriable,
                    task.about,
                    task.size,
                ),
            )
        };

        self.state.insert(id, ctx);

        Ok(())
    }
}

impl<'a> Phase for Collection<'a> {
    fn execute(&mut self) -> Result<PhaseResult> {
        let mut affected = 0;

        let running: Vec<TaskID> = self
            .state
            .running_ids()
            .chain(self.state.preempting_ids())
            .copied()
            .collect();

        for id in running {
            if self.collect_task(id)? {
                affected += 1;
            }
        }

        let changed = affected > 0;
        Ok(PhaseResult { changed, affected })
    }

    fn name(&self) -> &'static str {
        "Collection"
    }
}
