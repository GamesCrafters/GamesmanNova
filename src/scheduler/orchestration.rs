//! # Scheduler Orchestration
//!
//! Coordinates execution of 5-phase tick system using core types.

use anyhow::Context;
use anyhow::Result;

use std::collections::HashSet;

use crate::scheduler::TaskID;
use crate::scheduler::core;
use crate::scheduler::core::Phase;
use crate::scheduler::traits::Executable;
use crate::scheduler::traits::Logger;
use crate::scheduler::traits::Policy;
use crate::scheduler::traits::Runner;

/* TYPE ALIASES */

pub(crate) type Deps = HashSet<TaskID>;

/* STRUCTURES */

/// Task registration data.
pub struct Task {
    pub(super) executable: Box<dyn Executable>,
    pub(super) dependencies: Deps,
    pub(super) retriable: bool,
    pub(super) about: String,
    pub(super) size: Option<u64>,
}

impl Task {
    pub fn builder() -> TaskBuilder {
        TaskBuilder::default()
    }

    pub(crate) fn id(&self) -> TaskID {
        self.executable.id()
    }
}

#[derive(Default)]
pub struct TaskBuilder {
    executable: Option<Box<dyn Executable>>,
    dependencies: Option<Deps>,
    retriable: Option<bool>,
    about: Option<String>,
    size: Option<u64>,
}

impl TaskBuilder {
    #[allow(private_bounds)]
    pub fn executable(mut self, executable: impl Executable + 'static) -> Self {
        self.executable = Some(Box::new(executable));
        self
    }

    pub fn dependencies(mut self, dependencies: Deps) -> Self {
        self.dependencies = Some(dependencies);
        self
    }

    pub fn retriable(mut self, retriable: bool) -> Self {
        self.retriable = Some(retriable);
        self
    }

    pub fn about(mut self, about: impl Into<String>) -> Self {
        self.about = Some(about.into());
        self
    }

    pub fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn build(self) -> Result<Task> {
        let executable = self
            .executable
            .context("executable required")?;
        let dependencies = self
            .dependencies
            .unwrap_or_default();
        let retriable = self.retriable.unwrap_or(false);
        let about = self.about.unwrap_or_default();
        let size = self.size;

        Ok(Task {
            executable,
            dependencies,
            retriable,
            about,
            size,
        })
    }
}

/* ORCHESTRATOR */

/// Scheduler orchestrator using new core types.
pub struct Orchestrator {
    state: core::State,
    registry: core::Registry,
    runner: Box<dyn Runner>,
    policy: Box<dyn Policy>,
    logger: Box<dyn Logger>,
}

/* IMPLEMENTATIONS */

impl Orchestrator {
    pub fn builder() -> OrchestratorBuilder {
        OrchestratorBuilder::default()
    }

    pub fn register(&mut self, task: Task) -> Result<&mut Self> {
        let id = task.id();

        if self.state.contains(&id) {
            return Ok(self);
        }

        let ctx = if task.dependencies.is_empty() {
            core::AnyContext::Ready(core::TaskContext::ready(
                task.executable,
                task.retriable,
                task.about,
                task.size,
            ))
        } else {
            core::AnyContext::Waiting(core::TaskContext::waiting(
                task.executable,
                task.dependencies,
                task.retriable,
                task.about,
                task.size,
            ))
        };

        self.state.insert(id, ctx);
        self.registry
            .validate()
            .context("Registration created cycle")?;

        Ok(self)
    }

    pub fn run(&mut self) -> Result<()> {
        while self.state.active() {
            self.tick()
                .context("Scheduler tick failed")?;
        }

        Ok(())
    }

    pub fn tick(&mut self) -> Result<bool> {
        let capacity = self
            .runner
            .capacity()
            .unwrap_or(usize::MAX);

        let mut changed = false;
        changed |= self.run_collection()?;
        changed |= self.run_resolution(capacity)?;
        changed |= self.run_retry(capacity)?;
        changed |= self.run_preemption(capacity)?;
        changed |= self.run_execution(capacity)?;

        self.state.increment();
        let snapshot = self
            .state
            .snapshot(self.runner.as_mut(), self.policy.as_ref());

        self.logger
            .report(&snapshot, changed)
            .context("Logger report failed")?;

        Ok(changed)
    }

    fn run_collection(&mut self) -> Result<bool> {
        let mut phase = core::phase::collection::Collection::new(
            &mut self.state,
            self.runner.as_mut(),
        );

        let result = phase
            .execute()
            .context("Collection phase failed")?;

        Ok(result.changed)
    }

    fn run_resolution(&mut self, capacity: usize) -> Result<bool> {
        let mut phase = core::phase::resolution::Resolution::new(
            &mut self.state,
            self.runner.as_mut(),
            self.policy.as_mut(),
            capacity,
        );

        let result = phase
            .execute()
            .context("Resolution phase failed")?;

        Ok(result.changed)
    }

    fn run_retry(&mut self, _capacity: usize) -> Result<bool> {
        let mut phase = core::phase::retry::Retry::new(
            &mut self.state,
            self.policy.as_mut(),
        );

        let result = phase
            .execute()
            .context("Retry phase failed")?;

        Ok(result.changed)
    }

    fn run_preemption(&mut self, capacity: usize) -> Result<bool> {
        let mut phase = core::phase::preemption::Preemption::new(
            &mut self.state,
            self.runner.as_mut(),
            self.policy.as_mut(),
            capacity,
        );

        let result = phase
            .execute()
            .context("Preemption phase failed")?;

        Ok(result.changed)
    }

    fn run_execution(&mut self, capacity: usize) -> Result<bool> {
        let mut phase = core::phase::execution::Execution::new(
            &mut self.state,
            self.runner.as_mut(),
            self.policy.as_mut(),
            capacity,
        );

        let result = phase
            .execute()
            .context("Execution phase failed")?;

        Ok(result.changed)
    }
}

/* BUILDER */

#[derive(Default)]
pub struct OrchestratorBuilder {
    runner: Option<Box<dyn Runner>>,
    policy: Option<Box<dyn Policy>>,
    logger: Option<Box<dyn Logger>>,
}

impl OrchestratorBuilder {
    #[allow(private_bounds)]
    pub fn runner(mut self, runner: impl Runner + 'static) -> Self {
        self.runner = Some(Box::new(runner));
        self
    }

    #[allow(private_bounds)]
    pub fn policy(mut self, policy: impl Policy + 'static) -> Self {
        self.policy = Some(Box::new(policy));
        self
    }

    #[allow(private_bounds)]
    pub fn logger(mut self, logger: impl Logger + 'static) -> Self {
        self.logger = Some(Box::new(logger));
        self
    }

    pub fn build(self) -> Result<Orchestrator> {
        let runner = self
            .runner
            .context("runner required")?;
        let policy = self
            .policy
            .context("policy required")?;
        let logger = self
            .logger
            .context("logger required")?;

        Ok(Orchestrator {
            state: core::State::new(),
            registry: core::Registry::new(),
            runner,
            policy,
            logger,
        })
    }
}
