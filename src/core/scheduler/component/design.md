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
