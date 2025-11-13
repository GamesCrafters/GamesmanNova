# GamesmanNova Style Guide

**This style guide is mandatory for all code contributions. Read completely before writing any code.**

---

## 1. Core Principles

### Code is Self-Documenting
- **No comments in code bodies** (implementation details, explanations, TODOs)
- **Doc comments only for public API** (`///` for items, `//!` for modules)
- Code clarity comes from naming, structure, and decomposition—not comments

### Iterator Preference
- **Always prefer iterators over loops** when possible
- Use combinator chains (`map`, `filter`, `flat_map`, `fold`, etc.)
- Avoid manual mutation and index management

### Flat, Shallow Structure
- **Avoid multi-level indentation** (3+ levels is a code smell)
- Extract local variables to flatten nesting
- Extract methods to reduce complexity
- Use early returns and guard clauses

### Short Methods 
- **Methods start becoming long at ~40 lines** (soft limit, not absolute)
- If a method does multiple things, decompose it
- Each method should have a single, clear purpose
- **Exception**: Long methods are acceptable for highly procedural code (e.g., complex object construction with many components)
- **Use braces `{}` to create scopes** for semantic grouping and lifetime management in procedural methods

### Visual Appeal and Regularity 
- **Code should form uniform, regular blocks**
- Organize items by DECREASING length when order doesn't matter (longest first)
- Apply length ordering to: imports, match arms, fields, constants, enum variants
- Sort by semantic meaning first, then by length within semantic groups
- Semantic line breaks for readability
- Aim for "visual harmony" in the code layout

### Concise Naming
- **Methods**: 1-2 words maximum
- **Local variables**: 1 word (abbreviations allowed)
- **Struct fields**: 1 word
- **Types**: 1-2 words maximum
- Short names force precision and reduce visual noise

### Module-Local Utilities
- **Extensive use of module-local `impl T`** for utility methods
- Keeps public implementations clean and semantically focused
- Private helpers in separate impl blocks

### 80 Character Hard Limit
- **Enforced by rustfmt**: `max_width = 80`
- No exceptions
- Forces concise expression and proper decomposition

### Conservative Visibility
- **Visibility is a first-class concern**
- Default to private (`pub(crate)` or no visibility modifier)
- Only expose what is necessary for the public API
- Think carefully before making anything `pub`

### External Crate Preference
- **Always prefer well-maintained external crates** over custom implementations
- Use derive macros, builders, and utilities aggressively
- Don't reinvent the wheel

### Module Organization
- **No empty `mod.rs` files**
- If `mod.rs` would only contain `mod` declarations, use nested declarations in parent:
  ```rust
  mod parent {
      mod child1;
      mod child2;
  }
  ```

### Loop Preference
- **Prefer `while let` over `loop {}`** unless absolutely necessary
- `loop {}` should be rare and have a clear justification

### Path Specifications
- **NEVER use inline path specifications** (e.g., `std::collections::HashMap::new()`)
- **Always prefer `use` statements** at the top of the file
- **Exception**: In top-level modules that integrate many submodules, ONE level of specification is acceptable to avoid ambiguity
  - Example: `mock::Session` when multiple modules have a `Session` type
  - Use sparingly and only when clarity demands it

---

## 2. File Organization Template

Every Rust file should follow this structure with section separators at zero-indentation.

### Section Separators (in strict order)

The following separators MUST appear in this exact order at zero-indentation level:

1. `/* CONSTANTS */` - Module-level constants
2. `/* TYPE ALIASES */` - Type aliases (e.g., `type Result<T> = ...`)
3. `/* ENUMERATIONS */` - Enum definitions
4. `/* API STRUCTURES */` - Public structs with doc comments
5. `/* STRUCTURES */` - Private/internal structs
6. `/* IMPLEMENTATIONS */` - All impl blocks

**Note**: Not all sections need to be present. Only include sections that have content. Within `/* IMPLEMENTATIONS */`, you may use subsection comments like `/* Utilities for X */`.

### Complete Template Example

```rust
use std::collections::HashMap;
use std::sync::Arc;

use external_crate::OtherType;
use external_crate::Type;

use crate::module::LocalType;
use crate::other::Thing;

/* CONSTANTS */

const DEFAULT_TIMEOUT: u64 = 30;
const MAX_SIZE: usize = 1024;

/* TYPE ALIASES */

type Result<T> = std::result::Result<T, Error>;
type TaskMap = HashMap<TaskID, Task>;

/* ENUMERATIONS */

pub enum State {
    Waiting,
    Running,
    Ready,
}

/* API STRUCTURES */

/// Public API structure with documentation.
pub struct Scheduler {
    policy: Box<dyn Policy>,
    tasks: TaskMap,
}

/* STRUCTURES */

struct Task {
    state: State,
    size: u64,
    id: TaskID,
}

/* IMPLEMENTATIONS */

impl Scheduler {
    pub fn new(policy: Box<dyn Policy>) -> Self {
        Self {
            tasks: HashMap::new(),
            policy,
        }
    }

    pub fn tick(&mut self) -> Result<()> {
        self.collect()?;
        self.resolve()?;
        self.execute()?;
        Ok(())
    }
}

/* Utilities for Scheduler */

impl Scheduler {
    fn collect(&mut self) -> Result<()> {
        Ok(())
    }

    fn resolve(&mut self) -> Result<()> {
        Ok(())
    }

    fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}
```

**Key Points**:
- Section separators are `/* SECTION_NAME */` at zero-indentation
- Separators appear in the order listed above (skip sections with no content)
- Within `/* IMPLEMENTATIONS */`, utility impls can have subsection comments
- Imports grouped: std → external → local (by module distance), then by DECREASING length within each group
- Fields, enum variants, constants sorted by DECREASING length when semantics allow
- Braces on same line (stable rustfmt behavior)

---

## 3. Naming Conventions

### Methods: 1-2 Words

**Good**:
```rust
fn tick(&mut self) -> Result<()>
fn collect(&mut self) -> Vec<Task>
fn validate(&self, state: &State) -> bool
fn compute_weight(&self) -> u64
```

**Bad**:
```rust
fn execute_the_next_tick_cycle(&mut self) -> Result<()>
fn collect_all_completed_tasks_from_runner(&mut self) -> Vec<Task>
fn validate_state_against_rules(&self, state: &State) -> bool
fn compute_weighted_priority_value(&self) -> u64
```

### Local Variables: 1 Word (abbreviations allowed)

**Good**:
```rust
let task = self.next();
let id = task.id();
let idx = self.find(key);
let len = items.len();
let cfg = Config::default();
```

**Bad**:
```rust
let current_task = self.next();
let task_identifier = task.id();
let found_index = self.find(key);
let items_length = items.len();
let configuration = Config::default();
```

### Struct Fields: 1 Word (length-ordered when semantics allow)

**Good**:
```rust
struct Task {
    deps: Vec<TaskID>,
    state: State,
    size: u64,
    id: TaskID,
}
```

**Bad**:
```rust
struct Task {
    task_id: TaskID,
    current_state: State,
    estimated_size: u64,
    dependencies: Vec<TaskID>,
}
```

### Types: 1-2 Words

**Good**:
```rust
struct Scheduler {}
struct Task {}
enum State {}
struct TaskID(u64);
struct PolicyContext {}
```

**Bad**:
```rust
struct TaskSchedulerCoordinator {}
struct ExecutableTask {}
enum TaskExecutionState {}
struct UniqueTaskIdentifier(u64);
struct PolicyDecisionContext {}
```

---

## 4. Iterator Preference

Always prefer iterator combinators over imperative loops.

### Bad: Imperative with Manual Mutation

```rust
fn process(&self, tasks: Vec<Task>) -> Vec<TaskID> {
    let mut results = Vec::new();
    for task in tasks {
        if task.is_ready() {
            let id = task.id();
            if !self.seen.contains(&id) {
                results.push(id);
            }
        }
    }
    results
}
```

**Problems**:
- Manual mutation (`results`)
- Multiple levels of nesting (3 levels)
- Imperative style obscures intent
- Visual noise

### Good: Iterator Combinators

```rust
fn process(&self, tasks: Vec<Task>) -> Vec<TaskID> {
    tasks
        .into_iter()
        .filter(|t| t.is_ready())
        .map(|t| t.id())
        .filter(|id| !self.seen.contains(id))
        .collect()
}
```

**Benefits**:
- No mutation
- Flat structure (no deep nesting)
- Declarative intent
- Visually clean

### Bad: Nested Loops for Transformation

```rust
fn flatten(&self, groups: Vec<Vec<Task>>) -> Vec<TaskID> {
    let mut ids = Vec::new();
    for group in groups {
        for task in group {
            ids.push(task.id());
        }
    }
    ids
}
```

### Good: Flat Iterator Chain

```rust
fn flatten(&self, groups: Vec<Vec<Task>>) -> Vec<TaskID> {
    groups
        .into_iter()
        .flat_map(|g| g.into_iter())
        .map(|t| t.id())
        .collect()
}
```

---

## 5. Avoiding Deep Indentation

Deep nesting (3+ levels) is a sign of poor decomposition. Flatten using multiple techniques.

### Bad: 4-Level Nesting

```rust
fn process(&mut self, tasks: Vec<Task>) -> Result<()> {
    for task in tasks {
        if let Some(exec) = task.executable() {
            if exec.is_valid() {
                match exec.tick() {
                    Some(update) => {
                        self.handle(update)?;
                    }
                    None => {
                        return Err(Error::NoUpdate);
                    }
                }
            }
        }
    }
    Ok(())
}
```

**Problems**:
- 4 levels of nesting
- Hard to follow control flow
- Difficult to extract or test

### Good: Flattened with Early Returns

```rust
fn process(&mut self, tasks: Vec<Task>) -> Result<()> {
    for task in tasks {
        self.process_one(task)?;
    }
    Ok(())
}

fn process_one(&mut self, task: Task) -> Result<()> {
    let exec = task.executable().ok_or(Error::NoExec)?;

    if !exec.is_valid() {
        return Ok(());
    }

    let update = exec.tick().ok_or(Error::NoUpdate)?;
    self.handle(update)
}
```

**Benefits**:
- Maximum 2 levels of nesting
- Clear control flow with early returns
- Each method has a single responsibility
- Easy to test `process_one` separately

### Bad: Nested Match

```rust
fn handle(&self, result: Result<Update>) -> Status {
    match result {
        Ok(update) => {
            match update.intention {
                Intention::Ready => {
                    match self.queue(update) {
                        Ok(_) => Status::Queued,
                        Err(e) => Status::Failed(e),
                    }
                }
                Intention::Waiting => Status::Waiting,
            }
        }
        Err(e) => Status::Error(e),
    }
}
```

### Good: Extracted Local Variables

```rust
fn handle(&self, result: Result<Update>) -> Status {
    let update = match result {
        Err(e) => return Status::Error(e),
        Ok(u) => u,
    };

    match update.intention {
        Intention::Ready => self.queue_or_fail(update),
        Intention::Waiting => Status::Waiting,
    }
}

fn queue_or_fail(&self, update: Update) -> Status {
    self.queue(update)
        .map(|_| Status::Queued)
        .unwrap_or_else(Status::Failed)
}
```

---

## 6. Visual Appeal & Length-Based Ordering

Code should form uniform, visually pleasing blocks. When order doesn't affect semantics, organize by DECREASING length (longest first). Always prioritize semantic grouping first.

### Bad: Jagged Imports (No Organization)

```rust
use std::collections::HashMap;
use std::sync::Arc;
use bitvec::BitArray;
use derive_builder::Builder;
use crate::scheduler::Scheduler;
use crate::task::Task;
```

**Problem**: Visually chaotic, no pattern, no grouping

### Good: Grouped and Length-Ordered Imports

```rust
use std::collections::HashMap;
use std::sync::Arc;

use derive_builder::Builder;
use bitvec::BitArray;

use crate::scheduler::Scheduler;
use crate::task::Task;
```

**Benefits**: Grouped (std → external → local), then by DECREASING length within each group, visual harmony

### Bad: Random Match Arm Order

```rust
match state {
    State::Ready => 0,
    State::Running => 1,
    State::Waiting(deps) => deps.len(),
    State::Error => u64::MAX,
    State::Suspended(outcome) => outcome.code(),
}
```

### Good: Length-Ordered Match Arms (Decreasing)

```rust
match state {
    State::Suspended(outcome) => outcome.code(),
    State::Waiting(deps) => deps.len(),
    State::Running => 1,
    State::Error => u64::MAX,
    State::Ready => 0,
}
```

### Bad: Random Field Order

```rust
struct Config {
    timeout: u64,
    name: String,
    max_retries: usize,
    enabled: bool,
    default_priority: u8,
}
```

### Good: Length-Ordered Fields (Decreasing, when semantics allow)

```rust
struct Config {
    default_priority: u8,
    max_retries: usize,
    timeout: u64,
    enabled: bool,
    name: String,
}
```

**Note**: Only reorder when it doesn't violate semantic grouping. If fields have logical relationships, group them semantically first, then apply length ordering within groups.

### Struct Initialization: Shorthand Fields Last

When initializing structs, field init shorthand (`field` instead of `field: field`) **always goes last**, regardless of length:

**Good**:
```rust
Self {
    tasks: HashMap::new(),
    policy,
    runner,
}
```

**Bad**:
```rust
Self {
    policy,
    tasks: HashMap::new(),
    runner,
}
```

**Rule**: Sort explicit field assignments (`field: value`) by decreasing length, then place all shorthand fields at the end (also sorted by decreasing length among themselves).

### Semantic Grouping Example

When fields have semantic relationships, group by meaning first:

```rust
struct Task {
    // Identity (semantic group)
    name: String,
    id: TaskID,

    // Execution state (semantic group, length-ordered within)
    dependencies: Vec<TaskID>,
    state: State,
    size: u64,

    // Timing (semantic group, length-ordered within)
    started_at: Option<Instant>,
    duration: Duration,
}
```

---

## 7. Module-Local `impl T` Pattern

Public implementations should be clean and focused. Move utilities to module-local impl blocks.

### Bad: Bloated Public Impl

```rust
pub struct Scheduler {
    policy: Box<dyn Policy>,
    tasks: HashMap<TaskID, Task>,
}

impl Scheduler {
    pub fn new(policy: Box<dyn Policy>) -> Self {
        Self {
            tasks: HashMap::new(),
            policy,
        }
    }

    pub fn tick(&mut self) -> Result<()> {
        let completed = self.get_completed_tasks();
        self.update_internal_state(completed);

        let ready = self.find_ready_tasks();
        self.schedule_with_policy(ready);

        Ok(())
    }

    fn get_completed_tasks(&self) -> Vec<Task> {
        vec![]
    }

    fn update_internal_state(&mut self, tasks: Vec<Task>) {
    }

    fn find_ready_tasks(&self) -> Vec<Task> {
        vec![]
    }

    fn schedule_with_policy(&mut self, tasks: Vec<Task>) {
    }
}
```

**Problems**:
- Public and private methods mixed
- Hard to see the public API
- Visually cluttered

### Good: Clean Public Impl + Utility Impl

```rust
pub struct Scheduler {
    policy: Box<dyn Policy>,
    tasks: HashMap<TaskID, Task>,
}

/* Public API */

impl Scheduler {
    pub fn new(policy: Box<dyn Policy>) -> Self {
        Self {
            tasks: HashMap::new(),
            policy,
        }
    }

    pub fn tick(&mut self) -> Result<()> {
        let completed = self.completed();
        self.update(completed);

        let ready = self.ready();
        self.schedule(ready);

        Ok(())
    }
}

/* Utilities for Scheduler */

impl Scheduler {
    fn completed(&self) -> Vec<Task> {
        vec![]
    }

    fn update(&mut self, tasks: Vec<Task>) {
    }

    fn ready(&self) -> Vec<Task> {
        vec![]
    }

    fn schedule(&mut self, tasks: Vec<Task>) {
    }
}
```

**Benefits**:
- Public API is immediately visible
- Clear separation of concerns
- Easier to navigate
- Utilities can be further organized with subsection comments

---

## 8. Method Length & Decomposition

Methods become long around 40 lines. Decompose into focused, single-purpose methods.

### Bad: 60-Line Method Doing Multiple Things

```rust
fn build(&mut self, game: &Game, db: &mut Database) -> Result<()> {
    let start = game.source();
    let mut queue = vec![start];
    let mut visited = HashSet::new();

    while let Some(state) = queue.pop() {
        if visited.contains(&state) {
            continue;
        }
        visited.insert(state);

        let adj = game.adjacent(&state);
        for next in adj {
            if !visited.contains(&next) {
                queue.push(next);
            }
        }

        if game.sink(&state) {
            let utility = game.utility(&state);
            let record = Record {
                remoteness: 0,
                utility,
                state,
            };
            db.write(&record)?;
        }
    }

    let transposed = game.transpose();
    let mut changed = true;

    while changed {
        changed = false;
        for state in &visited {
            let pred = transposed.adjacent(state);
            let values: Vec<_> = pred
                .iter()
                .filter_map(|s| db.read(s).ok())
                .collect();

            if values.is_empty() {
                continue;
            }

            let best = values
                .iter()
                .max_by_key(|r| r.remoteness)
                .unwrap();

            let current = db.read(state).ok();
            let needs_update = current.is_none()
                || current.unwrap().remoteness != best.remoteness + 1;

            if needs_update {
                changed = true;
                let record = Record {
                    utility: best.utility.invert(),
                    remoteness: best.remoteness + 1,
                    state: *state,
                };
                db.write(&record)?;
            }
        }
    }

    Ok(())
}
```

**Problems**:
- 60+ lines doing exploration + solving
- Multiple responsibilities
- Hard to understand, test, or modify

### Good: Decomposed into Focused Methods

```rust
fn build(&mut self, game: &Game, db: &mut Database) -> Result<()> {
    let states = self.explore(game)?;
    self.initialize(game, db, &states)?;
    self.propagate(game, db, &states)?;
    Ok(())
}

fn explore(&self, game: &Game) -> Result<HashSet<State>> {
    let start = game.source();
    let mut visited = HashSet::new();
    let mut queue = vec![start];

    while let Some(state) = queue.pop() {
        if visited.insert(state) {
            let adj = game.adjacent(&state);
            let new = adj.into_iter().filter(|s| !visited.contains(s));
            queue.extend(new);
        }
    }

    Ok(visited)
}

fn initialize(
    &self,
    game: &Game,
    db: &mut Database,
    states: &HashSet<State>,
) -> Result<()> {
    states
        .iter()
        .filter(|s| game.sink(s))
        .map(|s| self.terminal_record(game, s))
        .try_for_each(|r| db.write(&r))
}

fn propagate(
    &self,
    game: &Game,
    db: &mut Database,
    states: &HashSet<State>,
) -> Result<()> {
    let transposed = game.transpose();

    while self.propagate_once(&transposed, db, states)? {}

    Ok(())
}

fn propagate_once(
    &self,
    game: &Game,
    db: &mut Database,
    states: &HashSet<State>,
) -> Result<bool> {
    let mut changed = false;

    for state in states {
        if self.update_from_children(game, db, state)? {
            changed = true;
        }
    }

    Ok(changed)
}
```

**Benefits**:
- Each method < 30 lines
- Single responsibility per method
- Easy to understand each step
- Easy to test individually
- Clear naming communicates intent

### Exception: Procedural Construction Methods

Long methods (>40 lines) are acceptable when implementing **highly procedural code**, particularly complex object construction with many interdependent components.

**When this applies**:
- Building complex test fixtures with many components
- Constructing objects with extensive initialization requirements
- Procedural setup with sequential dependencies
- Each step is straightforward but numerous

**Use scoped blocks `{}` for**:
- **Semantic grouping**: Group related initialization steps
- **Lifetime management**: Drop temporary values early
- **Visual structure**: Create uniform, scannable blocks

### Good: Procedural Construction with Scoped Blocks

```rust
fn complex_scenario() -> Scenario {
    let scheduler = {
        let capacity = num_cpus::get();
        let runner = ThreadPoolRunner::new(capacity);
        let policy = CriticalPathPolicy::default();
        Scheduler::new(runner, policy)
    };

    let database = {
        let schema = SchemaBuilder::new("test")
            .column("remoteness", "INTEGER")
            .column("utility", "INTEGER")
            .players(2)
            .build()
            .unwrap();

        let path = temp_dir().join("scenario.db");
        SqliteDatabase::connect(&path, schema).unwrap()
    };

    let game = {
        let variant = "2-10-1-2";
        let session = ZeroBy::variant(variant).unwrap();
        let transposed = session.transpose();
        session
    };

    let tasks = {
        let explore = ExploreTask::new(game.source());
        let solve = SolveTask::new(&game);
        let store = StoreTask::new(&database);

        vec![
            (TaskID(0), Box::new(explore) as Box<dyn Executable>),
            (TaskID(1), Box::new(solve) as Box<dyn Executable>),
            (TaskID(2), Box::new(store) as Box<dyn Executable>),
        ]
    };

    let assertions = {
        let expected_states = 11;
        let expected_remoteness = 3;
        let expected_winner = Player(0);

        Assertions {
            expected_remoteness,
            expected_winner,
            expected_states,
        }
    };

    Scenario {
        assertions,
        scheduler,
        database,
        tasks,
        game,
    }
}
```

**Why this is acceptable**:
- Single clear purpose: build a complete test scenario
- Each block is semantically meaningful (scheduler, database, game, etc.)
- Scopes improve readability through visual grouping
- Temporary values dropped immediately (lifetimes managed)
- Procedural nature makes decomposition artificial
- Total complexity is low despite length

**Alternative (Not Better)**:
Breaking this into `build_scheduler()`, `build_database()`, `build_game()`, etc. would:
- Create methods used exactly once
- Scatter related initialization across file
- Obscure the holistic scenario setup
- Add navigation overhead for readers

**Rule**: Use this pattern sparingly. If the method has conditional logic, loops, or complex control flow, decompose it instead.

---

## 9. External Crates

Always prefer well-maintained external crates over custom implementations. These are the crates currently used in the project and when to use them:

### `derive_builder` - Struct Builders

**When to use**: Complex structs with many optional fields, test fixtures, public APIs with optional configuration.

```rust
use derive_builder::Builder;

#[derive(Builder)]
pub struct Config {
    name: String,

    #[builder(default = "30")]
    timeout: u64,

    #[builder(default)]
    enabled: bool,
}

let cfg = ConfigBuilder::default()
    .name("scheduler".into())
    .timeout(60)
    .build()?;
```

### `anyhow` - Error Handling

**When to use**: Application code (not libraries), when you need context-rich error propagation.

```rust
use anyhow::{Context, Result};

fn load(&self, path: &str) -> Result<Config> {
    let file = File::open(path)
        .context("Failed to open config file")?;

    let cfg: Config = serde_json::from_reader(file)
        .context("Failed to parse config JSON")?;

    Ok(cfg)
}
```

**Pattern**: Use `?` with `.context()` for informative error chains.

### `bitvec` - Bit Manipulation

**When to use**: Bit-packed state representations, efficient boolean collections.

```rust
use bitvec::prelude::*;

fn encode(&self, player: u8, elements: u16) -> [u8; 8] {
    let mut bits = BitArray::<[u8; 8]>::ZERO;
    bits[0..2].store(player);
    bits[2..18].store(elements);
    bits.into_inner()
}
```

### `modular-bitfield` - Bitfield Structs

**When to use**: Fixed-layout bitfield structures with named fields.

```rust
use modular_bitfield::prelude::*;

#[bitfield]
struct StateFlags {
    visited: bool,
    terminal: bool,
    player: B2,
    remoteness: B12,
}
```

### Other Key Crates

- **`clap`**: CLI argument parsing (use derive API)
- **`rusqlite`**: SQLite database access
- **`sled`**: Embedded key-value database
- **`petgraph`**: Graph algorithms and data structures
- **`threadpool`**: Thread pool for parallel execution
- **`mockall`**: Mock objects for testing
- **`tokio`**: Async runtime (use sparingly, prefer sync where possible)
- **`ratatui`**: TUI (terminal user interface) framework
- **`crossterm`**: Cross-platform terminal manipulation

**General Principle**: Before implementing custom functionality, search crates.io for a well-maintained solution.

---

## 10. Rustfmt Configuration (Stable Features)

The following rustfmt settings are enforced (stable features only):

### `max_width = 80`
**Hard line limit**. No exceptions. Forces concise expression and proper decomposition.

### `fn_params_layout = "Tall"`
Function parameters go on separate lines when they don't fit:
```rust
fn example(
    first: String,
    second: u64,
    third: Vec<Item>,
) -> Result<()>
```

### `merge_derives = true`
Combine multiple derive attributes:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
```

### `use_try_shorthand = true`
Use `?` operator instead of `try!` macro:
```rust
let file = File::open(path)?;
```

### `use_field_init_shorthand = true`
Use field shorthand when variable name matches:
```rust
let name = "scheduler".to_string();
let timeout = 30;

Config { name, timeout }  // not { name: name, timeout: timeout }
```

### `merge_imports = false`
Keep imports separate (one per line):
```rust
use std::fs::File;
use std::io::Read;
```

### `reorder_imports = true`
Imports within a group are alphabetically sorted.

### `match_block_trailing_comma = true`
Always use trailing commas in match blocks:
```rust
match state {
    State::Ready => 0,
    State::Running => 1,  // <- trailing comma
}
```

### `remove_nested_parens = true`
Remove unnecessary nested parentheses:
```rust
(x + y)  // not ((x + y))
```

**Note**: Unstable features in `rustfmt.toml` (like `brace_style = "AlwaysNextLine"`) are NOT enforced by the formatter. All code examples in this guide use stable formatting (braces on same line).

---

## 11. Visibility & API Design

Visibility is a **first-class concern**. Default to the most restrictive visibility possible.

### Bad: Everything Public

```rust
pub struct Scheduler {
    pub policy: Box<dyn Policy>,
    pub runner: Runner,
    pub tasks: HashMap<TaskID, Task>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            policy: Box::new(DefaultPolicy),
            runner: Runner::new(),
            tasks: HashMap::new(),
        }
    }
    pub fn tick(&mut self) {}
    pub fn collect(&mut self) {}
    pub fn resolve(&mut self) {}
    pub fn execute(&mut self) {}
}
```

**Problems**:
- Internal state exposed (`tasks`, `policy`, `runner`)
- Implementation details public (`collect`, `resolve`, `execute`)
- No encapsulation
- Future refactoring breaks users

### Good: Minimal Public Surface

```rust
pub struct Scheduler {
    policy: Box<dyn Policy>,
    runner: Runner,
    tasks: HashMap<TaskID, Task>,
}

impl Scheduler {
    pub fn new(policy: Box<dyn Policy>, runner: Runner) -> Self {
        Self {
            tasks: HashMap::new(),
            policy,
            runner,
        }
    }

    pub fn tick(&mut self) -> Result<()> {
        self.collect()?;
        self.resolve()?;
        self.execute()?;
        Ok(())
    }
}

impl Scheduler {
    fn collect(&mut self) -> Result<()> {
        Ok(())
    }

    fn resolve(&mut self) -> Result<()> {
        Ok(())
    }

    fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}
```

**Benefits**:
- Only `new()` and `tick()` are public (the actual API)
- Internal state hidden
- Implementation can change without breaking users
- Clear contract

### Visibility Levels (in order of restrictiveness)

1. **Private** (default): Only visible in current module
2. **`pub(crate)`**: Visible within the crate
3. **`pub(super)`**: Visible to parent module
4. **`pub`**: Visible to everyone

**Rule**: Use the most restrictive visibility that works. Only make things `pub` when they are part of the stable external API.

---

## 12. Module Organization

### No Empty `mod.rs` Files

If `mod.rs` would only contain `mod` declarations, use nested declarations in the parent module instead.

### Bad: Empty `mod.rs`

```
src/
  scheduler/
    mod.rs          <- only contains "pub mod policy; pub mod runner;"
    policy.rs
    runner.rs
```

```rust
// scheduler/mod.rs
pub mod policy;
pub mod runner;
```

### Good: Nested Declarations in Parent

```
src/
  lib.rs
  policy.rs
  runner.rs
```

```rust
// lib.rs
pub mod scheduler {
    pub mod policy;
    pub mod runner;
}
```

### When `mod.rs` Is Justified

When the module itself contains significant code beyond just declarations:

```rust
// scheduler/mod.rs

mod policy;
mod runner;

pub use policy::Policy;
pub use runner::Runner;

/* STRUCTURES */

pub struct Scheduler {
    policy: Box<dyn Policy>,
    runner: Runner,
}

/* IMPLEMENTATIONS */

impl Scheduler {
    pub fn new(policy: Box<dyn Policy>, runner: Runner) -> Self {
        Self { runner, policy }
    }

    pub fn tick(&mut self) -> Result<()> {
        Ok(())
    }
}
```

In this case, `mod.rs` is justified because it contains the `Scheduler` implementation, not just module declarations.

---

## 13. Loop Preference

Prefer `while let` patterns over bare `loop {}` blocks.

### Bad: Bare `loop {}`

```rust
fn process(&mut self) {
    loop {
        let item = match self.queue.pop() {
            Some(i) => i,
            None => break,
        };

        self.handle(item);
    }
}
```

### Good: `while let`

```rust
fn process(&mut self) {
    while let Some(item) = self.queue.pop() {
        self.handle(item);
    }
}
```

### When `loop {}` Is Justified

Only use `loop {}` when you truly need an unconditional loop with multiple exit points or complex break conditions:

```rust
fn run(&mut self) -> Result<()> {
    loop {
        match self.tick()? {
            Status::Continue => continue,
            Status::Pause => return Ok(()),
            Status::Shutdown => {
                self.cleanup()?;
                return Ok(());
            }
        }
    }
}
```

Even here, consider refactoring to eliminate the `loop {}` if possible.

---

## 14. Path Specifications

Always use `use` statements at the top of the file instead of inline path specifications.

### Bad: Inline Path Specifications

```rust
fn process(&self) -> Result<()> {
    let map = std::collections::HashMap::new();
    let path = std::path::PathBuf::from("data");
    let file = std::fs::File::open(path)?;

    Ok(())
}
```

**Problems**:
- Visual clutter
- Repetitive namespacing
- Harder to refactor
- Obscures actual logic

### Good: Use Statements

```rust
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

fn process(&self) -> Result<()> {
    let map = HashMap::new();
    let path = PathBuf::from("data");
    let file = File::open(path)?;

    Ok(())
}
```

**Benefits**:
- Clean, readable code
- Easy to see dependencies at top
- Simple to refactor imports
- Logic stands out

### Exception: One-Level Specification in Integrating Modules

In top-level modules that **integrate or use implementations from many submodules**, one level of path specification is acceptable to avoid ambiguity.

**When to use**: Multiple modules define the same type name (e.g., `Session`, `Config`, `Builder`).

**Good: One-Level Specification for Clarity**

```rust
use crate::game::mock;
use crate::game::zero_by;

fn test_games() {
    let mock_game = mock::Session::new();
    let zero_game = zero_by::Session::new();

    // Clear which Session we're using
    assert_ne!(mock_game.name(), zero_game.name());
}
```

**Bad: Two-Level or Deeper Specification**

```rust
fn test_games() {
    let mock_game = crate::game::mock::Session::new();
    let zero_game = crate::game::zero_by::Session::new();

    // Too much nesting
}
```

**Bad: Unclear Without Module Prefix**

```rust
use crate::game::mock::Session;
use crate::game::zero_by::Session;  // Error: conflicting names!

fn test_games() {
    // Which Session?
    let game = Session::new();
}
```

### Rule Summary

1. **Default**: Always use `use` statements, never inline paths
2. **Exception**: ONE level of specification (`module::Type`) in integrating modules when ambiguity exists
3. **Never**: Two or more levels of inline specification (`crate::module::submodule::Type`)

---

## Summary Checklist

Before submitting code, verify:

- [ ] No comments in code bodies (only doc comments `///` and `//!` for public API)
- [ ] Iterators preferred over manual loops
- [ ] No deep nesting (3+ levels)
- [ ] Methods under ~40 lines (exception: procedural construction with scoped blocks)
- [ ] Visual appeal: length-ordered DECREASING when possible, uniform blocks
- [ ] All names are 1-2 words maximum (methods, types) or 1 word (variables, fields)
- [ ] Abbreviations only for local variables
- [ ] Module-local `impl T` for utilities
- [ ] 80 character limit enforced (run `cargo fmt`)
- [ ] Conservative visibility (default to private, carefully consider `pub`, use `pub(in module)` if needed)
- [ ] External crates used where appropriate (derive_builder, anyhow, bitvec, etc.)
- [ ] No empty `mod.rs` files (use nested declarations)
- [ ] `while let` instead of `loop {}` where possible
- [ ] No inline path specifications (use `use` statements, except one-level in integrating modules)
- [ ] File organization follows template with /* SEPARATORS */ in correct order
- [ ] Imports grouped: std → external → local (by distance), length-ordered DECREASING
- [ ] Fields, constants, enum variants length-ordered DECREASING (when semantics allow)
- [ ] Struct initialization: shorthand fields always last (after explicit assignments)
- [ ] Semantic grouping prioritized over length-ordering
- [ ] Early returns and guard clauses to flatten nesting
- [ ] Each method has single, clear purpose
- [ ] Local variable extraction used to reduce complexity
- [ ] Scoped blocks `{}` used for semantic grouping in procedural construction
- [ ] Iterator combinators: map, filter, flat_map, fold used appropriately
- [ ] `anyhow::Context` used for error messages
- [ ] Builder pattern (derive_builder) for complex construction

**This style guide is non-negotiable. All code must conform before merging.**
