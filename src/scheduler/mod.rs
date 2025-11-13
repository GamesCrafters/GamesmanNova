//! # Nova Scheduler
//!
//! Cooperative task scheduler for DAG execution using a 5-phase
//! tick-based system.
//!
//! ## Architecture
//!
//! Each scheduler tick executes 5 phases sequentially:
//! 1. Collection - Poll runners, collect completed tasks
//! 2. Resolution - Execute satisfied Waiting tasks directly
//! 3. Retry - Move Error tasks back to Ready (policy-driven)
//! 4. Preemption - Signal running tasks to stop (policy-driven)
//! 5. Execution - Execute Ready tasks (policy-driven)

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use derive_builder::Builder;

use std::collections::HashMap;
use std::collections::HashSet;

use crate::scheduler::traits::Executable;
use crate::scheduler::traits::Logger;
use crate::scheduler::traits::Policy;
use crate::scheduler::traits::Runner;
use crate::scheduler::util::find_cycle_path;
use crate::scheduler::util::format_cycle_path;

/* API RE-EXPORTS */

pub use logger::compose::ComposedLogger;
pub use logger::count::CountLogger;
pub use logger::dashboard::DashboardLogger;
pub use logger::history::HistoryLogger;

pub use policy::critical::CriticalPathPolicy;
pub use policy::critical::RetryPolicy;
pub use policy::trivial::TrivialPolicy;

pub use runner::sync::SyncRunner;
pub use runner::thread::ThreadPoolConfig;
pub use runner::thread::ThreadPoolRunner;

/* SUBMODULES */

mod util;
mod traits;
mod logger {
    pub mod dashboard;
    pub mod history;
    pub mod compose;
    pub mod count;
}

mod policy {
    pub mod critical;
    pub mod trivial;
}

mod runner {
    pub mod sync;
    pub mod thread;
}

mod task {
    #[cfg(test)]
    pub mod mock;
    pub mod explore;
    pub mod solve;
    pub mod store;
}

/* TYPE ALIASES */

type TaskID = u64;
type OutcomeCode = u64;
type Dependencies = HashSet<TaskID>;
type TaskOutcomes = HashMap<TaskID, TaskOutcome>;
type TaskRegistry = HashMap<TaskID, TaskContext>;
type MergeRegistry = HashMap<TaskID, MergeContext>;

/* ENUMERATIONS */

#[derive(Clone, Copy, Debug)]
enum TaskOutcome {
    Success(OutcomeCode),
    Failure(OutcomeCode),
    Error,
}

#[derive(Clone, Debug)]
enum TaskState {
    Suspended(TaskOutcome),
    Waiting(Dependencies),
    Preempting,
    Running,
    Error,
    Ready,
}

struct MergeContext {
    executable: Box<dyn Executable>,
    dependencies: Dependencies,
}

#[derive(Debug)]
enum YieldIntention {
    Suspended(TaskOutcome),
    Waiting(Dependencies),
    Ready,
}

enum PollStatus {
    Ready(YieldUpdate),
    Panic(String),
    Pending,
}

#[derive(Clone, Copy, Debug)]
enum Phase {
    Collection,
    Resolution,
    Preemption,
    Execution,
    Retry,
}

/* API STRUCTURES */

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct Task {
    executable: Box<dyn Executable>,
    retriable: bool,
    tid: TaskID,

    #[builder(default)]
    requires: Dependencies,

    #[builder(default)]
    about: String,

    #[builder(default)]
    size: Option<u64>,
}

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct SchedulerContext {
    #[builder(setter(custom))]
    policy: Box<dyn Policy>,

    #[builder(setter(custom))]
    logger: Box<dyn Logger>,

    #[builder(setter(custom))]
    runner: Box<dyn Runner>,
}

#[derive(Default, Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct SchedulerState {
    #[builder(default)]
    #[builder(setter(custom))]
    buffer: TaskRegistry,

    #[builder(default)]
    #[builder(setter(skip))]
    merges: MergeRegistry,

    #[builder(default)]
    #[builder(setter(skip))]
    ticks: u64,
}

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct Scheduler {
    context: SchedulerContext,
    state: SchedulerState,

    #[builder(default)]
    #[builder(setter(skip))]
    transitions: Vec<Transition>,
}

/* PRIVATE STRUCTURES */

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
struct YieldUpdate {
    intention: YieldIntention,

    #[builder(setter(each = "found"))]
    discovered: Vec<Task>,
}

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
struct TaskContext {
    executable: Option<Box<dyn Executable>>,
    retriable: bool,
    incoming: Dependencies,
    progress: Option<u64>,
    state: TaskState,
    about: String,
    size: Option<u64>,
}

#[derive(Clone, Copy)]
struct SizeStats {
    stddev: f64,
    mean: f64,
}

#[derive(Clone)]
struct SchedulerSnapshot {
    transitions: Vec<Transition>,
    tasks: HashMap<TaskID, TaskContextSnapshot>,
    tick: u64,
}

#[derive(Clone)]
struct TaskContextSnapshot {
    retriable: bool,
    progress: Option<u64>,
    incoming: Dependencies,
    state: TaskState,
    about: String,
    size: Option<u64>,
}

#[derive(Clone, Debug)]
struct Transition {
    phase: Phase,
    task: TaskID,
    from: TaskState,
    to: TaskState,
}

/// Restricted view of scheduler state for policy decisions.
/// Contains only tasks valid for the current decision type.
struct DecisionContext<'a> {
    candidates: HashMap<TaskID, &'a TaskContext>,
    buffer: &'a TaskRegistry,
    units: Option<usize>,
    ticks: u64,
}

/* SCHEDULER IMPLEMENTATION */

impl Scheduler {
    /* TASK REGISTRATION */

    /// Register a new task with the scheduler.
    ///
    /// If a task with the same ID already exists, the behavior depends on
    /// the existing task's state:
    ///
    /// - **Offshore tasks** (missing executable): The merge is deferred by
    ///   storing the new task in the merge registry. When the offshore task
    ///   returns, pending merges are applied via `merge_pending()`.
    ///
    /// - **Other tasks**: Attempts immediate merge via `attempt_merge()`,
    ///   which merges executables and combines dependencies.
    ///
    /// After registration/merge, validates that no dependency cycles were
    /// introduced.
    ///
    /// # Errors
    /// - Merge fails if executables are incompatible
    /// - Validation fails if a dependency cycle is detected
    fn register(&mut self, task: Task) -> Result<&mut Self> {
        if self
            .state
            .buffer
            .contains_key(&task.tid)
        {
            if self.should_defer(&task.tid) {
                self.defer_merge(task.tid, task.executable, task.requires)
                    .context("Failed to defer merge on offshore task")?;
            } else {
                self.attempt_merge(task.tid, task)
                    .context("Failed to merge discovery with existing task")?;
            }
        } else {
            self.register_new(task)
                .context("Failed to register new task")?;
        }

        self.ensure_acyclic()
            .context("Found task cycle among scheduler tasks")?;

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
            .map(|ctx| ctx.missing_executable())
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
            existing
                .merge_into(context)
                .context("Failed to combine pending merge contexts")?;
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
            let _changed = self
                .tick()
                .context("Scheduler failed during an execution tick")?;
        }

        Ok(())
    }

    /// Execute one pass of work phases.
    pub fn tick(&mut self) -> Result<bool> {
        self.transitions.clear();

        self.collect_phase()
            .context("Scheduler collect phase failed")?;

        self.resolve_phase()
            .context("Scheduler resolution phase failed")?;

        self.restart_phase()
            .context("Scheduler restart phase failed")?;

        self.preempt_phase()
            .context("Scheduler preempt phase failed")?;

        self.execute_phase()
            .context("Scheduler execute phase failed")?;

        self.update_progress()
            .context("Scheduler failed to update task progress")?;

        let changed = !self.transitions.is_empty();
        let snapshot = self.snapshot();
        self.context
            .logger
            .observe(&snapshot, changed)
            .context("Scheduler failed to invoke logger component")?;

        self.state.ticks += 1;
        Ok(changed)
    }

    /* COLLECTION PHASE */

    fn collect_phase(&mut self) -> Result<()> {
        let tasks = self
            .state
            .collect_runner_task_ids();

        for tid in tasks {
            self.collect_task(tid)?;
        }

        Ok(())
    }

    /// Poll and collect a single task from the runner.
    ///
    /// Handles three possible outcomes:
    /// - **Pending**: Task still running, no action taken
    /// - **Ready**: Task completed with yield update, processes the update
    ///   via `handle_yield()` and applies any pending merges
    /// - **Panic**: Task panicked, transitions to Error state
    ///
    /// When a task is Ready, this method:
    /// 1. Collects the executable from the runner
    /// 2. Applies any pending merges via `finalize_collection()`
    /// 3. Handles the yield intention (Ready/Waiting/Suspended)
    /// 4. Registers any newly discovered tasks
    fn collect_task(&mut self, tid: TaskID) -> Result<()> {
        match self
            .context
            .runner
            .poll(tid)
            .context("Failed to poll runner for task status")?
        {
            PollStatus::Pending => Ok(()),
            PollStatus::Ready(update) => {
                let executable = self
                    .context
                    .runner
                    .collect(tid)
                    .context("Failed to collect ready task from runner")?;

                let pending_deps = self.finalize_collection(tid, executable)?;
                self.handle_yield(tid, update, pending_deps)?;
                Ok(())
            },
            PollStatus::Panic(_) => {
                let executable = self
                    .context
                    .runner
                    .collect(tid)
                    .context("Failed to collect panicked task from runner")?;

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
            .context("Collected non-existent task")?
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

    /* RESOLUTION PHASE */

    fn resolve_phase(&mut self) -> Result<()> {
        while !self.at_capacity() {
            let ctx = DecisionContext::for_resolution(
                &self.state,
                self.context.runner.capacity(),
            );

            if ctx.candidates.is_empty() {
                break;
            }

            let Some(tid) = self.context.policy.execute(&ctx) else {
                break;
            };

            if !ctx.candidates.contains_key(&tid) {
                bail!(
                    "Policy selected non-candidate task {} in resolution",
                    tid
                );
            }

            let executable = self
                .state
                .buffer
                .get_mut(&tid)
                .context("Task not in registry")?
                .executable
                .take()
                .context("Waiting task has no executable")?;

            let awaited = self.collect_awaited(tid)?;
            self.transition(tid, TaskState::Running, Phase::Resolution)?;
            self.context
                .runner
                .execute(tid, awaited, executable)?;
        }

        Ok(())
    }

    /* RETRY PHASE */

    fn restart_phase(&mut self) -> Result<()> {
        while let Some(tid) = {
            let ctx = DecisionContext::for_retry(
                &self.state,
                self.context.runner.capacity(),
            );
            self.context.policy.retry(&ctx)
        } {
            let ctx = DecisionContext::for_retry(
                &self.state,
                self.context.runner.capacity(),
            );
            if !ctx.candidates.contains_key(&tid) {
                bail!(
                    "Policy selected non-candidate task {} in retry",
                    tid
                );
            }

            self.transition(tid, TaskState::Ready, Phase::Retry)?;
        }

        Ok(())
    }

    /* PREEMPTION PHASE */

    fn preempt_phase(&mut self) -> Result<()> {
        while let Some(tid) = {
            let ctx = DecisionContext::for_preemption(
                &self.state,
                self.context.runner.capacity(),
            );
            self.context.policy.preempt(&ctx)
        } {
            let ctx = DecisionContext::for_preemption(
                &self.state,
                self.context.runner.capacity(),
            );
            if !ctx.candidates.contains_key(&tid) {
                bail!(
                    "Policy selected non-candidate task {} in preemption",
                    tid
                );
            }

            self.context.runner.preempt(tid)?;
            self.transition(tid, TaskState::Preempting, Phase::Preemption)?;
        }

        Ok(())
    }

    /* EXECUTION PHASE */

    fn execute_phase(&mut self) -> Result<()> {
        while !self.at_capacity() {
            let tid = {
                let ctx = DecisionContext::for_execution(
                    &self.state,
                    self.context.runner.capacity(),
                );
                self.context.policy.execute(&ctx)
            };

            let Some(tid) = tid else {
                break;
            };

            let ctx = DecisionContext::for_execution(
                &self.state,
                self.context.runner.capacity(),
            );

            if !ctx.candidates.contains_key(&tid) {
                bail!(
                    "Policy selected non-candidate task {} in execution",
                    tid
                );
            }

            let task = self
                .state
                .buffer
                .get_mut(&tid)
                .context("Fetched non-existent task from registry")?
                .executable
                .take()
                .context("Task has no executable to execute")?;

            let awaited = self.collect_awaited(tid)?;
            self.transition(tid, TaskState::Running, Phase::Execution)?;
            self.context
                .runner
                .execute(tid, awaited, task)?;
        }

        Ok(())
    }

    /* YIELD HANDLING */

    /// Process a task's yield update and transition to appropriate state.
    ///
    /// Coordinates the task's state transition based on yield intention:
    /// - **Suspended(outcome)**: Task completed, to Suspended state
    /// - **Waiting(deps)**: Task needs dependencies, to Waiting state
    /// - **Ready**: Task ready to run again, to Ready or Waiting state
    ///
    /// Also handles:
    /// - Registering any newly discovered tasks from the yield update
    /// - Merging yielded dependencies with pending dependencies from
    ///   registration
    /// - Validating no cycles were introduced by new dependencies
    ///
    /// # Errors
    /// - Returns error if task yields TaskOutcome::Error (invalid)
    /// - Returns error if new dependencies create a cycle
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

        self.register_discovered(update.discovered)
            .context("Failed to register newly discovered tasks")?;

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

        Ok(())
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

        self.ensure_acyclic()
            .context("Yielded (waiting) task created deadlock")?;

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

            self.ensure_acyclic()
                .context("Yielded (ready) task created deadlock")?;
        }

        Ok(())
    }

    fn register_discovered(&mut self, tasks: Vec<Task>) -> Result<()> {
        for task in tasks {
            self.register(task)
                .context("Failed to register discovered task")?;
        }

        Ok(())
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

        let from = std::mem::replace(&mut ctx.state, state);
        let to = ctx.state.clone();

        let transition = Transition {
            task: tid,
            from,
            to,
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
        let tasks = self
            .state
            .collect_runner_task_ids();
        for tid in tasks {
            if let Some(value) = self.context.runner.progress(tid) {
                let ctx = self
                    .state
                    .buffer
                    .get_mut(&tid)
                    .context("Task not found in registry")?;

                ctx.progress = Some(value);
            }
        }

        Ok(())
    }

    fn snapshot(&mut self) -> SchedulerSnapshot {
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
            transitions: std::mem::take(&mut self.transitions),
            tick: self.state.ticks,
            tasks,
        }
    }

    fn update_size(&mut self, tid: TaskID) -> Result<()> {
        let ctx = self
            .state
            .buffer
            .get_mut(&tid)
            .context("Task not found in registry")?;

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

    /// Collect outcomes from all dependencies a task is waiting for.
    ///
    /// Extracts the outcomes (Success/Failure) from all tasks that this task
    /// depends on. Only returns outcomes for suspended/completed dependencies;
    /// filters out any dependencies still in progress.
    ///
    /// # Returns
    /// Map of dependency TaskIDs to their TaskOutcomes, ready to be passed
    /// to the task's executable when it runs.
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
                .map(|outcome| (dep_tid, *outcome));

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

    /// Check if scheduler has reached execution unit capacity.
    /// Returns false if capacity is None (unlimited).
    fn at_capacity(&self) -> bool {
        self.context
            .runner
            .capacity()
            .map(|limit| self.state.runner_tasks().count() >= limit)
            .unwrap_or(false)
    }

    fn ensure_acyclic(&self) -> Result<()> {
        let Some(path) = find_cycle_path(&self.state.buffer) else {
            return Ok(());
        };

        let message = format_cycle_path(&path, &self.state.buffer);
        bail!(message)
    }
}
