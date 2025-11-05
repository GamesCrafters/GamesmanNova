//! # Thread Pool Runner Implementation
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use crossbeam_channel::Receiver;
use crossbeam_channel::unbounded;

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::Builder;
use std::time::Duration;

use crate::traits::scheduler::Executable;
use crate::traits::scheduler::Runner;
use crate::types::scheduler::PollStatus;
use crate::types::scheduler::TaskID;
use crate::types::scheduler::TaskOutcomes;
use crate::types::scheduler::YieldIntention;
use crate::types::scheduler::YieldUpdate;
use crate::types::scheduler::runner::thread::CompletionPacket;
use crate::types::scheduler::runner::thread::RunningTaskState;
use crate::types::scheduler::runner::thread::ThreadPoolConfig;
use crate::types::scheduler::runner::thread::ThreadPoolRunner;
use crate::types::scheduler::runner::thread::WorkPacket;
use crate::types::scheduler::runner::thread::WorkResult;

/* IMPLEMENTATIONS */

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self {
            preempt_timeout: Duration::from_millis(100),
            num_threads: num_cpus::get(),
        }
    }
}

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
        self.signals
            .get(&tid)
            .map(|signal| signal.store(true, Ordering::Relaxed));
    }

    /// Remove all metadata for a task.
    fn cleanup(&mut self, tid: TaskID) {
        self.running.remove(&tid);
        self.completed.remove(&tid);
        self.signals.remove(&tid);
    }

    /// Get task state.
    fn state(&self, tid: TaskID) -> Option<&RunningTaskState> {
        self.running.get(&tid)
    }
}

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
        let packet = WorkPacket {
            result_tx,
            executable,
            awaited,
            signal,
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
}

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

/* HELPER FUNCTIONS */

fn harness(work_rx: Receiver<WorkPacket>, shutdown: Arc<AtomicBool>) {
    while !shutdown.load(Ordering::Relaxed) {
        let timeout = Duration::from_millis(100);
        let packet = match work_rx.recv_timeout(timeout) {
            Ok(packet) => packet,
            Err(_) => continue,
        };

        let (executable, result) =
            execute_task(packet.signal, packet.awaited, packet.executable);

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
) -> (Box<dyn Executable>, WorkResult) {
    let execute = || loop {
        if signal.load(Ordering::Relaxed) {
            return WorkResult::Preempted;
        }

        let update = executable.tick(awaited);
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
