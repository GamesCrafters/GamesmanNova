//! # Thread Pool Runner
//!
//! Concurrent runner that executes tasks across a pool of worker threads.

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use derive_builder::Builder;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::traits::scheduler::Executable;
use crate::types::scheduler::TaskID;
use crate::types::scheduler::TaskOutcomes;
use crate::types::scheduler::YieldUpdate;

/* ENUMERATIONS */

/// Result of worker execution.
pub enum WorkResult {
    Yielded(YieldUpdate),
    Panicked(String),
    Preempted,
}

/// State of a running task from the runner's perspective.
pub enum RunningTaskState {
    Completed(WorkResult),
    Preempting,
    Executing,
}

/* TYPES */

/// Thread pool runner configuration.
#[derive(Clone, Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct ThreadPoolConfig {
    #[builder(default = "num_cpus::get()")]
    pub num_threads: usize,
    #[builder(default = "Duration::from_millis(100)")]
    pub preempt_timeout: Duration,
}

/// Concurrent runner that executes tasks across a pool of worker threads.
pub struct ThreadPoolRunner {
    /// Worker thread handles
    pub workers: Vec<JoinHandle<()>>,

    /// Channel to dispatch work to idle workers
    pub work_tx: Sender<WorkPacket>,

    /// Channel to send completed work from workers
    pub result_tx: Sender<CompletionPacket>,

    /// Channel to receive completed work from workers
    pub result_rx: Receiver<CompletionPacket>,

    /// Track state of running tasks (scheduler's perspective)
    pub running: HashMap<TaskID, RunningTaskState>,

    /// Completed tasks waiting to be collected
    pub completed: HashMap<TaskID, Box<dyn Executable>>,

    /// Preemption signals indexed by TaskID
    pub signals: HashMap<TaskID, Arc<AtomicBool>>,

    /// Configuration
    pub config: ThreadPoolConfig,

    /// Shutdown signal for all workers
    pub shutdown: Arc<AtomicBool>,
}

/// Work packet sent from runner to worker.
pub struct WorkPacket {
    pub executable: Box<dyn Executable>,
    pub result_tx: Sender<CompletionPacket>,
    pub awaited: TaskOutcomes,
    pub signal: Arc<AtomicBool>,
    pub tid: TaskID,
}

/// Completion packet sent from worker back to runner.
pub struct CompletionPacket {
    pub executable: Box<dyn Executable>,
    pub result: WorkResult,
    pub tid: TaskID,
}
