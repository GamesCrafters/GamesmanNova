//! # Scheduler Models
//!
//! TODO

use derive_builder::Builder;

use std::collections::HashMap;
use std::collections::HashSet;

use crate::traits::scheduler::Executable;
use crate::traits::scheduler::Logger;
use crate::traits::scheduler::Policy;
use crate::traits::scheduler::Runner;

/* SUBMODULES */

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

/* TYPE ALIASES */

/// Identifier for a logical piece of work (universally unique).
pub type TaskID = u64;

/// Integer encoding of the logical outcome of a task's dependency.
pub type OutcomeCode = u64;

/// IDs of all tasks which must be completed before another task.
pub type Dependencies = HashSet<TaskID>;

/// Logical outcomes of a set of tasks.  
pub type TaskOutcomes = HashMap<TaskID, TaskOutcome>;

/// Collection of unique tasks indexed by ID.
pub type TaskBuffer = HashMap<TaskID, Box<dyn Executable>>;

/// Metadata of unique tasks indexed by ID.
pub type TaskRegistry = HashMap<TaskID, TaskContext>;

/* ENUMERATIONS */

/// The logical outcome of a task.
#[derive(Clone, Debug)]
pub enum TaskOutcome {
    Success(OutcomeCode),
    Failure(OutcomeCode),
    Error,
}

/// The logical progress of a task.
pub enum TaskState {
    Finished(TaskOutcome),
    Waiting(Dependencies),
    Preempting,
    Running,
    Error,
    Ready,
}

/// Treatment of a task that just yielded its worker.
pub enum YieldIntention {
    Finished(TaskOutcome),
    Waiting(Dependencies),
    Ready,
}

/* STRUCTURES */

/// Update provided by a task upon yielding or being preempted. Any information
/// included about another existing task (through `discovered`) is ignored.
pub struct YieldUpdate {
    pub intention: YieldIntention,
    pub discovered: Vec<Task>,
}

/// Status returned by the runner when polling a task.
pub enum PollStatus {
    /// Task is still executing, not ready to collect yet.
    Pending,
    /// Task has completed (yielded, finished, or was preempted) and is ready to collect.
    Ready(YieldUpdate),
    /// Task executable panicked (internal task failure).
    Panic(String),
}

/// The information needed to register a new task.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct Task {
    /* Mandatory fields */
    pub executable: Box<dyn Executable>,
    pub retriable: bool,
    pub tid: TaskID,

    /* Defaults provided */
    #[builder(default)]
    pub requires: Dependencies,

    #[builder(default)]
    pub about: String,

    #[builder(default)]
    pub size: Option<u64>,
}

/// Scheduling metadata. One-to-one basis with seen tasks.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct TaskContext {
    pub retriable: bool,
    pub incoming: Dependencies,
    pub progress: TaskState,
    pub about: String,
    pub size: Option<u64>,
}

/// All abstract scheduler components.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct SchedulerContext {
    pub policy: Box<dyn Policy>,
    pub logger: Box<dyn Logger>,
    pub runner: Box<dyn Runner>,
}

/// State considered for scheduling decisions.
#[derive(Default)]
pub struct SchedulerState {
    pub registry: TaskRegistry,
    pub buffer: TaskBuffer,
    pub ticks: u64,
    pub units: usize,
}

/// Generic task-recursive scheduler.
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct Scheduler {
    pub context: SchedulerContext,
    pub state: SchedulerState,
}

/// Task size statistics across the task registry.
#[derive(Clone, Copy)]
pub struct SizeStats {
    pub stddev: f64,
    pub mean: f64,
}
