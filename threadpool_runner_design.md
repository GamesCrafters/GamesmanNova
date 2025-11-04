# ThreadPool Runner - Final Design

## Core Structure

```rust
use threadpool::ThreadPool;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::collections::{HashMap, HashSet};

pub struct ThreadPoolRunner {
    /// Fixed-size thread pool for executing ticks
    pool: ThreadPool,

    /// Channel for workers to send completions back
    result_sender: Sender<WorkerMessage>,
    result_receiver: Receiver<WorkerMessage>,

    /// Tasks currently executing on workers
    in_flight: HashSet<TaskID>,

    /// Completed tick results (polled and removed)
    completed_results: HashMap<TaskID, Result<YieldUpdate>>,

    /// Completed executables (stopped and removed)
    completed_executables: HashMap<TaskID, Box<dyn Executable>>,
}

struct WorkerMessage {
    tid: TaskID,
    executable: Box<dyn Executable>,
    result: Result<YieldUpdate>,
}
```

## Implementation

```rust
#[async_trait]
impl Runner for ThreadPoolRunner {
    async fn spawn(
        &mut self,
        tid: TaskID,
        mut task: Box<dyn Executable>,
        deps: TaskOutcomes,
    ) -> Result<()> {
        if self.in_flight.contains(&tid) {
            bail!("Task {} already executing", tid);
        }

        let sender = self.result_sender.clone();
        self.in_flight.insert(tid);

        // Dispatch to thread pool
        self.pool.execute(move || {
            // Catch panics to avoid poisoning the pool
            let result = std::panic::catch_unwind(
                std::panic::AssertUnwindSafe(|| task.execute(deps))
            );

            let result = match result {
                Ok(update) => Ok(update),
                Err(e) => Err(anyhow!("Task panicked: {:?}", e)),
            };

            // Send back to main thread
            let _ = sender.send(WorkerMessage {
                tid,
                executable: task,
                result,
            });
        });

        Ok(())
    }

    fn poll(&mut self, tid: TaskID) -> Option<Result<YieldUpdate>> {
        // Drain all available completions from channel
        while let Ok(msg) = self.result_receiver.try_recv() {
            self.in_flight.remove(&msg.tid);
            self.completed_results.insert(msg.tid, msg.result);
            self.completed_executables.insert(msg.tid, msg.executable);
        }

        // Return result for this specific task
        self.completed_results.remove(&tid)
    }

    async fn stop(&mut self, tid: TaskID) -> Result<Box<dyn Executable>> {
        // Clean up result if poll() wasn't called (preemption case)
        self.completed_results.remove(&tid);

        // Fast path: already completed
        if let Some(exec) = self.completed_executables.remove(&tid) {
            return Ok(exec);
        }

        // Slow path: wait for completion (blocks up to one tick duration)
        loop {
            let msg = self.result_receiver.recv()
                .context("Channel closed while waiting for task")?;

            self.in_flight.remove(&msg.tid);

            if msg.tid == tid {
                // This is the one we're waiting for
                return Ok(msg.executable);
            } else {
                // Buffer for later retrieval
                self.completed_results.insert(msg.tid, msg.result);
                self.completed_executables.insert(msg.tid, msg.executable);
            }
        }
    }
}

impl ThreadPoolRunner {
    pub fn new(num_workers: usize) -> Self {
        let (sender, receiver) = channel();

        Self {
            pool: ThreadPool::new(num_workers),
            result_sender: sender,
            result_receiver: receiver,
            in_flight: HashSet::new(),
            completed_results: HashMap::new(),
            completed_executables: HashMap::new(),
        }
    }
}
```

## Execution Flow

### Spawn Flow
```
┌──────────────┐
│ spawn(tid)   │
└──────┬───────┘
       │ 1. Mark in_flight
       │ 2. Clone sender
       │ 3. Dispatch to pool
       ▼
┌──────────────┐
│ Worker Thread│
│ - execute()  │
│ - catch panic│
│ - send msg   │
└──────────────┘
```

### Poll Flow
```
┌──────────────┐
│ poll(tid)    │
└──────┬───────┘
       │ 1. Drain all try_recv()
       │    ├─ remove from in_flight
       │    ├─ insert result
       │    └─ insert executable
       │ 2. Remove & return result[tid]
       ▼
┌──────────────┐
│ Option<Res>  │
└──────────────┘
```

### Stop Flow (Fast Path)
```
┌──────────────┐
│ stop(tid)    │
└──────┬───────┘
       │ 1. Remove result (cleanup)
       │ 2. Check completed_executables
       │ 3. Found! Return immediately
       ▼
┌──────────────┐
│ Executable   │
└──────────────┘
```

### Stop Flow (Slow Path - Preemption)
```
┌──────────────┐
│ stop(tid)    │
└──────┬───────┘
       │ 1. Remove result (cleanup)
       │ 2. Not in completed
       │ 3. Block on recv()
       │    ├─ Got tid? Return it
       │    └─ Other? Buffer and loop
       ▼
┌──────────────┐
│ Executable   │
│ (after wait) │
└──────────────┘
```

## Performance Characteristics

### Time Complexity
- `spawn()`: O(1) - channel send + hashset insert
- `poll()`: O(c + 1) where c = completed tasks since last poll
  - First call drains channel: O(c)
  - Subsequent calls: O(1) - empty channel check
- `stop()` fast: O(1) - hashmap lookup
- `stop()` slow: O(c + 1) - blocks until completion

### Space Complexity
- O(i + c) where i = in-flight, c = completed buffered
- Bounded by total number of tasks

### Channel Overhead
- One message per tick (unavoidable)
- Message size: ~24 bytes + executable size
- Modern channels are very fast (~100ns per message)

## Correctness Properties

### Invariants
1. ∀ tid ∈ in_flight: tid ∉ completed_*
2. ∀ tid ∈ completed_results: tid ∈ completed_executables (until poll)
3. After poll(tid) returns Some(): tid ∈ completed_executables
4. After stop(tid) returns: tid ∉ any internal structures

### Safety
- No data races: channel provides synchronization
- No deadlocks: stop() always makes progress (channel will eventually receive)
- Panic safety: panics caught in workers, pool not poisoned
- No lost tasks: every spawned task eventually completes or errors

## Trade-offs vs Alternatives

### vs. Shared Mutex State
✓ Better: No lock contention on fast path (poll)
✓ Better: Clear ownership transfer via channel
✗ Worse: Slightly higher memory (channel buffering)

### vs. Lock-Free Structures
✓ Better: Simpler, easier to maintain
✓ Better: Can transfer non-Sync types (Box<dyn Executable>)
✗ Worse: Slightly higher latency (channel vs atomic)

### vs. Async Tasks
✓ Better: True parallelism (not cooperative)
✓ Better: Works with blocking executables
✗ Worse: Can't cancel mid-tick (but tick-based makes this OK)

## Bounded Blocking Guarantee

**Critical Property**: `stop()` blocking is bounded by tick duration.

**Proof**:
1. Tasks MUST return from `execute()` within bounded time (contract)
2. When `stop(tid)` blocks, tid is in_flight
3. Worker will complete within tick duration
4. Worker sends message before exiting
5. `stop()` receives message and unblocks
6. Therefore: blocking ≤ max_tick_duration

**Recommended**: Enforce tick duration < 10ms for responsive preemption.
