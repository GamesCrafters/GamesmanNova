//! # Thread Pool Runner
//!
//! Concurrent execution across worker threads with true
//! preemption via atomic signals.

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::Builder;
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use crossbeam_channel::unbounded;
use derive_builder::Builder as DeriveBuilder;

use crate::core::scheduler::PollStatus;
use crate::core::scheduler::TaskID;
use crate::core::scheduler::TaskOutcomes;
use crate::core::scheduler::YieldIntention;
use crate::core::scheduler::YieldUpdate;
use crate::traits::scheduler::Executable;
use crate::traits::scheduler::Runner;

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

/* STRUCTURES */

/// Thread pool runner configuration.
#[derive(Clone, DeriveBuilder)]
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

    /// Progress samples from running tasks
    pub progress: Arc<RwLock<HashMap<TaskID, u64>>>,

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
    pub progress: Arc<RwLock<HashMap<TaskID, u64>>>,
    pub tid: TaskID,
}

/// Completion packet sent from worker back to runner.
pub struct CompletionPacket {
    pub executable: Box<dyn Executable>,
    pub result: WorkResult,
    pub tid: TaskID,
}

/* IMPLEMENTATIONS */

impl ThreadPoolRunner {
    pub fn new(config: ThreadPoolConfig) -> Result<Self> {
        let (work_tx, work_rx) = unbounded();
        let (result_tx, result_rx) = unbounded();
        let shutdown = Arc::new(AtomicBool::new(false));

        let spawn = |i| {
            let work_rx = work_rx.clone();
            let shutdown = shutdown.clone();
            Builder::new()
                .name(format!("nova-worker-{}", i))
                .spawn(move || harness(work_rx, shutdown))
                .context(format!("Failed to spawn worker thread {}", i))
        };

        let workers = (0..config.num_threads)
            .map(spawn)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            progress: Arc::new(RwLock::new(HashMap::new())),
            completed: HashMap::new(),
            running: HashMap::new(),
            signals: HashMap::new(),
            result_tx,
            result_rx,
            shutdown,
            work_tx,
            workers,
            config,
        })
    }

    /// Receive completed task results and drain their metadata.
    fn process_completed(&mut self) {
        while let Ok(completion) = self.result_rx.try_recv() {
            self.running.insert(
                completion.tid,
                RunningTaskState::Completed(completion.result),
            );

            self.completed
                .insert(completion.tid, completion.executable);

            self.signals
                .remove(&completion.tid);
        }
    }

    /// Check if runner has capacity for new tasks.
    fn available(&self) -> bool {
        self.running.len() < self.config.num_threads
    }

    /// Set preemption signal for a task.
    fn signal(&self, tid: TaskID) {
        if let Some(signal) = self.signals.get(&tid) {
            signal.store(true, Ordering::Relaxed)
        }
    }

    /// Remove all metadata for a task.
    fn cleanup(&mut self, tid: TaskID) {
        self.running.remove(&tid);
        self.completed.remove(&tid);
        self.signals.remove(&tid);
        if let Ok(mut map) = self.progress.write() {
            map.remove(&tid);
        }
    }

    /// Get task state.
    fn state(&self, tid: TaskID) -> Option<&RunningTaskState> {
        self.running.get(&tid)
    }
}

impl WorkResult {
    /// Convert panic payload to WorkResult::Panicked.
    pub fn panicked(panic: Box<dyn Any + Send>) -> Self {
        let message = if let Some(s) = panic.downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = panic.downcast_ref::<&str>() {
            s.to_string()
        } else {
            "Unknown panic".to_string()
        };
        Self::Panicked(message)
    }

    /// Convert WorkResult to PollStatus.
    pub fn status(self) -> PollStatus {
        match self {
            Self::Yielded(update) => PollStatus::Ready(update),
            Self::Preempted => PollStatus::Ready(YieldUpdate {
                intention: YieldIntention::Ready,
                discovered: Vec::new(),
            }),
            Self::Panicked(msg) => PollStatus::Panic(msg),
        }
    }

    /// Check if result is yielded.
    pub fn yielded(&self) -> bool {
        match self {
            Self::Yielded(_) => true,
            Self::Preempted | Self::Panicked(_) => false,
        }
    }

    /// Check if result is preempted.
    pub fn preempted(&self) -> bool {
        match self {
            Self::Preempted => true,
            Self::Yielded(_) | Self::Panicked(_) => false,
        }
    }
}

impl RunningTaskState {
    /// Check if task is executing.
    pub fn executing(&self) -> bool {
        match self {
            Self::Executing => true,
            Self::Preempting | Self::Completed(_) => false,
        }
    }

    /// Check if task is preempting.
    pub fn preempting(&self) -> bool {
        match self {
            Self::Preempting => true,
            Self::Executing | Self::Completed(_) => false,
        }
    }

    /// Check if task is completed.
    pub fn completed(&self) -> bool {
        match self {
            Self::Completed(_) => true,
            Self::Executing | Self::Preempting => false,
        }
    }

    /// Check if task can be preempted.
    pub fn preemptable(&self) -> bool {
        match self {
            Self::Executing | Self::Preempting | Self::Completed(_) => true,
        }
    }

    /// Check if task can be collected.
    pub fn collectable(&self) -> bool {
        match self {
            Self::Completed(_) => true,
            Self::Executing | Self::Preempting => false,
        }
    }

    /// Extract result, consuming self.
    pub fn result(self) -> Option<WorkResult> {
        match self {
            Self::Completed(result) => Some(result),
            Self::Executing | Self::Preempting => None,
        }
    }

    /// Take result, leaving Preempted placeholder.
    pub fn take(&mut self) -> Option<WorkResult> {
        match self {
            Self::Completed(_) => {
                let placeholder = Self::Completed(WorkResult::Preempted);
                match std::mem::replace(self, placeholder) {
                    Self::Completed(result) => Some(result),
                    Self::Executing | Self::Preempting => unreachable!(),
                }
            },
            Self::Executing | Self::Preempting => None,
        }
    }
}

/* IMPL TRAIT FOR TYPE */

impl Runner for ThreadPoolRunner {
    fn units(&self) -> usize {
        self.config.num_threads
    }

    fn execute(
        &mut self,
        tid: TaskID,
        awaited: TaskOutcomes,
        executable: Box<dyn Executable>,
    ) -> Result<()> {
        if self.running.contains_key(&tid) {
            bail!("Task {} is already running", tid);
        }

        if !self.available() {
            bail!(
                "All {} workers are busy (task {} cannot be dispatched)",
                self.config.num_threads,
                tid
            );
        }

        let signal = Arc::new(AtomicBool::new(false));
        self.signals
            .insert(tid, signal.clone());

        let result_tx = self.result_tx.clone();
        let progress = self.progress.clone();
        let packet = WorkPacket {
            result_tx,
            executable,
            awaited,
            signal,
            progress,
            tid,
        };

        self.work_tx
            .send(packet)
            .map_err(|_| anyhow!("Failed to dispatch task to worker"))?;

        self.running
            .insert(tid, RunningTaskState::Executing);

        Ok(())
    }

    fn poll(&mut self, tid: TaskID) -> Result<PollStatus> {
        self.process_completed();
        let state = self
            .running
            .get_mut(&tid)
            .context(format!("Task {} not found", tid))?;

        match state {
            RunningTaskState::Executing | RunningTaskState::Preempting => {
                Ok(PollStatus::Pending)
            },
            RunningTaskState::Completed(_) => {
                let result = state
                    .take()
                    .context(format!("Task {} state inconsistency", tid))?;

                let status = result.status();
                Ok(status)
            },
        }
    }

    fn preempt(&mut self, tid: TaskID) -> Result<()> {
        self.process_completed();
        let state = self
            .running
            .get(&tid)
            .context(format!("Task {} not found", tid))?;

        match state {
            RunningTaskState::Executing => {
                self.signal(tid);
                self.running
                    .insert(tid, RunningTaskState::Preempting);

                Ok(())
            },
            RunningTaskState::Preempting | RunningTaskState::Completed(_) => {
                Ok(())
            },
        }
    }

    fn collect(&mut self, tid: TaskID) -> Result<Box<dyn Executable>> {
        self.process_completed();
        let state = self
            .running
            .get(&tid)
            .context(format!("Task {} not found", tid))?;

        match state {
            RunningTaskState::Completed(_) => {
                self.running.remove(&tid);
                let executable = self
                    .completed
                    .remove(&tid)
                    .context(format!("Task {} executable missing", tid))?;

                Ok(executable)
            },
            RunningTaskState::Executing | RunningTaskState::Preempting => {
                bail!(
                    "Task {} is not collectable (executing or preempting)",
                    tid
                )
            },
        }
    }

    fn progress(&self, tid: TaskID) -> Option<u64> {
        self.progress
            .read()
            .ok()?
            .get(&tid)
            .copied()
    }
}

/* IMPL EXTERNAL TRAIT */

impl Drop for ThreadPoolRunner {
    fn drop(&mut self) {
        self.shutdown
            .store(true, Ordering::Relaxed);

        drop(std::mem::replace(
            &mut self.work_tx,
            unbounded().0,
        ));

        while let Some(handle) = self.workers.pop() {
            let _ = handle.join();
        }
    }
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self {
            preempt_timeout: Duration::from_millis(100),
            num_threads: num_cpus::get(),
        }
    }
}

/* HELPER FUNCTIONS */

fn harness(work_rx: Receiver<WorkPacket>, shutdown: Arc<AtomicBool>) {
    while !shutdown.load(Ordering::Relaxed) {
        let timeout = Duration::from_millis(100);
        let packet = match work_rx.recv_timeout(timeout) {
            Ok(packet) => packet,
            Err(_) => continue,
        };

        let (executable, result) = execute_task(
            packet.signal,
            packet.awaited,
            packet.executable,
            packet.progress,
            packet.tid,
        );

        let _ = packet
            .result_tx
            .send(CompletionPacket {
                tid: packet.tid,
                executable,
                result,
            });
    }
}

fn execute_task(
    signal: Arc<AtomicBool>,
    mut awaited: TaskOutcomes,
    mut executable: Box<dyn Executable>,
    progress: Arc<RwLock<HashMap<TaskID, u64>>>,
    tid: TaskID,
) -> (Box<dyn Executable>, WorkResult) {
    let execute = || loop {
        if signal.load(Ordering::Relaxed) {
            return WorkResult::Preempted;
        }

        let update = executable.tick(awaited);

        if let Some(value) = executable.progress()
            && let Ok(mut map) = progress.write()
        {
            map.insert(tid, value);
        }

        if !update.ready() {
            return WorkResult::Yielded(update);
        }

        awaited = TaskOutcomes::new();
    };

    let panic_result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(execute));

    let result = panic_result.unwrap_or_else(WorkResult::panicked);
    (executable, result)
}
