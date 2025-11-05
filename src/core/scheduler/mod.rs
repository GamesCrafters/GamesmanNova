//! # Scheduler Implementations
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;

use crate::types::scheduler::Dependencies;
use crate::types::scheduler::PollStatus;
use crate::types::scheduler::Scheduler;
use crate::types::scheduler::SchedulerContext;
use crate::types::scheduler::SchedulerState;
use crate::types::scheduler::Task;
use crate::types::scheduler::TaskContext;
use crate::types::scheduler::TaskID;
use crate::types::scheduler::TaskOutcome;
use crate::types::scheduler::TaskOutcomes;
use crate::types::scheduler::TaskState;
use crate::types::scheduler::YieldIntention;
use crate::types::scheduler::YieldUpdate;

use utils::find_cycle_path;

/* SUBMODULES */

mod utils;

pub mod logger {
    pub mod count;
}

pub mod policy {
    pub mod critical;
    pub mod trivial;
}

pub mod runner {
    pub mod sync;
    pub mod thread;
}

pub mod task {
    #[cfg(test)]
    pub mod mock;
}

/* IMPLEMENTATIONS */

impl Scheduler {
    /// Create a scheduler in its own universe.
    pub fn new(context: SchedulerContext, mut state: SchedulerState) -> Self {
        state.units = context.runner.units();
        Self { context, state }
    }

    /// Register a new task with the shceduler.
    pub fn register(&mut self, task: Task) -> Result<&mut Self> {
        let requires = task.requires;
        let ctx = TaskContext {
            retriable: task.retriable,
            about: task.about,
            progress: TaskState::Ready,
            incoming: Dependencies::new(),
            size: task.size,
        };

        self.state
            .buffer
            .insert(task.tid, task.executable);

        self.state
            .registry
            .insert(task.tid, ctx);

        if !requires.is_empty() {
            self.link_dependencies(task.tid, &requires)?;
            self.set_state(task.tid, TaskState::Waiting(requires))?;
        }

        self.ensure_acyclic()?;
        Ok(self)
    }

    /// Loop the scheduler until there are no active tasks.
    pub fn run(&mut self) -> Result<()> {
        while self
            .state
            .registry
            .values()
            .any(|ctx| ctx.active())
        {
            self.tick()?;
        }

        Ok(())
    }

    fn tick(&mut self) -> Result<()> {
        let changed = [
            self.collect_phase()?,
            self.retries_phase()?,
            self.preempt_phase()?,
            self.execute_phase()?,
        ];

        self.state.ticks += 1;
        if changed.contains(&true) {
            self.context
                .logger
                .log(&self.state)?;
        }

        Ok(())
    }

    /* TICK PHASES */

    fn collect_phase(&mut self) -> Result<bool> {
        let executing = self.state.runner_tasks();
        let tasks: Vec<TaskID> = executing
            .map(|(tid, _)| *tid)
            .collect();

        let mut changed = false;
        for tid in tasks {
            changed |= self.collect_task(tid)?;
        }

        Ok(changed)
    }

    fn retries_phase(&mut self) -> Result<bool> {
        let mut changed = false;
        while let Some(tid) = self
            .context
            .policy
            .retry(&self.state)
        {
            self.set_state(tid, TaskState::Ready)?;
            changed = true;
        }

        Ok(changed)
    }

    fn preempt_phase(&mut self) -> Result<bool> {
        let mut changed = false;
        while let Some(tid) = self
            .context
            .policy
            .preempt(&self.state)
        {
            self.context.runner.preempt(tid)?;
            self.set_state(tid, TaskState::Preempting)?;
            changed = true;
        }

        Ok(changed)
    }

    fn execute_phase(&mut self) -> Result<bool> {
        let mut changed = false;
        while let Some(tid) = self
            .context
            .policy
            .execute(&self.state)
        {
            let task = self
                .state
                .buffer
                .remove(&tid)
                .context("Attempted to fetch non-existing task from buffer.")?;

            let awaited = self.collect_awaited(tid)?;
            self.set_state(tid, TaskState::Running)?;
            self.context
                .runner
                .execute(tid, awaited, task)?;

            changed = true;
        }

        Ok(changed)
    }

    /* TASK COLLECTION */

    fn collect_task(&mut self, tid: TaskID) -> Result<bool> {
        match self.context.runner.poll(tid)? {
            PollStatus::Pending => Ok(false),
            PollStatus::Ready(update) => {
                let executable = self.context.runner.collect(tid)?;
                self.state
                    .buffer
                    .insert(tid, executable);

                self.update_size(tid)?;
                self.handle_yield(tid, update)?;
                Ok(true)
            },
            PollStatus::Panic(_) => {
                let executable = self.context.runner.collect(tid)?;
                self.state
                    .buffer
                    .insert(tid, executable);

                self.update_size(tid)?;
                self.set_state(tid, TaskState::Error)?;
                Ok(true)
            },
        }
    }

    fn handle_yield(&mut self, tid: TaskID, update: YieldUpdate) -> Result<()> {
        match update.intention {
            YieldIntention::Finished(TaskOutcome::Error) => {
                bail!("Task {} returned TaskOutcome::Error", tid);
            },
            YieldIntention::Finished(outcome) => {
                if let Some(deps) = self
                    .state
                    .get_dependencies(tid)
                    .cloned()
                {
                    self.unlink_dependencies(tid, &deps);
                }

                self.set_state(tid, TaskState::Finished(outcome))?;
                self.state.buffer.remove(&tid);
            },
            YieldIntention::Waiting(new_deps) => {
                let old_deps = self
                    .state
                    .get_dependencies(tid)
                    .cloned()
                    .unwrap_or_default();

                self.relink_dependencies(tid, &old_deps, &new_deps)?;
                self.set_state(tid, TaskState::Waiting(new_deps))?;
                self.ensure_acyclic()?;
            },
            YieldIntention::Ready => {
                self.set_state(tid, TaskState::Ready)?;
            },
        }

        let register = |task| {
            self.register(task)
                .context("Failed to register discovered task")
                .map(|_| ())
        };

        update
            .discovered
            .into_iter()
            .try_for_each(register)
    }

    /* STATE MANIPULATION */

    fn set_state(&mut self, tid: TaskID, progress: TaskState) -> Result<()> {
        self.state
            .registry
            .get_mut(&tid)
            .context("Task not in registry")?
            .progress = progress;

        Ok(())
    }

    fn update_size(&mut self, tid: TaskID) -> Result<()> {
        let task = self
            .state
            .buffer
            .get(&tid)
            .context("Task not in buffer")?;

        let Some(size) = task.size() else {
            return Ok(());
        };

        let ctx = self
            .state
            .registry
            .get_mut(&tid)
            .context("Task not in registry")?;

        if ctx.size == Some(size) {
            return Ok(());
        }

        ctx.size = Some(size);
        Ok(())
    }

    fn link_dependencies(
        &mut self,
        source: TaskID,
        targets: &Dependencies,
    ) -> Result<()> {
        let link = |target: &TaskID| {
            self.state
                .registry
                .get_mut(target)
                .context(format!(
                    "Task {} depends on non-existent task {}",
                    source, target
                ))
                .map(|ctx| {
                    ctx.incoming.insert(source);
                })
        };

        targets.iter().try_for_each(link)
    }

    fn unlink_dependencies(&mut self, source: TaskID, targets: &Dependencies) {
        targets.iter().for_each(|target| {
            self.state
                .registry
                .get_mut(target)
                .map(|ctx| ctx.incoming.remove(&source));
        });
    }

    fn relink_dependencies(
        &mut self,
        source: TaskID,
        old: &Dependencies,
        new: &Dependencies,
    ) -> Result<()> {
        let added: Vec<TaskID> = new
            .difference(old)
            .copied()
            .collect();

        let removed: Vec<TaskID> = old
            .difference(new)
            .copied()
            .collect();

        let link = |target: &TaskID| {
            self.state
                .registry
                .get_mut(target)
                .context(format!(
                    "Task {} updated to depend on non-existent task {}",
                    source, target
                ))
                .map(|ctx| {
                    ctx.incoming.insert(source);
                })
        };

        added.iter().try_for_each(link)?;
        let unlink = |target: &TaskID| {
            if let Some(ctx) = self.state.registry.get_mut(target) {
                ctx.incoming.remove(&source);
            }
        };

        removed.iter().for_each(unlink);
        Ok(())
    }

    /* DEPENDENCY OUTCOMES */

    fn collect_awaited(&self, tid: TaskID) -> Result<TaskOutcomes> {
        let dependencies = self
            .state
            .get_dependencies(tid)
            .cloned()
            .unwrap_or_default();

        let extract = |dep_tid| {
            let dep_ctx = self
                .state
                .registry
                .get(&dep_tid)
                .context(format!(
                    "Dependency {} not found in registry",
                    dep_tid
                ))?;

            let result = dep_ctx
                .outcome()
                .map(|outcome| (dep_tid, outcome.clone()));

            Ok(result)
        };

        let outcomes = dependencies
            .into_iter()
            .map(extract)
            .collect::<Result<Vec<_>>>()?;

        let awaited = outcomes
            .into_iter()
            .flatten()
            .collect();

        Ok(awaited)
    }

    /* VALIDATION */

    fn ensure_acyclic(&self) -> Result<()> {
        if let Some(path) = find_cycle_path(&self.state.registry) {
            let base = format!(
                "These {} tasks wait for each other cyclically:\n",
                path.len() - 1
            );

            let format =
                |mut msg: String, (tid, ctx): (&TaskID, &TaskContext)| {
                    msg.push_str(&format!("-> {:?}: {}\n", tid, ctx.about));
                    msg
                };

            let contexts = path.iter().filter_map(|tid| {
                self.state
                    .registry
                    .get(tid)
                    .map(|ctx| (tid, ctx))
            });

            let message = contexts.fold(base, format);
            bail!(message);
        }

        Ok(())
    }
}
