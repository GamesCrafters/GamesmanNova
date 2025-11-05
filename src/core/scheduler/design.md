# Scheduler Design

## Overview

## Core Scheduler

The scheduler is a task-recursive execution engine that coordinates the execution of
interdependent tasks through a poll-based control flow model. It maintains a registry
of all tasks and their dependency relationships, delegating actual execution to a
Runner implementation while using Policy and Logger components for decision making and
observability.

### Architecture

The scheduler operates through a tick-based execution loop where each tick consists
of four phases executed in sequence:

1. **Collect Phase**: Poll the runner for completion of all running and preempting
   tasks, collecting executables from completed tasks, processing any yields to update
   task states and discover new tasks.

2. **Retry Phase**: Consult the policy to determine if any failed tasks should be
   retried, transitioning them from Error to Ready state.

3. **Preempt Phase**: Consult the policy to determine if any running tasks should be
   preempted, signaling the runner to initiate preemption and transitioning them to
   Preempting state.

4. **Execute Phase**: Consult the policy to select the next ready task to execute,
   dispatching it to the runner and transitioning it to Running state.

Collecting first ensures that retry, preempt, and execute decisions are based on the
most recent runner state. This prevents attempting to preempt tasks that have already
yielded and ensures policies have accurate information about task progress.

The scheduler continues ticking until all active tasks have completed (either
successfully or with errors) or until an unrecoverable error occurs.

### State Management

The scheduler maintains two core data structures:

- **Registry**: Maps TaskID to TaskContext, tracking progress, dependencies, and
  metadata for all discovered tasks.

- **Buffer**: Maps TaskID to Box<dyn Executable>, storing task implementations that
  are not currently executing on the runner.

Executables move from buffer to runner during dispatch, and return from runner 
to buffer during retrieval (after yield or preemption).

### Control Flow

The scheduler uses **poll-based communication** with the runner:

- The scheduler never blocks waiting for tasks to complete
- Each poll operation returns immediately with current status (Pending, Ready, or Panic)
- The runner is responsible for managing its internal execution model
- Preemption is a two-phase operation:
  1. Scheduler calls runner.preempt() to signal the task should stop
  2. Scheduler polls until task yields, then calls runner.collect() to retrieve it

This design ensures the scheduler remains responsive and can make scheduling decisions
at tick boundaries without being coupled to the runner's execution strategy. The
separation of preempt signaling from collection allows tasks to finish their current
work unit before yielding.

### Methods

```rust
fn new(context: SchedulerContext, state: SchedulerState) -> Self
```

Constructs a new scheduler with the provided components and initial state.

```rust
fn register(&mut self, task: Task) -> Result<&mut Self>
```

Registers a new task with the scheduler. Inserts the executable into the buffer and
creates a TaskContext in the registry. If the task has dependencies, links them in the
dependency graph and sets the task to Waiting state. Validates that the dependency
graph remains acyclic. Returns a mutable reference to self for method chaining.

```rust
fn run(&mut self) -> Result<()>
```

Runs the scheduler until all tasks have completed (either successfully or with
errors). Repeatedly calls tick() while any tasks remain active in the registry.

```rust
fn tick(&mut self) -> Result<()>
```

Executes one scheduler tick, running all four phases in sequence. Increments the tick
counter and logs state changes if any phase modified task states. Returns after all
phases complete.

```rust
fn collect_phase(&mut self) -> Result<bool>
```

Collect phase: Polls the runner for completion status of all running and preempting
tasks. For each task that has completed (poll returns Ready or Panic), calls
runner.collect() to retrieve the executable, inserts it back into the buffer, updates
size tracking, and processes the yield result (either transitioning to a new state or
registering discovered tasks). Returns true if any task completed.

```rust
fn retry_phase(&mut self) -> Result<bool>
```

Retry phase: Consults the policy to identify failed tasks that should be retried.
Transitions identified tasks from Error to Ready state. Returns true if any task state
changed.

```rust
fn preempt_phase(&mut self) -> Result<bool>
```

Preempt phase: Consults the policy to identify running tasks that should be preempted.
For each identified task, calls runner.preempt() to signal preemption and transitions
the task to Preempting state. The actual retrieval happens later in collect_phase when
the task completes its current tick and yields. Returns true if any task was signaled
for preemption.

```rust
fn execute_phase(&mut self) -> Result<bool>
```

Execute phase: Consults the policy to select the next ready task to execute. Removes
the executable from the buffer, collects dependency outcomes, transitions the task to
Running state, and calls runner.execute() to dispatch it. Returns true if any task was
dispatched.

## Scheduler Components

### Runner Interface

The Runner trait abstracts task execution, allowing different concurrency models
(synchronous, thread pool) without changing scheduler logic.

```rust
fn units(&self) -> usize
```

Returns the number of parallel execution units available. Zero indicates synchronous
execution where tasks run to completion before returning. Used by the scheduler to
determine how many tasks can run concurrently.

```rust
fn execute(&mut self, tid: TaskID, awaited: TaskOutcomes, executable: Box<dyn Executable>) -> Result<()>
```

Initiates execution of a task with its dependencies' outcomes. Transfers ownership of
the executable to the runner. The task transitions to Running state in the scheduler
before this call. May return immediately (thread pool dispatch) or block until first yield
(synchronous execution).

```rust
fn poll(&mut self, tid: TaskID) -> Result<PollStatus>
```

Checks if a running task has yielded since the last poll. Returns:
- `PollStatus::Pending` if the task is still executing
- `PollStatus::Ready(update)` if it yielded successfully with an update
- `PollStatus::Panic(msg)` if the task executable panicked

This method must not block - it returns immediately with current status. Returns an error
for runner infrastructure failures (invalid task ID, etc).

```rust
fn preempt(&mut self, tid: TaskID) -> Result<()>
```

Signals a running task to preempt (stop execution and yield control). For synchronous
runners, this may be a no-op. For concurrent runners, this sets the preemption signal
that the task checks during execution. Does not block. Returns an error for
infrastructure failures (task not found, not running, etc).

```rust
fn collect(&mut self, tid: TaskID) -> Result<Box<dyn Executable>>
```

Retrieves a completed task's executable. Only succeeds if the task has finished
executing (poll returned Ready or Panic). Transfers ownership of the executable back to
the scheduler. Returns an error if the task is not found, still executing, or not ready
to collect. Does not block.

#### PollStatus

The PollStatus enum represents the result of polling a running task:

- **Pending**: Task is still executing and has not yielded yet. Nothing to collect.

- **Ready(YieldUpdate)**: Task has yielded and the update is available. The executable
  can now be collected via runner.collect().

- **Panic(String)**: Task executable panicked during execution. The error message is
  captured. The executable can still be collected to return it to the buffer.

### Policy Interface

The Policy trait encapsulates scheduling decisions, determining task selection,
preemption, and retry behavior.

```rust
fn retry(&mut self, state: &SchedulerState) -> Option<TaskID>
```

Identifies a failed task that should be retried. Returns a TaskID with Progress::Error
that should transition to Ready, or None if no retries are needed. Must be
idempotent - repeated calls without state changes should return the same result.

```rust
fn preempt(&mut self, state: &SchedulerState) -> Option<TaskID>
```

Identifies a running task that should be preempted. Returns a TaskID with
Progress::Running that should be retrieved and transitioned to Ready, or None if no
preemption is needed. Must be idempotent.

```rust
fn execute(&mut self, state: &SchedulerState) -> Option<TaskID>
```

Selects the next ready task to execute. Returns a TaskID with Progress::Ready that
should be dispatched to the runner, or None if no tasks should run. Must be
idempotent.

### Logger Interface

The Logger trait provides observability into scheduler state changes.

```rust
fn log(&mut self, state: &SchedulerState) -> Result<()>
```

Called after any tick that results in state changes. Receives immutable reference to
the full scheduler state including registry, buffer, and tick count. Used for metrics,
progress reporting, and debugging.

### Executable Interface

The Executable trait defines the contract for user-provided task implementations.

```rust
fn tick(&mut self, deps: TaskOutcomes) -> YieldUpdate
```

Executes one tick of work (bounded time quantum). Receives outcomes of all
dependencies and returns an update indicating whether the task is finished, waiting
for new dependencies, or ready to continue. Must checkpoint internal state via &mut
self to support resumption after preemption.

```rust
fn size(&self) -> Option<u64>
```

Returns the task's estimated computational size for weighted scheduling policies.
Defaults to None, in which case policies may use average task size or uniform weights.

## State Machines

### Task State

Each task in the scheduler registry has a Progress state that governs its lifecycle.

**States:**

- **Ready**: Task has no unsatisfied dependencies and is eligible for execution. Can be
  selected by policy for dispatch.

- **Running**: Task is currently executing on the runner. The executable has been
  transferred from buffer to runner.

- **Preempting**: Task has been marked for preemption but has not yet been collected
  from the runner. Intermediate state between Running and Ready during preemption.

- **Waiting(Dependencies)**: Task is blocked waiting for one or more dependencies to
  complete. The set of TaskIDs represents tasks that must finish before this one can
  proceed.

- **Finished(TaskOutcome)**: Task has completed execution. The outcome indicates
  success, failure, or error. Terminal state - task will not transition further.

- **Error**: Task encountered an internal failure during execution. May transition to
  Ready if policy decides to retry.

**Transitions:**

- Ready → Running: Execute phase selects task and dispatches to runner
- Running → Preempting: Preempt phase identifies task, calls runner.preempt() to signal
  preemption, and marks task as Preempting
- Preempting → Ready: Collect phase polls preempted task until it yields, collects the
  executable, and transitions to Ready
- Running → Waiting: Collect phase processes yield with new dependencies
- Running → Finished: Collect phase processes yield indicating completion
- Running → Error: Collect phase detects task panic and transitions to Error
- Error → Ready: Retry phase selects task for retry
- Waiting → Ready: Dependency resolution detects all dependencies satisfied

**Invariants:**

- Only Ready tasks can transition to Running
- Only Running tasks can be marked for preemption
- Only Running or Preempting tasks can be polled and collected
- Only Error tasks can be retried
- Finished tasks never transition
- Tasks in buffer must be Ready, Waiting, Error, or Finished (never Running or Preempting)
- Tasks on runner must be Running or Preempting

### Yield Intention

When a task yields (returns from tick()), it communicates its intention through a
YieldIntention enum that determines its next Progress state.

**Intentions:**

- **Ready**: Task has more work to do and should remain Ready after yield. Used for
  voluntary preemption points or when task wants to yield but continue later.

- **Waiting(Dependencies)**: Task is blocked on new dependencies discovered during
  execution. Transitions to Waiting state.

- **Finished(TaskOutcome)**: Task has completed all work. Transitions to Finished state
  with the provided outcome.

**Mapping to Progress:**

- YieldIntention::Ready → Progress::Ready
- YieldIntention::Waiting(deps) → Progress::Waiting(deps)
- YieldIntention::Finished(outcome) → Progress::Finished(outcome)

The scheduler processes yield intentions during collect phase and transitions tasks
accordingly.

### Task Outcome

TaskOutcome represents the final result of a completed task.

**Outcomes:**

- **Success(OutcomeCode)**: Task completed successfully. The code is domain-specific and
  can be used by dependent tasks to make execution decisions.

- **Failure(OutcomeCode)**: Task completed but determined its goal could not be
  achieved. The code indicates the type of logical failure. Distinct from Error - this
  is a valid completion state.

- **Error**: Task encountered an unexpected internal error. Should not occur in
  well-behaved executables but included for robustness.

Task outcomes are collected and passed to dependent tasks via the TaskOutcomes parameter
in tick(). This enables data-flow style dependencies where task behavior depends on
how predecessors completed.
