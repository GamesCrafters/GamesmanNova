# RUNNING-After-Finish Bug: Experimental Observations

## Problem Statement

Tasks remain in RUNNING state for astronomical periods (10+ seconds) after finishing work. The task's `tick()` method at forward.rs:260 gets called repeatedly even though it yields `Suspended` each time.

## Experimentally Verified Observations

(These are based on user's experimentation and debugging, but may not be universally true)

### Execution Trace

**forward.rs:260 gets hit over and over**
- This line returns `Some(YieldUpdate)` with `YieldIntention::Suspended`
- Occurs inside the `else` block when `frontier.pop_back()` returns `None`
- Indicates the task has no more states to explore

**thread.rs:531 returns Yielded WorkResult**
- Worker thread's `execute_task()` function calls `executable.tick()`
- When tick returns `Some(yield_update)`, worker returns `WorkResult::Yielded(yield_update)`
- This is the normal completion path

**thread.rs:494 sends CompletionPacket**
- Worker sends the completion packet via channel: `packet.result_tx.send(CompletionPacket { ... })`
- This should signal to scheduler that task is done

**thread.rs:172 during scheduler/mod.rs:475**
- The CompletionPacket gets collected by `process_completed()` which drains `result_rx.try_recv()`
- This happens during the Collection phase of scheduler's tick

**scheduler/mod.rs:475 should mark as ineligible**
- Collection phase at line 475 calls `collect()`
- This should collect completed tasks and mark them as ineligible for rescheduling
- Uses `collect_runner_ids()` to snapshot Running/Preempting tasks

**scheduler/mod.rs:487 reschedules task**
- Execute phase at line 487 calls `execute()`
- Apparently the task becomes eligible again and gets re-dispatched
- This causes the loop to repeat

### Observable Behavior

**Logger sees task as RUNNING the whole time**
- Logger only observes state after all tick phases complete
- No state transition appears to happen from logger's perspective
- Task remains in RUNNING state throughout the bug duration

**Eventually transitions to SUSPENDED**
- After a long time (10+ seconds), the bug stops
- Task properly transitions to SUSPENDED state
- The program continues normally after this

**Time proportional to states explored**
- Games with more states exhibit longer RUNNING-after-finish duration
- Suggests the tight loop executes approximately once per explored state
- The relationship is roughly linear

### Isolation Constraints

**Happens with single ForwardTask**
- Only one task is ever registered with the scheduler
- No children are spawned by the task
- Zero-by assigns all states to component 0 for small games
- No merge operations occur

**Tight loop after exploration done**
- Bug only manifests after the task has finished exploring states
- Does not happen during active state exploration
- Task's frontier is empty when the loop begins

**Not I/O, deadlocks, or actual progress**
- Bug occurs even with in-memory storage (no disk I/O)
- No threads are deadlocked
- Task is not making forward progress on game exploration

**Program actually hangs**
- This is not a dashboard or logging display issue
- Happens even when no logger is attached
- CPU usage remains high during the hang

### Location Hints

**Likely in scheduler/mod.rs and thread.rs**
- The bug involves scheduler/runner interaction
- Specifically Collection phase and Execute phase

**Bug is practically deterministic**
- Consistently reproduces under the same conditions
- Not a random race condition

### Game-Specific Trigger

**Requires '1' in the choices list**
- `2-100000-1` (choices=[1]) exhibits the bug
- `2-100000-2` (choices=[2]) does NOT exhibit the bug
- The ability to remove exactly 1 element is necessary for the bug to occur

**Why this matters for zero-by:**
- When '1' is in choices, states like (1, player0) and (1, player1) are reachable non-terminal states
- When only '2' is in choices, state (1, player) is either unreachable or terminal
- This affects the depth and structure of the game graph

**Component assignment:**
- Zero-by uses `elements / 1000000` for component assignment
- For small games (e.g., 2-100000-1), all states map to component 0
- This guarantees only one ForwardTask exists with no children spawned

## Why These Observations Matter

1. The tight loop involves the task yielding Suspended repeatedly
2. The worker correctly completes and sends a packet
3. The scheduler's Collection phase should handle this but doesn't prevent re-execution
4. The Execute phase somehow sees the task as eligible again
5. The proportionality to states explored suggests the loop iterates once per previously explored state
6. The game-specific trigger ('1' in choices) suggests something about the game graph structure matters
7. The eventually-completes behavior means there's a counter or accumulator that eventually drains
