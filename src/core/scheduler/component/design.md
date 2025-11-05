# Scheduler Component Design

## Logger Implementations

### `CountLogger`

Prints the number of tasks that are in each possible task state (ready, running, 
etc.) whenever any of these counts change. Intended to be a simple and reliable 
fallback option for debugging.

## Policy Implementations

### `TrivialPolicy`

Does not preempt or retry any tasks. Always schedules the ready task with the 
smallest ID, numerically speaking. Intended to be a simple and reliable fallback 
option for debugging.

### `CriticalPathPolicy`

Implements weighted critical path scheduling with preemption support. This policy
prioritizes tasks that lie on the longest weighted path in the dependency graph,
where path weights are determined by task sizes. The algorithm provides theoretical
performance bounds of (2 - 1/P) × optimal makespan under certain assumptions.

**Scheduling Strategy**: Always selects the ready task with the longest critical
path depth for execution. The critical path depth of a task is defined as:
`depth(t) = size(t) + max{depth(c) : c depends on t}`. Tasks with unknown sizes
use the average size of all tasks that have reported sizes.

**Preemption Strategy**: When the number of running tasks reaches the configured
parallelism limit (`units`), considers preempting the running task with the
shortest critical path if there exists a ready task whose critical path is
sufficiently longer. The preemption threshold is determined by the `sigma`
parameter: a running task is preempted only if the maximum ready depth exceeds
the minimum running depth by at least `sigma × stddev`, where `stddev` is the
standard deviation of all task sizes.

**Retry Strategy**: Delegates to a configurable retry policy function that
determines which errored tasks should be retried. The default policy uses a
threshold-based approach that allows a fixed number of retries per task.

**Configuration**:
- `units`: Number of parallel execution units (0 disables preemption entirely)
- `sigma`: Preemption threshold multiplier (default: 1.0)
- `retry`: Custom retry policy function (default: no retries)

## Runner Implementations

### `SyncRunner`

Executes tasks synchronously, ticking them in loop until they yield. This means 
that preemption policies are redundant when this runner is used (as there will 
never be any running tasks from the perspective of the scheduler).

### `ThreadPoolRunner`

Executes tasks concurrently across a pool of worker threads, enabling true parallel
execution with support for cooperative preemption. Each worker continuously ticks
assigned tasks in a loop, checking for preemption signals between ticks to respond
to scheduler requests without blocking the scheduler's control flow.

**Execution Model**: The runner maintains exactly `num_threads` worker threads, each
capable of executing one task at a time. When a task is dispatched via `execute()`,
it is assigned to a specific idle worker thread which begins ticking it repeatedly
until the task yields or a preemption signal is received. If no workers are idle,
`execute()` returns an error, leaving the task in the scheduler's buffer. The
scheduler uses `units()` to determine how many workers are available before
attempting dispatch, avoiding unnecessary work assignment failures.

**Preemption Mechanism**: Preemption is cooperative rather than forced. When the
scheduler requests preemption via `collect()`, the runner sets an atomic flag for
that task. The worker checks this flag before each tick, and if set, immediately
stops execution and returns the task to the scheduler. This ensures preemption
occurs at well-defined boundaries (between ticks) without interrupting mid-tick
work. The scheduler may wait briefly for the current tick to complete, bounded by
the configured timeout.

**Completion Handling**: Workers send completion packets through a lock-free channel
as soon as a task yields or is preempted. The runner drains this channel on every
`poll()` and `collect()` call, ensuring the scheduler sees results with minimal
latency. Completed tasks are cached internally until the scheduler collects them,
maintaining ownership semantics.

**Error Handling**: Worker threads catch panics from task executables and convert
them to error results, preventing individual task failures from crashing workers or
affecting other concurrent tasks.

**Configuration**:
- `num_threads`: Number of worker threads in the pool (default: number of CPU cores)
- `preempt_timeout`: Maximum duration to wait for a preempted task to yield
