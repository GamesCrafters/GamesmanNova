//! # Scheduler Implementations
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;

use crate::types::scheduler::Dependencies;
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

pub mod logger;
pub mod runner;
pub mod policy;

/* IMPLEMENTATIONS */

impl Scheduler {
    /// Create a scheduler in its own universe.
    pub fn new(context: SchedulerContext, state: SchedulerState) -> Self {
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
            self.set_progress(task.tid, TaskState::Waiting(requires))?;
        }

        self.ensure_acyclic()?;
        Ok(self)
    }

    /// Loop the scheduler until there are no active tasks.
    pub async fn run(&mut self) -> Result<()> {
        while self
            .state
            .registry
            .values()
            .any(|ctx| ctx.active())
        {
            self.tick().await?;
        }

        Ok(())
    }

    /// Execute one pass of retry policy phases and log changes.
    async fn tick(&mut self) -> Result<()> {
        let changed = [
            self.collect_phase().await?,
            self.retry_phase()?,
            self.preempt_phase().await?,
            self.execute_phase().await?,
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

    fn retry_phase(&mut self) -> Result<bool> {
        let mut changed = false;
        while let Some(tid) = self
            .context
            .policy
            .retry(&self.state)
        {
            self.set_progress(tid, TaskState::Ready)?;
            changed = true;
        }

        Ok(changed)
    }

    async fn preempt_phase(&mut self) -> Result<bool> {
        let mut changed = false;
        while let Some(tid) = self
            .context
            .policy
            .preempt(&self.state)
        {
            self.set_progress(tid, TaskState::Preempting)?;
            changed = true;
        }

        Ok(changed)
    }

    async fn collect_phase(&mut self) -> Result<bool> {
        let mut changed = false;
        let tasks: Vec<TaskID> = self
            .state
            .registry
            .iter()
            .filter_map(|(tid, ctx)| {
                matches!(
                    ctx.progress,
                    TaskState::Running | TaskState::Preempting
                )
                .then_some(*tid)
            })
            .collect();

        for tid in tasks {
            let Some(result) = self.context.runner.poll(tid) else {
                continue;
            };

            let task = self
                .context
                .runner
                .collect(tid)
                .await?;

            self.state.buffer.insert(tid, task);
            self.update_size(tid)?;

            match result {
                Ok(update) => self.reabsorb(tid, update).await?,
                Err(_) => self.set_progress(tid, TaskState::Error)?,
            }

            changed = true;
        }

        Ok(changed)
    }

    async fn execute_phase(&mut self) -> Result<bool> {
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

            let outcomes = self.collect_outcomes(tid)?;
            self.set_progress(tid, TaskState::Running)?;
            self.context
                .runner
                .execute(tid, task, outcomes)
                .await?;

            changed = true;
        }

        Ok(changed)
    }

    /* TASK YIELDING */

    async fn reabsorb(
        &mut self,
        tid: TaskID,
        update: YieldUpdate,
    ) -> Result<()> {
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

                self.set_progress(tid, TaskState::Finished(outcome))?;
                self.state.buffer.remove(&tid);
            },
            YieldIntention::Waiting(new_deps) => {
                let old_deps = self
                    .state
                    .get_dependencies(tid)
                    .cloned()
                    .unwrap_or_default();

                self.relink_dependencies(tid, &old_deps, &new_deps)?;
                self.set_progress(tid, TaskState::Waiting(new_deps))?;
                self.ensure_acyclic()?;
            },
            YieldIntention::Ready => {
                self.set_progress(tid, TaskState::Ready)?;
            },
        }

        for task in update.discovered {
            self.register(task)
                .context("Failed to register discovered task")?;
        }

        Ok(())
    }

    /* STATE MANIPULATION */

    fn set_progress(&mut self, tid: TaskID, progress: TaskState) -> Result<()> {
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
        targets
            .iter()
            .try_for_each(|target| {
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
            })
    }

    fn unlink_dependencies(&mut self, source: TaskID, targets: &Dependencies) {
        targets.iter().for_each(|target| {
            if let Some(ctx) = self.state.registry.get_mut(target) {
                ctx.incoming.remove(&source);
            }
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

        added
            .iter()
            .try_for_each(|target| {
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
            })?;

        removed.iter().for_each(|target| {
            if let Some(ctx) = self.state.registry.get_mut(target) {
                ctx.incoming.remove(&source);
            }
        });

        Ok(())
    }

    /* DEPENDENCY OUTCOMES */

    fn collect_outcomes(&self, tid: TaskID) -> Result<TaskOutcomes> {
        let deps = self
            .state
            .get_dependencies(tid)
            .cloned()
            .unwrap_or_default();

        let mut outcomes = TaskOutcomes::new();
        for dep_tid in deps {
            let dep_ctx = self
                .state
                .registry
                .get(&dep_tid)
                .context(format!(
                    "Dependency {} not found in registry",
                    dep_tid
                ))?;

            if let TaskState::Finished(outcome) = &dep_ctx.progress {
                outcomes.insert(dep_tid, outcome.clone());
            }
        }

        Ok(outcomes)
    }

    /* VALIDATION */

    fn ensure_acyclic(&self) -> Result<()> {
        if let Some(path) = find_cycle_path(&self.state.registry) {
            let base = format!(
                "These {} tasks wait for each other cyclically:\n",
                path.len() - 1
            );

            let message = path
                .iter()
                .filter_map(|tid| {
                    self.state
                        .registry
                        .get(tid)
                        .map(|ctx| (tid, ctx))
                })
                .fold(base, |mut msg, (tid, ctx)| {
                    msg.push_str(&format!("-> {:?}: {}\n", tid, ctx.about));
                    msg
                });

            bail!(message);
        }

        Ok(())
    }
}
