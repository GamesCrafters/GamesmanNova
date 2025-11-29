//! # Task Context with Type-State Safety
//!
//! Compile-time enforcement of valid state transitions using phantom
//! types. Each state is a distinct type - illegal transitions won't
//! compile.

use anyhow::Context as _;
use anyhow::Result;
use anyhow::bail;

use std::collections::HashSet;
use std::marker::PhantomData;

use crate::scheduler::TaskID;
use crate::scheduler::TaskOutcome;
use crate::scheduler::traits::Executable;

/* TYPE ALIASES */

type Deps = HashSet<TaskID>;

/* ENUMERATIONS */

/// Zero-sized state markers for type-state pattern.
pub struct Preempting;
pub struct Suspended;
pub struct Waiting;
pub struct Running;
pub struct Error;
pub struct Ready;

/// Helper for handling tasks that can be in either Running or Preempting state.
pub enum Either<L, R> {
    Running(L),
    Preempting(R),
}

/// Type-erased context for HashMap storage.
pub enum AnyContext {
    Suspended(TaskContext<Suspended>),
    Preempting(TaskContext<Preempting>),
    Waiting(TaskContext<Waiting>),
    Running(TaskContext<Running>),
    Error(TaskContext<Error>),
    Ready(TaskContext<Ready>),
}

/* STRUCTURES */

/// Task context parameterized by state.
pub struct TaskContext<S> {
    executable: Option<Box<dyn Executable>>,
    dependencies: Option<Deps>,
    outcome: Option<TaskOutcome>,
    retriable: bool,
    progress: Option<u64>,
    about: String,
    size: Option<u64>,
    _state: PhantomData<S>,
}

/* IMPLEMENTATIONS */

/* CONSTRUCTORS */

impl TaskContext<Ready> {
    pub(in crate::scheduler) fn ready(
        executable: Box<dyn Executable>,
        retriable: bool,
        about: String,
        size: Option<u64>,
    ) -> Self {
        Self {
            executable: Some(executable),
            dependencies: None,
            outcome: None,
            retriable,
            progress: None,
            about,
            size,
            _state: PhantomData,
        }
    }
}

impl TaskContext<Waiting> {
    pub(in crate::scheduler) fn waiting(
        executable: Box<dyn Executable>,
        dependencies: Deps,
        retriable: bool,
        about: String,
        size: Option<u64>,
    ) -> Self {
        Self {
            executable: Some(executable),
            dependencies: Some(dependencies),
            outcome: None,
            retriable,
            progress: None,
            about,
            size,
            _state: PhantomData,
        }
    }
}

/* ACCESSORS */

impl<S> TaskContext<S> {
    pub fn retriable(&self) -> bool {
        self.retriable
    }

    pub fn progress(&self) -> Option<u64> {
        self.progress
    }

    pub fn about(&self) -> &str {
        &self.about
    }

    pub fn size(&self) -> Option<u64> {
        self.size
    }

    pub fn has_executable(&self) -> bool {
        self.executable.is_some()
    }
}

/* STATE ACCESSORS */

impl TaskContext<Waiting> {
    pub fn dependencies(&self) -> &Deps {
        self.dependencies
            .as_ref()
            .expect("Waiting must have dependencies")
    }
}

impl TaskContext<Suspended> {
    pub fn outcome(&self) -> &TaskOutcome {
        self.outcome
            .as_ref()
            .expect("Suspended must have outcome")
    }
}

/* READY TRANSITIONS */

impl TaskContext<Ready> {
    /// Transition: Ready -> Running
    /// Takes executable, prepares for runner dispatch.
    pub(in crate::scheduler) fn dispatch(
        mut self,
    ) -> (TaskContext<Running>, Box<dyn Executable>) {
        let exec = self
            .executable
            .take()
            .expect("Ready must have executable");

        let running = TaskContext {
            executable: None,
            dependencies: None,
            outcome: None,
            retriable: self.retriable,
            progress: self.progress,
            about: self.about,
            size: self.size,
            _state: PhantomData,
        };

        (running, exec)
    }

    /// Transition: Ready -> Waiting
    /// Task discovered dependencies before execution.
    pub fn defer(self, dependencies: Deps) -> TaskContext<Waiting> {
        TaskContext {
            executable: self.executable,
            dependencies: Some(dependencies),
            outcome: None,
            retriable: self.retriable,
            progress: self.progress,
            about: self.about,
            size: self.size,
            _state: PhantomData,
        }
    }

    /// Merge another executable into this Ready task.
    pub(in crate::scheduler) fn merge_with(
        &mut self,
        incoming: Box<dyn Executable>,
    ) -> anyhow::Result<()> {
        let existing = self
            .executable
            .as_mut()
            .context("Ready task missing executable")?;

        existing.merge(incoming)?;
        Ok(())
    }
}

/* RUNNING TRANSITIONS */

impl TaskContext<Running> {
    /// Transition: Running -> Preempting
    /// Signal sent to runner, waiting for cooperative yield.
    pub fn preempt(self) -> TaskContext<Preempting> {
        TaskContext {
            executable: self.executable,
            dependencies: None,
            outcome: None,
            retriable: self.retriable,
            progress: self.progress,
            about: self.about,
            size: self.size,
            _state: PhantomData,
        }
    }

    /// Transition: Running -> Ready
    /// Task yielded, ready to continue execution.
    pub(in crate::scheduler) fn restore(
        mut self,
        executable: Box<dyn Executable>,
    ) -> TaskContext<Ready> {
        self.executable = Some(executable);
        TaskContext {
            executable: self.executable,
            dependencies: None,
            outcome: None,
            retriable: self.retriable,
            progress: self.progress,
            about: self.about,
            size: self.size,
            _state: PhantomData,
        }
    }

    /// Transition: Running -> Waiting
    /// Task yielded with new dependencies.
    pub(in crate::scheduler) fn wait(
        mut self,
        executable: Box<dyn Executable>,
        dependencies: Deps,
    ) -> TaskContext<Waiting> {
        self.executable = Some(executable);
        TaskContext {
            executable: self.executable,
            dependencies: Some(dependencies),
            outcome: None,
            retriable: self.retriable,
            progress: self.progress,
            about: self.about,
            size: self.size,
            _state: PhantomData,
        }
    }

    /// Transition: Running -> Suspended
    /// Task completed successfully.
    pub(in crate::scheduler) fn complete(
        mut self,
        executable: Box<dyn Executable>,
        outcome: TaskOutcome,
    ) -> TaskContext<Suspended> {
        self.executable = Some(executable);
        TaskContext {
            executable: self.executable,
            dependencies: None,
            outcome: Some(outcome),
            retriable: self.retriable,
            progress: self.progress,
            about: self.about,
            size: self.size,
            _state: PhantomData,
        }
    }

    /// Transition: Running -> Error
    /// Task panicked during execution.
    pub(in crate::scheduler) fn crash(
        mut self,
        executable: Box<dyn Executable>,
    ) -> TaskContext<Error> {
        self.executable = Some(executable);
        TaskContext {
            executable: self.executable,
            dependencies: None,
            outcome: None,
            retriable: self.retriable,
            progress: self.progress,
            about: self.about,
            size: self.size,
            _state: PhantomData,
        }
    }
}

/* PREEMPTING TRANSITIONS */

impl TaskContext<Preempting> {
    /// Transition: Preempting -> Ready
    /// Task yielded after preemption signal.
    pub(in crate::scheduler) fn restore(
        mut self,
        executable: Box<dyn Executable>,
    ) -> TaskContext<Ready> {
        self.executable = Some(executable);
        TaskContext {
            executable: self.executable,
            dependencies: None,
            outcome: None,
            retriable: self.retriable,
            progress: self.progress,
            about: self.about,
            size: self.size,
            _state: PhantomData,
        }
    }

    /// Transition: Preempting -> Waiting
    /// Task yielded with new dependencies after preemption.
    pub(in crate::scheduler) fn wait(
        mut self,
        executable: Box<dyn Executable>,
        dependencies: Deps,
    ) -> TaskContext<Waiting> {
        self.executable = Some(executable);
        TaskContext {
            executable: self.executable,
            dependencies: Some(dependencies),
            outcome: None,
            retriable: self.retriable,
            progress: self.progress,
            about: self.about,
            size: self.size,
            _state: PhantomData,
        }
    }

    /// Transition: Preempting -> Suspended
    /// Task completed after preemption signal (rare but valid).
    pub(in crate::scheduler) fn complete(
        mut self,
        executable: Box<dyn Executable>,
        outcome: TaskOutcome,
    ) -> TaskContext<Suspended> {
        self.executable = Some(executable);
        TaskContext {
            executable: self.executable,
            dependencies: None,
            outcome: Some(outcome),
            retriable: self.retriable,
            progress: self.progress,
            about: self.about,
            size: self.size,
            _state: PhantomData,
        }
    }

    /// Transition: Preempting -> Error
    /// Task panicked after preemption signal.
    pub(in crate::scheduler) fn crash(
        mut self,
        executable: Box<dyn Executable>,
    ) -> TaskContext<Error> {
        self.executable = Some(executable);
        TaskContext {
            executable: self.executable,
            dependencies: None,
            outcome: None,
            retriable: self.retriable,
            progress: self.progress,
            about: self.about,
            size: self.size,
            _state: PhantomData,
        }
    }
}

/* WAITING TRANSITIONS */

impl TaskContext<Waiting> {
    /// Transition: Waiting -> Running
    /// Immediate execution in resolution phase.
    pub(in crate::scheduler) fn resolve(
        mut self,
    ) -> (TaskContext<Running>, Box<dyn Executable>) {
        let exec = self
            .executable
            .take()
            .expect("Waiting must have executable");

        let running = TaskContext {
            executable: None,
            dependencies: None,
            outcome: None,
            retriable: self.retriable,
            progress: self.progress,
            about: self.about,
            size: self.size,
            _state: PhantomData,
        };

        (running, exec)
    }

    /// Merge another executable into this Waiting task and union dependencies.
    pub(in crate::scheduler) fn merge_with(
        &mut self,
        incoming: Box<dyn Executable>,
        incoming_deps: Deps,
    ) -> anyhow::Result<()> {
        let existing = self
            .executable
            .as_mut()
            .context("Waiting task missing executable")?;

        existing.merge(incoming)?;

        if let Some(deps) = &mut self.dependencies {
            deps.extend(incoming_deps);
        }

        Ok(())
    }
}

/* ERROR TRANSITIONS */

impl TaskContext<Error> {
    /// Transition: Error -> Ready
    /// Retry after failure.
    pub fn retry(self) -> TaskContext<Ready> {
        TaskContext {
            executable: self.executable,
            dependencies: None,
            outcome: None,
            retriable: self.retriable,
            progress: self.progress,
            about: self.about,
            size: self.size,
            _state: PhantomData,
        }
    }

    /// Merge another executable into this Error task.
    pub(in crate::scheduler) fn merge_with(
        &mut self,
        incoming: Box<dyn Executable>,
    ) -> anyhow::Result<()> {
        let existing = self
            .executable
            .as_mut()
            .context("Error task missing executable")?;

        existing.merge(incoming)?;
        Ok(())
    }
}

/* ANYCONTEXT OPERATIONS */

impl AnyContext {
    pub fn into_ready(self) -> Result<TaskContext<Ready>> {
        match self {
            AnyContext::Ready(ctx) => Ok(ctx),
            _ => bail!("Task not in Ready state"),
        }
    }

    pub fn into_running(self) -> Result<TaskContext<Running>> {
        match self {
            AnyContext::Running(ctx) => Ok(ctx),
            _ => bail!("Task not in Running state"),
        }
    }

    pub fn into_waiting(self) -> Result<TaskContext<Waiting>> {
        match self {
            AnyContext::Waiting(ctx) => Ok(ctx),
            _ => bail!("Task not in Waiting state"),
        }
    }

    pub fn into_preempting(self) -> Result<TaskContext<Preempting>> {
        match self {
            AnyContext::Preempting(ctx) => Ok(ctx),
            _ => bail!("Task not in Preempting state"),
        }
    }

    pub fn into_error(self) -> Result<TaskContext<Error>> {
        match self {
            AnyContext::Error(ctx) => Ok(ctx),
            _ => bail!("Task not in Error state"),
        }
    }

    pub fn into_running_or_preempting(
        self,
    ) -> Result<Either<TaskContext<Running>, TaskContext<Preempting>>> {
        match self {
            AnyContext::Running(ctx) => Ok(Either::Running(ctx)),
            AnyContext::Preempting(ctx) => Ok(Either::Preempting(ctx)),
            _ => bail!("Task not in Running or Preempting state"),
        }
    }

    pub fn state_name(&self) -> &'static str {
        match self {
            AnyContext::Suspended(_) => "Suspended",
            AnyContext::Preempting(_) => "Preempting",
            AnyContext::Waiting(_) => "Waiting",
            AnyContext::Running(_) => "Running",
            AnyContext::Error(_) => "Error",
            AnyContext::Ready(_) => "Ready",
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(
            self,
            AnyContext::Suspended(_) | AnyContext::Error(_)
        )
    }

    pub fn has_executable(&self) -> bool {
        match self {
            AnyContext::Ready(c) => c.has_executable(),
            AnyContext::Running(c) => c.has_executable(),
            AnyContext::Waiting(c) => c.has_executable(),
            AnyContext::Preempting(c) => c.has_executable(),
            AnyContext::Error(c) => c.has_executable(),
            AnyContext::Suspended(c) => c.has_executable(),
        }
    }
}
