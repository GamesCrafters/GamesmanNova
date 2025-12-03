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

use derive_builder::Builder;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use crate::game::Component;

/* API RE-EXPORTS */

pub use orchestration::Orchestrator;
pub use orchestration::TaskBuilder;

pub use logger::compose::ComposeLoggerBuilder;
pub use logger::dashboard::DashboardLoggerBuilder;
pub use logger::dashboard::SortOrder;
pub use logger::dashboard::TaskFilter;
pub use logger::tracing::TracingLoggerBuilder;

pub use policy::critical::CriticalPathPolicyBuilder;
pub use policy::trivial::TrivialPolicy;

pub use runner::sync::SyncRunner;
pub use runner::thread::ThreadPoolRunnerBuilder;

pub use task::backward::BackwardTask;
pub use task::forward::ForwardTaskBuilder;
pub use task::tabulate::TabulateTask;

/* SUBMODULES */

pub mod core;
mod orchestration;
mod traits;
mod util;

mod logger {
    pub mod compose;
    pub mod dashboard;
    pub mod history;
    pub mod tracing;
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
    pub mod backward;
    pub mod forward;
    #[cfg(test)]
    pub mod mock;
    pub mod tabulate;
}

/* TYPE ALIASES */

pub type TaskOutcomes = HashMap<TaskID, TaskOutcome>;
pub type Dependencies = HashSet<TaskID>;
type OutcomeCode = u64;

/* ENUMERATIONS */

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum TaskCategory {
    Explore,
    Solve,
    Store,
    Flush,
    Mock,
}

/* STRUCTURES */

#[derive(Builder, Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct TaskID {
    component: Component,
    category: TaskCategory,
}

impl Display for TaskID {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:?}-{}", self.category, self.component)
    }
}

/* TASK ENUMERATIONS */

#[derive(Clone, Copy, Debug)]
pub enum TaskOutcome {
    Success(OutcomeCode),
    Failure(OutcomeCode),
    Error,
}

pub(in crate::scheduler) enum DispatchOutcome {
    Accepted,
    CapacityExhausted(Box<dyn traits::Executable>),
}

impl Debug for DispatchOutcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Accepted => write!(f, "Accepted"),
            Self::CapacityExhausted(_) => write!(f, "CapacityExhausted"),
        }
    }
}

/* SNAPSHOT TYPES */

#[derive(Clone, Debug)]
pub struct SchedulerSnapshot {
    pub tasks: HashMap<TaskID, TaskContextSnapshot>,
    pub tick: u64,
    pub runner: Option<RunnerSnapshot>,
    pub policy: Option<PolicySnapshot>,
    pub transitions: Vec<Transition>,
}

#[derive(Clone, Debug)]
pub struct Transition {
    pub task: TaskID,
    pub from: TaskState,
    pub to: TaskState,
    pub phase: String,
}

#[derive(Clone, Debug)]
pub struct RunnerSnapshot {
    pub capacity: Option<usize>,
    pub ticks: Vec<u64>,
}

#[derive(Clone, Debug)]
pub struct PolicySnapshot {
    pub decisions: Vec<PolicyDecision>,
}

#[derive(Clone, Debug)]
pub struct TaskContextSnapshot {
    pub progress: Option<u64>,
    pub size: Option<u64>,
    pub state: TaskState,
    pub category: TaskCategory,
    pub component: Component,
    pub about: String,
}

#[derive(Clone, Debug)]
pub enum TaskState {
    Ready,
    Running,
    Waiting(Dependencies),
    Preempting,
    Suspended(TaskOutcome),
    Error,
}

#[derive(Clone, Debug)]
pub struct PolicyDecision {
    pub action: PolicyAction,
    pub weight: Option<u64>,
    pub task: TaskID,
}

#[derive(Clone, Copy, Debug)]
pub enum PolicyAction {
    Execute,
    Preempt,
    Retry,
}
