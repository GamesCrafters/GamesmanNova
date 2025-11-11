//! # Thread Pool Runner Implementation
//!
//! TODO

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
    fn capacity(&self) -> Option<usize> {
        Some(self.config.num_threads)
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

        self.running
            .insert(tid, RunningTaskState::Executing);

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

        match update {
            None => {
                // Task wants to continue ticking - don't yield to scheduler yet
                awaited = TaskOutcomes::new();
            },
            Some(yield_update) => {
                // Task yielding control back to scheduler
                return WorkResult::Yielded(yield_update);
            },
        }
    };

    let panic_result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(execute));

    let result = panic_result.unwrap_or_else(WorkResult::panicked);
    (executable, result)
}

/* TESTS */

#[cfg(test)]
mod tests {

    use anyhow::Result;
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::core::developer::GraphBuilder;
    use crate::core::scheduler::Scheduler;
    use crate::core::scheduler::SchedulerContextBuilder;
    use crate::core::scheduler::SchedulerSnapshot;
    use crate::core::scheduler::SchedulerState;
    use crate::core::scheduler::TaskOutcome;
    use crate::core::scheduler::logger::history::HistoryLogger;
    use crate::core::scheduler::logger::history::HistoryLoggerBuilder;
    use crate::core::scheduler::policy::critical::CriticalPathPolicyBuilder;
    use crate::core::scheduler::task::mock::TaskBuilder;
    use crate::core::scheduler::task::mock::TaskNodeBuilder;
    use crate::traits::scheduler::Logger;
    use crate::traits::scheduler::Policy;
    use crate::traits::scheduler::Runner;

    use super::*;

    /* HELPER FUNCTIONS */

    const MODULE: &str = "threadpool-runner";

    /// Helper: Poll until task is no longer Pending, or timeout.
    fn poll_until_ready(
        runner: &mut ThreadPoolRunner,
        tid: TaskID,
        timeout: Duration,
    ) -> Result<PollStatus> {
        let start = std::time::Instant::now();
        loop {
            let status = runner.poll(tid)?;
            if !matches!(status, PollStatus::Pending) {
                return Ok(status);
            }
            if start.elapsed() > timeout {
                bail!("Timeout waiting for task {} to complete", tid);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// Helper: Execute task and wait for it to complete or timeout.
    fn execute_and_wait(
        runner: &mut ThreadPoolRunner,
        tid: TaskID,
        executable: Box<dyn Executable>,
        timeout: Duration,
    ) -> Result<PollStatus> {
        runner.execute(tid, TaskOutcomes::new(), executable)?;
        poll_until_ready(runner, tid, timeout)
    }

    /// Wrapper for HistoryLogger that allows shared access in tests
    struct SharedLogger {
        inner: Rc<RefCell<HistoryLogger>>,
    }

    impl SharedLogger {
        fn new(logger: HistoryLogger) -> (Self, Rc<RefCell<HistoryLogger>>) {
            let inner = Rc::new(RefCell::new(logger));
            let shared = SharedLogger {
                inner: inner.clone(),
            };
            (shared, inner)
        }
    }

    impl Logger for SharedLogger {
        fn observe(
            &mut self,
            snapshot: &SchedulerSnapshot,
            changed: bool,
        ) -> Result<()> {
            self.inner
                .borrow_mut()
                .observe(snapshot, changed)
        }
    }

    /// Helper to create a scheduler with ThreadPoolRunner for integration tests
    fn create_test_scheduler_with_threadpool(
        num_threads: usize,
        sigma: f64,
    ) -> Result<(Scheduler, Rc<RefCell<HistoryLogger>>)> {
        let history = HistoryLoggerBuilder::default()
            .frequency(1usize)
            .build()?;

        let (logger, logger_ref) = SharedLogger::new(history);

        let config = ThreadPoolConfigBuilder::default()
            .num_threads(num_threads)
            .build()?;

        let runner = ThreadPoolRunner::new(config)?;

        let policy = CriticalPathPolicyBuilder::default()
            .sigma(sigma)
            .build()?;

        let context = SchedulerContextBuilder::default()
            .policy(Box::new(policy) as Box<dyn Policy>)
            .logger(Box::new(logger) as Box<dyn Logger>)
            .runner(Box::new(runner) as Box<dyn Runner>)
            .build()?;

        let state = SchedulerState::default();
        let scheduler = Scheduler::new(context, state);

        Ok((scheduler, logger_ref))
    }

    /// Helper to find task ID by name in final snapshot
    fn find_task_by_name(
        snapshots: &[SchedulerSnapshot],
        name: &str,
    ) -> TaskID {
        snapshots
            .last()
            .unwrap()
            .tasks
            .iter()
            .find_map(|(tid, ctx)| (ctx.about == name).then_some(*tid))
            .unwrap()
    }

    /* UNIT TESTS */

    #[test]
    fn test_execute_and_poll_simple_task() -> Result<()> {
        let config = ThreadPoolConfigBuilder::default()
            .num_threads(2usize)
            .build()?;

        let mut runner = ThreadPoolRunner::new(config)?;
        let task_config = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(42))
            .about("simple task")
            .build()?;

        let graph = GraphBuilder::new();
        let mock_task = TaskBuilder::new()
            .name("test-execute-poll")
            .graph(graph)
            .source(&task_config)
            .build()?;

        let tid = TaskID::from(0u64);
        let executable = mock_task.root_task()?.executable;
        let awaited = TaskOutcomes::new();

        runner.execute(tid, awaited, executable)?;

        let timeout = Duration::from_secs(1);
        let status = poll_until_ready(&mut runner, tid, timeout)?;
        let ready = matches!(
            status,
            PollStatus::Ready(YieldUpdate {
                intention: YieldIntention::Ready,
                ..
            })
        );
        assert!(ready, "Expected Ready after first execution");

        let executable = runner.collect(tid)?;
        runner.execute(tid, TaskOutcomes::new(), executable)?;

        let status = poll_until_ready(&mut runner, tid, timeout)?;
        let success = matches!(
            status,
            PollStatus::Ready(YieldUpdate {
                intention: YieldIntention::Suspended(TaskOutcome::Success(42)),
                ..
            })
        );
        assert!(success, "Expected Suspended(Success(42))");

        let _collected = runner.collect(tid)?;
        Ok(())
    }

    #[test]
    fn test_concurrent_execution_multiple_tasks() -> Result<()> {
        let config = ThreadPoolConfigBuilder::default()
            .num_threads(3usize)
            .build()?;
        let mut runner = ThreadPoolRunner::new(config)?;
        let config1 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .about("task1")
            .build()?;

        let config2 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .about("task2")
            .build()?;

        let config3 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(3))
            .about("task3")
            .build()?;

        let graph1 = GraphBuilder::new();
        let graph2 = GraphBuilder::new();
        let graph3 = GraphBuilder::new();

        let mock1 = TaskBuilder::new()
            .name("test-task1")
            .graph(graph1)
            .source(&config1)
            .build()?;

        let mock2 = TaskBuilder::new()
            .name("test-task2")
            .graph(graph2)
            .source(&config2)
            .build()?;

        let mock3 = TaskBuilder::new()
            .name("test-task3")
            .graph(graph3)
            .source(&config3)
            .build()?;

        let task1 = mock1.root_task()?.executable;
        let task2 = mock2.root_task()?.executable;
        let task3 = mock3.root_task()?.executable;

        let tid1 = TaskID::from(1u64);
        let tid2 = TaskID::from(2u64);
        let tid3 = TaskID::from(3u64);

        runner.execute(tid1, TaskOutcomes::new(), task1)?;
        runner.execute(tid2, TaskOutcomes::new(), task2)?;
        runner.execute(tid3, TaskOutcomes::new(), task3)?;

        let timeout = Duration::from_secs(1);
        let status1 = poll_until_ready(&mut runner, tid1, timeout)?;
        let status2 = poll_until_ready(&mut runner, tid2, timeout)?;
        let status3 = poll_until_ready(&mut runner, tid3, timeout)?;

        assert!(matches!(status1, PollStatus::Ready(_)));
        assert!(matches!(status2, PollStatus::Ready(_)));
        assert!(matches!(status3, PollStatus::Ready(_)));

        let exec1 = runner.collect(tid1)?;
        let exec2 = runner.collect(tid2)?;
        let exec3 = runner.collect(tid3)?;

        runner.execute(tid1, TaskOutcomes::new(), exec1)?;
        runner.execute(tid2, TaskOutcomes::new(), exec2)?;
        runner.execute(tid3, TaskOutcomes::new(), exec3)?;

        let final1 = poll_until_ready(&mut runner, tid1, timeout)?;
        let final2 = poll_until_ready(&mut runner, tid2, timeout)?;
        let final3 = poll_until_ready(&mut runner, tid3, timeout)?;

        let success1 = matches!(
            final1,
            PollStatus::Ready(YieldUpdate {
                intention: YieldIntention::Suspended(TaskOutcome::Success(1)),
                ..
            })
        );
        let success2 = matches!(
            final2,
            PollStatus::Ready(YieldUpdate {
                intention: YieldIntention::Suspended(TaskOutcome::Success(2)),
                ..
            })
        );
        let success3 = matches!(
            final3,
            PollStatus::Ready(YieldUpdate {
                intention: YieldIntention::Suspended(TaskOutcome::Success(3)),
                ..
            })
        );

        assert!(success1, "Task 1 should complete with Success(1)");
        assert!(success2, "Task 2 should complete with Success(2)");
        assert!(success3, "Task 3 should complete with Success(3)");

        runner.collect(tid1)?;
        runner.collect(tid2)?;
        runner.collect(tid3)?;

        Ok(())
    }

    #[test]
    fn test_capacity_enforcement_when_saturated() -> Result<()> {
        let config = ThreadPoolConfigBuilder::default()
            .num_threads(2usize)
            .build()?;
        let mut runner = ThreadPoolRunner::new(config)?;
        let config1 = TaskNodeBuilder::default()
            .ticks(5)
            .release(5)
            .outcome(TaskOutcome::Success(1))
            .about("long1")
            .build()?;

        let config2 = TaskNodeBuilder::default()
            .ticks(5)
            .release(5)
            .outcome(TaskOutcome::Success(2))
            .about("long2")
            .build()?;

        let config3 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(3))
            .about("quick")
            .build()?;

        let graph = GraphBuilder::new();
        let mock1 = TaskBuilder::new()
            .name("long1")
            .graph(graph)
            .source(&config1)
            .build()?;

        let graph = GraphBuilder::new();
        let mock2 = TaskBuilder::new()
            .name("long2")
            .graph(graph)
            .source(&config2)
            .build()?;

        let graph = GraphBuilder::new();
        let mock3 = TaskBuilder::new()
            .name("quick")
            .graph(graph)
            .source(&config3)
            .build()?;

        let task1 = mock1.root_task()?.executable;
        let task2 = mock2.root_task()?.executable;
        let task3 = mock3.root_task()?.executable;

        let tid1 = TaskID::from(1u64);
        let tid2 = TaskID::from(2u64);
        let tid3 = TaskID::from(3u64);

        runner.execute(tid1, TaskOutcomes::new(), task1)?;
        runner.execute(tid2, TaskOutcomes::new(), task2)?;

        let result = runner.execute(tid3, TaskOutcomes::new(), task3);
        assert!(result.is_err());

        let message = format!("{}", result.unwrap_err());
        assert!(message.contains("workers are busy"));

        Ok(())
    }

    #[test]
    fn test_preemption_interrupts_execution() -> Result<()> {
        let config = ThreadPoolConfigBuilder::default()
            .num_threads(1usize)
            .build()?;
        let mut runner = ThreadPoolRunner::new(config)?;

        let task_config = TaskNodeBuilder::default()
            .ticks(10)
            .release(10)
            .outcome(TaskOutcome::Success(42))
            .about("long-task")
            .build()?;

        let graph = GraphBuilder::new();
        let mock_task = TaskBuilder::new()
            .name("long-task")
            .graph(graph)
            .source(&task_config)
            .build()?;

        let tid = TaskID::from(1u64);
        let executable = mock_task.root_task()?.executable;

        runner.execute(tid, TaskOutcomes::new(), executable)?;
        assert!(matches!(runner.poll(tid)?, PollStatus::Pending));

        runner.preempt(tid)?;

        let timeout = Duration::from_secs(1);
        let status = poll_until_ready(&mut runner, tid, timeout)?;

        let ready = matches!(
            status,
            PollStatus::Ready(YieldUpdate {
                intention: YieldIntention::Ready,
                ..
            })
        );
        assert!(ready, "Expected Ready after preemption");

        let _ = runner.collect(tid)?;

        Ok(())
    }

    #[test]
    fn test_error_cases_double_execute_invalid_collect() -> Result<()> {
        let config = ThreadPoolConfigBuilder::default()
            .num_threads(2usize)
            .build()?;
        let mut runner = ThreadPoolRunner::new(config)?;

        let task_config = TaskNodeBuilder::default()
            .ticks(10)
            .release(10)
            .outcome(TaskOutcome::Success(1))
            .about("test-task")
            .build()?;

        let graph = GraphBuilder::new();
        let mock_task = TaskBuilder::new()
            .name("test")
            .graph(graph)
            .source(&task_config)
            .build()?;

        let tid = TaskID::from(1u64);
        let executable = mock_task.root_task()?.executable;
        runner.execute(tid, TaskOutcomes::new(), executable)?;

        let task_config2 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .about("task2")
            .build()?;

        let graph2 = GraphBuilder::new();
        let mock2 = TaskBuilder::new()
            .name("test2")
            .graph(graph2)
            .source(&task_config2)
            .build()?;

        let executable2 = mock2.root_task()?.executable;
        let result = runner.execute(tid, TaskOutcomes::new(), executable2);
        assert!(result.is_err());

        let message = format!("{}", result.unwrap_err());
        assert!(message.contains("already running"));

        let unknown = TaskID::from(999u64);
        assert!(runner.poll(unknown).is_err());

        assert!(runner.preempt(unknown).is_err());

        Ok(())
    }

    /* INTEGRATION TESTS */

    #[test]
    fn test_integration_concurrent_independent_branches() -> Result<()> {
        let (mut scheduler, logger_ref) =
            create_test_scheduler_with_threadpool(2, 1.0)?;
        let root = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("root")
            .size(Some(10))
            .build()?;

        let branch1 = TaskNodeBuilder::default()
            .ticks(2)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .about("branch1")
            .size(Some(20))
            .build()?;

        let child1 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .about("child1")
            .size(Some(15))
            .build()?;

        let branch2 = TaskNodeBuilder::default()
            .ticks(2)
            .release(1)
            .outcome(TaskOutcome::Success(3))
            .about("branch2")
            .size(Some(18))
            .build()?;

        let child2 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(4))
            .about("child2")
            .size(Some(12))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&root, &branch1)
            .edge(&branch1, &child1)
            .edge(&root, &branch2)
            .edge(&branch2, &child2);

        let task_graph = TaskBuilder::new()
            .name("integration-concurrent")
            .graph(graph)
            .source(&root)
            .build()?;

        task_graph.visualize(MODULE)?;

        scheduler.register(task_graph.root_task()?)?;
        scheduler.run()?;

        let logger = logger_ref.borrow();
        let snapshots = logger.snapshots();

        let root_tid = find_task_by_name(snapshots, "root");
        let branch1_tid = find_task_by_name(snapshots, "branch1");
        let child1_tid = find_task_by_name(snapshots, "child1");
        let branch2_tid = find_task_by_name(snapshots, "branch2");
        let child2_tid = find_task_by_name(snapshots, "child2");
        assert!(
            logger.before(root_tid, branch1_tid),
            "root should start before branch1"
        );
        assert!(
            logger.before(branch1_tid, child1_tid),
            "branch1 should start before child1"
        );
        assert!(
            logger.before(root_tid, branch2_tid),
            "root should start before branch2"
        );
        assert!(
            logger.before(branch2_tid, child2_tid),
            "branch2 should start before child2"
        );

        Ok(())
    }

    #[test]
    fn test_integration_complex_dag_execution() -> Result<()> {
        // Graph structure:
        //                      Root
        //           /          |           \
        //     FastPath    SlowPath    CriticalPath
        //       / \          / \            / \
        //    FC1  FC2     SC1  SC2       CC1  CC2
        //      \  /         \  /            \  /
        //     Merger1     Merger2         Merger3

        let (mut scheduler, logger_ref) =
            create_test_scheduler_with_threadpool(4, 1.0)?;
        let root = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .about("root")
            .size(Some(5))
            .build()?;

        let fast_path = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .about("fast_path")
            .size(Some(10))
            .build()?;

        let slow_path = TaskNodeBuilder::default()
            .ticks(3)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .about("slow_path")
            .size(Some(50))
            .build()?;

        let critical_path = TaskNodeBuilder::default()
            .ticks(5)
            .release(1)
            .outcome(TaskOutcome::Success(3))
            .about("critical_path")
            .size(Some(100))
            .build()?;

        let fast_child1 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(4))
            .about("fast_child1")
            .size(Some(10))
            .build()?;

        let fast_child2 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(5))
            .about("fast_child2")
            .size(Some(8))
            .build()?;

        let slow_child1 = TaskNodeBuilder::default()
            .ticks(2)
            .release(1)
            .outcome(TaskOutcome::Success(6))
            .about("slow_child1")
            .size(Some(20))
            .build()?;

        let slow_child2 = TaskNodeBuilder::default()
            .ticks(2)
            .release(1)
            .outcome(TaskOutcome::Success(7))
            .about("slow_child2")
            .size(Some(15))
            .build()?;

        let critical_child1 = TaskNodeBuilder::default()
            .ticks(2)
            .release(1)
            .outcome(TaskOutcome::Success(8))
            .about("critical_child1")
            .size(Some(30))
            .build()?;

        let critical_child2 = TaskNodeBuilder::default()
            .ticks(2)
            .release(1)
            .outcome(TaskOutcome::Success(9))
            .about("critical_child2")
            .size(Some(25))
            .build()?;

        let merger1 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(10))
            .about("merger1")
            .size(Some(5))
            .build()?;

        let merger2 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(11))
            .about("merger2")
            .size(Some(5))
            .build()?;

        let merger3 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(12))
            .about("merger3")
            .size(Some(5))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&root, &fast_path)
            .edge(&root, &slow_path)
            .edge(&root, &critical_path)
            .edge(&fast_path, &fast_child1)
            .edge(&fast_path, &fast_child2)
            .edge(&slow_path, &slow_child1)
            .edge(&slow_path, &slow_child2)
            .edge(&critical_path, &critical_child1)
            .edge(&critical_path, &critical_child2)
            .edge(&fast_child1, &merger1)
            .edge(&fast_child2, &merger1)
            .edge(&slow_child1, &merger2)
            .edge(&slow_child2, &merger2)
            .edge(&critical_child1, &merger3)
            .edge(&critical_child2, &merger3);

        let task_graph = TaskBuilder::new()
            .name("integration-complex-dag")
            .graph(graph)
            .source(&root)
            .build()?;

        task_graph.visualize(MODULE)?;

        scheduler.register(task_graph.root_task()?)?;
        scheduler.run()?;

        let logger = logger_ref.borrow();
        let snapshots = logger.snapshots();

        let root_tid = find_task_by_name(snapshots, "root");
        let fast_path_tid = find_task_by_name(snapshots, "fast_path");
        let slow_path_tid = find_task_by_name(snapshots, "slow_path");
        let critical_path_tid = find_task_by_name(snapshots, "critical_path");
        let fast_child1_tid = find_task_by_name(snapshots, "fast_child1");
        let fast_child2_tid = find_task_by_name(snapshots, "fast_child2");
        let slow_child1_tid = find_task_by_name(snapshots, "slow_child1");
        let critical_child1_tid =
            find_task_by_name(snapshots, "critical_child1");

        let merger1_tid = find_task_by_name(snapshots, "merger1");
        let merger2_tid = find_task_by_name(snapshots, "merger2");

        assert!(
            logger.before(root_tid, fast_path_tid),
            "root should start before fast_path"
        );
        assert!(
            logger.before(root_tid, slow_path_tid),
            "root should start before slow_path"
        );
        assert!(
            logger.before(root_tid, critical_path_tid),
            "root should start before critical_path"
        );
        assert!(
            logger.before(fast_path_tid, fast_child1_tid),
            "fast_path should start before fast_child1"
        );
        assert!(
            logger.before(fast_path_tid, fast_child2_tid),
            "fast_path should start before fast_child2"
        );
        assert!(
            logger.before(slow_path_tid, slow_child1_tid),
            "slow_path should start before slow_child1"
        );
        assert!(
            logger.before(critical_path_tid, critical_child1_tid),
            "critical_path should start before critical_child1"
        );
        assert!(
            logger.before(fast_child1_tid, merger1_tid),
            "fast_child1 should start before merger1"
        );
        assert!(
            logger.before(fast_child2_tid, merger1_tid),
            "fast_child2 should start before merger1"
        );
        assert!(
            logger.before(slow_child1_tid, merger2_tid),
            "slow_child1 should start before merger2"
        );

        Ok(())
    }
}
