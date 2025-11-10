//! # Scheduler Implementations
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use derive_builder::Builder;

use std::collections::HashMap;
use std::collections::HashSet;

use crate::core::scheduler::utils::find_cycle_path;
use crate::traits::scheduler::Executable;
use crate::traits::scheduler::Logger;
use crate::traits::scheduler::Policy;
use crate::traits::scheduler::Runner;

/* SUBMODULES */

mod utils;

pub mod logger {
    pub mod count;
    pub mod history;
    pub mod compose;
    pub mod tui;
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
    pub mod explore;
    pub mod solve;
    pub mod store;
}

/* TYPE ALIASES */

pub type TaskID = u64;
pub type OutcomeCode = u64;
pub type Dependencies = HashSet<TaskID>;
pub type TaskOutcomes = HashMap<TaskID, TaskOutcome>;
pub type TaskRegistry = HashMap<TaskID, TaskContext>;
pub type MergeRegistry = HashMap<TaskID, MergeContext>;

/* ENUMERATIONS */

#[derive(Clone, Debug)]
pub enum TaskOutcome {
    Success(OutcomeCode),
    Failure(OutcomeCode),
    Error,
}

#[derive(Clone, Debug)]
pub enum TaskState {
    Suspended(TaskOutcome),
    Waiting(Dependencies),
    Preempting,
    Running,
    Error,
    Ready,
}

pub struct MergeContext {
    executable: Box<dyn Executable>,
    dependencies: Dependencies,
}

pub enum YieldIntention {
    Suspended(TaskOutcome),
    Waiting(Dependencies),
    Ready,
}

pub enum PollStatus {
    Ready(YieldUpdate),
    Panic(String),
    Pending,
}

/* STRUCTURES */

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct YieldUpdate {
    pub intention: YieldIntention,

    #[builder(setter(each = "found"))]
    pub discovered: Vec<Task>,
}

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct Task {
    pub executable: Box<dyn Executable>,
    pub retriable: bool,
    pub tid: TaskID,

    #[builder(default)]
    pub requires: Dependencies,

    #[builder(default)]
    pub about: String,

    #[builder(default)]
    pub size: Option<u64>,
}

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct TaskContext {
    pub executable: Option<Box<dyn Executable>>,
    pub retriable: bool,
    pub incoming: Dependencies,
    pub progress: Option<u64>,
    pub state: TaskState,
    pub about: String,
    pub size: Option<u64>,
}

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct SchedulerContext {
    pub policy: Box<dyn Policy>,
    pub logger: Box<dyn Logger>,
    pub runner: Box<dyn Runner>,
}

#[derive(Default)]
pub struct SchedulerState {
    pub merges: MergeRegistry,
    pub buffer: TaskRegistry,
    pub units: usize,
    pub ticks: u64,
}

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct Scheduler {
    pub context: SchedulerContext,
    pub state: SchedulerState,

    #[builder(default)]
    #[builder(setter(skip))]
    transitions: Vec<Transition>,
}

#[derive(Clone, Copy)]
pub struct SizeStats {
    pub stddev: f64,
    pub mean: f64,
}

#[derive(Clone)]
pub struct SchedulerSnapshot {
    pub tick: u64,
    pub tasks: HashMap<TaskID, TaskContextSnapshot>,
    pub transitions: Vec<Transition>,
}

#[derive(Clone)]
pub struct TaskContextSnapshot {
    pub retriable: bool,
    pub incoming: Dependencies,
    pub state: TaskState,
    pub about: String,
    pub size: Option<u64>,
    pub progress: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct Transition {
    pub task: TaskID,
    pub from: TaskState,
    pub to: TaskState,
    pub phase: Phase,
}

#[derive(Clone, Copy, Debug)]
pub enum Phase {
    Collection,
    Retry,
    Preemption,
    Execution,
}

/* IMPLEMENTATIONS */

impl Scheduler {
    /// Create a scheduler in its own universe.
    pub fn new(context: SchedulerContext, mut state: SchedulerState) -> Self {
        state.units = context.runner.units();
        Self {
            context,
            state,
            transitions: Vec::new(),
        }
    }

    /* TASK REGISTRATION */

    /// Register a new task with the scheduler. If a task with same ID already
    /// exists, merges the new task with the existing one.
    pub fn register(&mut self, task: Task) -> Result<&mut Self> {
        if self
            .state
            .buffer
            .contains_key(&task.tid)
        {
            if self.should_defer(&task.tid) {
                self.defer_merge(task.tid, task.executable, task.requires)?;
            } else {
                self.attempt_merge(task.tid, task)?;
            }
        } else {
            self.register_new(task)?;
        }

        self.ensure_acyclic()?;
        Ok(self)
    }

    fn register_new(&mut self, task: Task) -> Result<()> {
        let state = if task.requires.is_empty() {
            TaskState::Ready
        } else {
            TaskState::Waiting(task.requires.clone())
        };

        let ctx = TaskContext {
            executable: Some(task.executable),
            retriable: task.retriable,
            incoming: Dependencies::new(),
            progress: None,
            about: task.about,
            size: task.size,
            state,
        };

        self.state
            .buffer
            .insert(task.tid, ctx);

        if !task.requires.is_empty() {
            self.link_dependencies(task.tid, &task.requires)?;
        }

        Ok(())
    }

    fn should_defer(&self, tid: &TaskID) -> bool {
        self.state
            .buffer
            .get(tid)
            .map(|ctx| ctx.executable.is_none())
            .unwrap_or(false)
    }

    fn attempt_merge(&mut self, tid: TaskID, task: Task) -> Result<()> {
        let ctx = self
            .state
            .buffer
            .get_mut(&tid)
            .context("Task not in registry")?;

        let existing = ctx
            .executable
            .as_mut()
            .context("Cannot merge with currently running task.")?;

        existing
            .merge(task.executable)
            .context("Failed to merge task executables")?;

        let old_deps = ctx
            .dependencies()
            .cloned()
            .unwrap_or_default();

        let new_deps: Dependencies = old_deps
            .union(&task.requires)
            .copied()
            .collect();

        self.relink_dependencies(tid, &old_deps, &new_deps)?;
        if new_deps.is_empty() {
            self.set_state(tid, TaskState::Ready)?;
        } else {
            self.set_state(tid, TaskState::Waiting(new_deps))?;
        }

        Ok(())
    }

    fn defer_merge(
        &mut self,
        tid: TaskID,
        executable: Box<dyn Executable>,
        dependencies: Dependencies,
    ) -> Result<()> {
        let context = MergeContext {
            dependencies,
            executable,
        };

        if let Some(existing) = self.state.merges.get_mut(&tid) {
            existing.merge_into(context)?;
        } else {
            self.state
                .merges
                .insert(tid, context);
        }

        Ok(())
    }

    /* EXECUTION LOOP */

    /// Loop the scheduler until there are no active tasks.
    pub fn run(&mut self) -> Result<()> {
        while self
            .state
            .buffer
            .values()
            .any(|ctx| ctx.active())
        {
            let _changed = self.tick()?;
        }

        Ok(())
    }

    /// Execute one pass of work phases.
    pub fn tick(&mut self) -> Result<bool> {
        self.transitions.clear();

        self.collect_phase()?;
        self.restart_phase()?;
        self.preempt_phase()?;
        self.execute_phase()?;

        self.update_progress()?;

        let changed = !self.transitions.is_empty();
        let snapshot = self.snapshot();
        self.context
            .logger
            .observe(&snapshot, changed)?;

        self.state.ticks += 1;
        Ok(changed)
    }

    /* COLLECTION PHASE */

    fn collect_phase(&mut self) -> Result<()> {
        let executing = self.state.runner_tasks();
        let tasks: Vec<TaskID> = executing
            .map(|(tid, _)| *tid)
            .collect();

        for tid in tasks {
            self.collect_task(tid)?;
        }

        Ok(())
    }

    fn collect_task(&mut self, tid: TaskID) -> Result<()> {
        match self.context.runner.poll(tid)? {
            PollStatus::Pending => Ok(()),
            PollStatus::Ready(update) => {
                let executable = self.context.runner.collect(tid)?;
                let pending_deps = self.finalize_collection(tid, executable)?;
                self.handle_yield(tid, update, pending_deps)?;
                Ok(())
            },
            PollStatus::Panic(_) => {
                let executable = self.context.runner.collect(tid)?;
                self.finalize_collection(tid, executable)?;
                self.transition(tid, TaskState::Error, Phase::Collection)?;
                Ok(())
            },
        }
    }

    fn finalize_collection(
        &mut self,
        tid: TaskID,
        executable: Box<dyn Executable>,
    ) -> Result<Option<Dependencies>> {
        let context = self.merge_pending(tid, executable)?;
        self.state
            .buffer
            .get_mut(&tid)
            .context("Task not in registry")?
            .executable = Some(context.executable);

        self.update_size(tid)?;
        let pending_deps = if context.dependencies.is_empty() {
            None
        } else {
            Some(context.dependencies)
        };

        Ok(pending_deps)
    }

    fn merge_pending(
        &mut self,
        tid: TaskID,
        mut executable: Box<dyn Executable>,
    ) -> Result<MergeContext> {
        if let Some(mut context) = self.state.take_merge(&tid) {
            executable
                .merge(context.executable)
                .context("Failed to apply pending merge")?;

            context.executable = executable;
            Ok(context)
        } else {
            Ok(MergeContext {
                dependencies: Dependencies::new(),
                executable,
            })
        }
    }

    /* RETRY PHASE */

    fn restart_phase(&mut self) -> Result<()> {
        while let Some(tid) = self
            .context
            .policy
            .retry(&self.state)
        {
            self.transition(tid, TaskState::Ready, Phase::Retry)?;
        }

        Ok(())
    }

    /* PREEMPTION PHASE */

    fn preempt_phase(&mut self) -> Result<()> {
        while let Some(tid) = self
            .context
            .policy
            .preempt(&self.state)
        {
            self.context.runner.preempt(tid)?;
            self.transition(tid, TaskState::Preempting, Phase::Preemption)?;
        }

        Ok(())
    }

    /* EXECUTION PHASE */

    fn execute_phase(&mut self) -> Result<()> {
        while let Some(tid) = self
            .context
            .policy
            .execute(&self.state)
        {
            let task = self
                .state
                .buffer
                .get_mut(&tid)
                .context("Fetched non-existing task from registry.")?
                .executable
                .take()
                .context("Task has no executable to execute.")?;

            let awaited = self.collect_awaited(tid)?;
            self.transition(tid, TaskState::Running, Phase::Execution)?;
            self.context
                .runner
                .execute(tid, awaited, task)?;
        }

        Ok(())
    }

    /* YIELD HANDLING */

    fn handle_yield(
        &mut self,
        tid: TaskID,
        update: YieldUpdate,
        pending_deps: Option<Dependencies>,
    ) -> Result<()> {
        if matches!(
            update.intention,
            YieldIntention::Suspended(TaskOutcome::Error)
        ) {
            bail!("Task {} returned TaskOutcome::Error", tid);
        }

        match update.intention {
            YieldIntention::Suspended(outcome) => {
                self.handle_suspension(tid, outcome)?;
            },
            YieldIntention::Waiting(yielded_deps) => {
                self.handle_waiting(tid, yielded_deps, pending_deps)?;
            },
            YieldIntention::Ready => {
                self.handle_ready(tid, pending_deps)?;
            },
        }

        self.register_discovered(update.discovered)
    }

    fn handle_suspension(
        &mut self,
        tid: TaskID,
        outcome: TaskOutcome,
    ) -> Result<()> {
        if let Some(deps) = self
            .state
            .get_dependencies(tid)
            .cloned()
        {
            self.unlink_dependencies(tid, &deps);
        }

        self.transition(
            tid,
            TaskState::Suspended(outcome),
            Phase::Collection,
        )?;
        Ok(())
    }

    fn handle_waiting(
        &mut self,
        tid: TaskID,
        yielded_deps: Dependencies,
        pending_deps: Option<Dependencies>,
    ) -> Result<()> {
        let old_deps = self
            .state
            .get_dependencies(tid)
            .cloned()
            .unwrap_or_default();

        let new_deps = if let Some(pending_deps) = pending_deps {
            yielded_deps
                .union(&pending_deps)
                .copied()
                .collect()
        } else {
            yielded_deps
        };

        self.relink_dependencies(tid, &old_deps, &new_deps)?;
        self.transition(
            tid,
            TaskState::Waiting(new_deps),
            Phase::Collection,
        )?;
        self.ensure_acyclic()?;
        Ok(())
    }

    fn handle_ready(
        &mut self,
        tid: TaskID,
        pending_deps: Option<Dependencies>,
    ) -> Result<()> {
        let new_deps = pending_deps.unwrap_or_default();
        if new_deps.is_empty() {
            self.transition(tid, TaskState::Ready, Phase::Collection)?;
        } else {
            let old_deps = self
                .state
                .get_dependencies(tid)
                .cloned()
                .unwrap_or_default();

            self.relink_dependencies(tid, &old_deps, &new_deps)?;
            self.transition(
                tid,
                TaskState::Waiting(new_deps),
                Phase::Collection,
            )?;
            self.ensure_acyclic()?;
        }

        Ok(())
    }

    fn register_discovered(&mut self, tasks: Vec<Task>) -> Result<()> {
        let register = |task| {
            self.register(task)
                .context("Failed to register discovered task")
                .map(|_| ())
        };

        tasks
            .into_iter()
            .try_for_each(register)
    }

    /* STATE MANAGEMENT */

    fn transition(
        &mut self,
        tid: TaskID,
        state: TaskState,
        phase: Phase,
    ) -> Result<()> {
        let ctx = self
            .state
            .buffer
            .get_mut(&tid)
            .context("Task not in registry")?;

        let from = ctx.state.clone();
        ctx.state = state.clone();

        let transition = Transition {
            task: tid,
            from,
            to: state,
            phase,
        };

        self.transitions.push(transition);
        Ok(())
    }

    fn set_state(&mut self, tid: TaskID, progress: TaskState) -> Result<()> {
        self.state
            .buffer
            .get_mut(&tid)
            .context("Task not in registry")?
            .state = progress;

        Ok(())
    }

    fn update_progress(&mut self) -> Result<()> {
        let running = self.state.runner_tasks();
        let tasks: Vec<TaskID> = running
            .map(|(tid, _)| *tid)
            .collect();

        for tid in tasks {
            if let Some(value) = self.context.runner.progress(tid) {
                let ctx = self
                    .state
                    .buffer
                    .get_mut(&tid)
                    .context("Task not in registry")?;
                ctx.progress = Some(value);
            }
        }

        Ok(())
    }

    fn snapshot(&self) -> SchedulerSnapshot {
        let convert = |(tid, ctx): (&TaskID, &TaskContext)| {
            let snapshot = TaskContextSnapshot {
                retriable: ctx.retriable,
                incoming: ctx.incoming.clone(),
                state: ctx.state.clone(),
                about: ctx.about.clone(),
                size: ctx.size,
                progress: ctx.progress,
            };
            (*tid, snapshot)
        };

        let tasks = self
            .state
            .buffer
            .iter()
            .map(convert)
            .collect();

        SchedulerSnapshot {
            transitions: self.transitions.clone(),
            tick: self.state.ticks,
            tasks,
        }
    }

    fn update_size(&mut self, tid: TaskID) -> Result<()> {
        let ctx = self
            .state
            .buffer
            .get_mut(&tid)
            .context("Task not in registry")?;

        let Some(ref executable) = ctx.executable else {
            return Ok(());
        };

        let Some(size) = executable.size() else {
            return Ok(());
        };

        ctx.size = Some(size);
        Ok(())
    }

    /* DEPENDENCY MANAGEMENT */

    fn link_dependencies(
        &mut self,
        source: TaskID,
        targets: &Dependencies,
    ) -> Result<()> {
        let link = |target: &TaskID| {
            self.state
                .buffer
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
                .buffer
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
        let added: Dependencies = new
            .difference(old)
            .copied()
            .collect();

        let removed: Dependencies = old
            .difference(new)
            .copied()
            .collect();

        self.unlink_dependencies(source, &removed);
        self.link_dependencies(source, &added)?;
        Ok(())
    }

    fn collect_awaited(&self, tid: TaskID) -> Result<TaskOutcomes> {
        let dependencies = self
            .state
            .get_dependencies(tid)
            .cloned()
            .unwrap_or_default();

        let extract = |dep_tid| {
            let dep_ctx = self
                .state
                .buffer
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
        let Some(path) = find_cycle_path(&self.state.buffer) else {
            return Ok(());
        };

        let message = self.format_cycle_error(&path);
        bail!(message)
    }

    fn format_cycle_error(&self, path: &[TaskID]) -> String {
        let mut message = format!(
            "These {} tasks wait for each other cyclically:\n",
            path.len() - 1
        );

        for tid in path {
            if let Some(ctx) = self.state.buffer.get(tid) {
                message.push_str(&format!("-> {:?}: {}\n", tid, ctx.about));
            }
        }

        message
    }
}
