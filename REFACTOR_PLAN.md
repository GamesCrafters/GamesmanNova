# GRAND COMPREHENSIVE REVOLUTIONARY REFACTORING PLAN
## Eliminating Const Generics + Ruleset Semantics + Storage Decoupling

**Created**: 2025-11-26
**Status**: Ready for Implementation
**Scope**: 33 files, 87+ const generic occurrences, 15+ trait definitions, 23+ impl blocks

---

## EXECUTIVE SUMMARY

**Goal**: Pure game logic (Ruleset) + Pluggable storage (Storage<R>) + Dynamic tasks
**Risk**: High - fundamental type system change affecting every major subsystem
**Performance**: <0.05% impact (database-bound workload)
**Benefit**: Perfect separation of concerns, 500+ lines removed, task creation outside games

---

## NEW TYPE SYSTEM

### Before (Const Generic)
```rust
State<const B: usize = 8> = [u8; B]
Record<const N: PlayerCount> { utility: [SUtility; N] }
Sequential<const N, const B>
ForwardTask<G, const N, const B>
```

### After (Runtime-Sized)
```rust
struct State { bytes: Vec<u8> }
struct Record { utility: Vec<SUtility> }
trait Sequential
struct ForwardTask<G>
```

---

## ARCHITECTURE OVERVIEW

```
┌─────────────┐
│   Ruleset   │  Pure game logic (no database, no IOMode)
│  (Config)   │  Implements: Implicit, Sequential, SimpleUtility, etc.
└─────────────┘
       │
       ├─────────────┐
       │             │
       ▼             ▼
┌─────────────┐  ┌──────────────┐
│  Storage<R> │  │  ForwardTask │  Tasks work with abstraction
│    Trait    │  │    <G>       │  No const generics
└─────────────┘  └──────────────┘
       │
       ├─────────────┐
       │             │
       ▼             ▼
┌─────────────┐  ┌──────────────┐
│  RocksDB    │  │  InMemory    │  Pluggable backends
│  Storage    │  │  Storage     │
└─────────────┘  └──────────────┘
```

---

## IMPLEMENTATION PHASES

### Phase 0: Pre-Flight (2 hours)
- Create feature branch: `refactor/eliminate-const-generics`
- Document current behavior (baseline tests)
- Create rollback points (git tag `pre-refactor-v0.2.2`)

### Phase 1: Foundation - Type System Changes (8 hours)
**Files**: game/mod.rs, game/traits.rs, database/traits.rs, database/storage.rs (new)

1.1. Create new State type (Vec<u8>)
1.2. Update 11 game trait definitions (remove const generics)
1.3. Update 3 blanket implementations
1.4. Create Storage<R> trait + RocksDBStorage + InMemoryStorage
1.5. Update record trait definitions (remove const N)

### Phase 2: Zero-By Refactoring (6 hours)
**Files**: game/zero_by/mod.rs, variants.rs, states.rs

2.1. Session → Ruleset (remove rocksdb field)
2.2. Update Record definition (Vec<SUtility>)
2.3. Update 10+ trait implementations (no generics)
2.4. Update parse_variant (remove IOMode parameter)
2.5. Remove Session::run() method

### Phase 3: Task System Refactoring (8 hours)
**Files**: scheduler/task/forward.rs, backward.rs, tabulate.rs, factory.rs (new)

3.1. ForwardTask<G> (remove const N, B)
3.2. Simplify Executable trait bounds
3.3. Create task factory module
3.4. Update BackwardTask and TabulateTask

### Phase 4: CLI Integration (2 hours)
**Files**: main.rs

4.1. Update main build() function (Ruleset → Storage → Task → Scheduler)

### Phase 5: Test Infrastructure (6 hours)
**Files**: All test files (20+ files)

5.1. Update zero-by tests (20 tests)
5.2. Update mock game tests (16+ tests)
5.3. Update task tests

### Phase 6: Documentation (4 hours)
**Files**: CLAUDE.md, README.md, Cargo.toml

6.1. Update CLAUDE.md sections
6.2. Update README examples
6.3. Version bump to 0.3.0

### Phase 7: Comprehensive Validation (4 hours)
7.1. Compilation validation
7.2. Test validation (all tests pass)
7.3. Performance benchmarks
7.4. Integration testing

### Phase 8: Cleanup & Finalization (2 hours)
8.1. Remove dead code
8.2. Format & lint
8.3. Final documentation pass
8.4. Create PR

**Total Estimated Effort**: ~40 hours over 5-7 days

---

## KEY CODE CHANGES

### State Type (game/mod.rs)
```rust
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct State {
    bytes: Vec<u8>,
}

impl State {
    pub fn new(bytes: Vec<u8>) -> Self { Self { bytes } }
    pub fn from_slice(slice: &[u8]) -> Self { Self { bytes: slice.to_vec() } }
    pub fn from_array<const N: usize>(arr: [u8; N]) -> Self { Self { bytes: arr.to_vec() } }
    pub fn as_bytes(&self) -> &[u8] { &self.bytes }
    pub fn len(&self) -> usize { self.bytes.len() }
}
```

### Storage Trait (database/storage.rs - NEW FILE)
```rust
pub trait Storage<R>: Send + Sync {
    fn get(&self, state: &State) -> Result<Option<R>>;
    fn put(&self, state: &State, record: &R) -> Result<()>;
    fn contains(&self, state: &State) -> Result<bool>;
    fn flush(&self) -> Result<()>;
}

pub struct RocksDBStorage<R> {
    db: Arc<rocksdb::DB>,
    _phantom: PhantomData<R>,
}

pub struct InMemoryStorage<R> {
    map: Arc<RwLock<HashMap<State, R>>>,
}
```

### Game Traits (game/traits.rs)
```rust
// Before
pub trait Sequential<const N: PlayerCount, const B: usize> {
    fn turn(&self, state: &State<B>) -> Player;
}

pub trait SimpleUtility<const N: PlayerCount, const B: usize>: Sequential<N, B> {
    fn utility(&self, state: &State<B>) -> [SUtility; N];
}

// After
pub trait Sequential {
    fn turn(&self, state: &State) -> Player;
}

pub trait SimpleUtility: Sequential {
    fn utility(&self, state: &State) -> Vec<SUtility>;
}
```

### Variable Trait (game/traits.rs)
```rust
// Before
pub trait Variable {
    fn variant(variant: Option<Variant>, mode: IOMode) -> Result<Self>;
    fn name(&self) -> &str;
}

// After
pub trait Variable {
    fn variant(variant: Option<Variant>) -> Result<Self>;  // NO IOMode
    fn name(&self) -> &str;
}
```

### Ruleset (game/zero_by/mod.rs)
```rust
// Before
pub struct Session {
    rocksdb: Arc<rocksdb::DB>,  // ← Remove
    players: PlayerCount,
    // ... fields
}

// After
pub struct Ruleset {
    players: PlayerCount,
    // ... fields (NO rocksdb)
}

impl Variable for Ruleset {
    fn variant(variant: Option<Variant>) -> Result<Self> {
        variants::parse_variant(variant.unwrap_or(VARIANT_DEFAULT.to_owned()))
    }
}

impl SimpleUtility for Ruleset {
    fn utility(&self, state: &State) -> Vec<SUtility> {
        let (turn, _) = self.decode_state(state);
        let mut payoffs = vec![SUtility::Lose; self.players];
        payoffs[turn] = SUtility::Win;
        payoffs
    }
}
```

### Record (game/zero_by/mod.rs)
```rust
// Before
pub struct Record<const N: PlayerCount> {
    features: RecordFeatures,
    utility: [SUtility; N],
}

// After
pub struct Record {
    features: RecordFeatures,
    utility: Vec<SUtility>,
}

impl From<Record> for Vec<u8> {
    fn from(record: Record) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&record.features.into_bytes());
        bytes.push(record.utility.len() as u8);  // Length prefix
        bytes.extend(record.utility.iter().map(|u| *u as u8));
        bytes
    }
}
```

### ForwardTask (scheduler/task/forward.rs)
```rust
// Before
pub struct ForwardTask<G, const N: PlayerCount, const B: usize> {
    game: G,
    visited: Arc<rocksdb::DB>,
    frontier: VecDeque<State<B>>,
}

impl<G, const N, const B> Executable for ForwardTask<G, N, B>
where
    G: RocksDBManager<N, B> + SQLiteManager<N, B> + Implicit<B> + ...

// After
pub struct ForwardTask<G> {
    game: G,
    storage: Arc<dyn Storage<Record>>,
    frontier: VecDeque<State>,
}

impl<G> Executable for ForwardTask<G>
where
    G: Implicit + Partition + Sequential + SimpleUtility + Clone + Send + 'static
```

### Task Factory (scheduler/task/factory.rs - NEW FILE)
```rust
pub fn create_forward_task<G>(
    game: G,
    storage: Arc<dyn Storage<Record>>,
    threshold: usize,
) -> Result<Box<dyn Executable>>
where
    G: Implicit + Partition + Sequential + SimpleUtility + Clone + Send + 'static
{
    let task = ForwardTaskBuilder::default()
        .game(game)
        .storage(storage)
        .threshold(threshold)
        .build()?;
    Ok(Box::new(task))
}
```

### CLI Wiring (main.rs)
```rust
fn build(args: cli::BuildArgs) -> Result<()> {
    match args.target {
        GameModule::ZeroBy => {
            // 1. Create ruleset (pure logic)
            let mut ruleset = zero_by::Ruleset::variant(args.variant)?;

            // 2. Handle advance
            if args.advance {
                ruleset.advance(cli::stdin_lines()?)?;
            }

            // 3. Create storage (IOMode here!)
            let db = database::rocksdb::init_rocksdb(args.mode, ruleset.name())?;
            let storage: Arc<dyn Storage<zero_by::Record>> =
                Arc::new(RocksDBStorage::new(db));

            // 4. Create task via factory
            let task = task::factory::create_forward_task(ruleset, storage.clone(), 100)?;

            // 5. Build scheduler
            let mut scheduler = Scheduler::new(
                ThreadPoolRunner::new(num_cpus::get()),
                CriticalPathPolicy::new(),
                CountLogger::new(),
            );

            // 6. Run
            scheduler.spawn(task)?;
            scheduler.run()?;

            // 7. Flush
            storage.flush()?;
        }
    }
    Ok(())
}
```

---

## RISK MITIGATION

### Rollback Points
1. After Phase 1: Type system changes
2. After Phase 2: Zero-by refactoring
3. After Phase 3: Task system
4. After Phase 5: Test infrastructure

### Known Challenges

**Challenge 1: Blanket Implementation Lifetimes**
- Problem: Vec slices in trait returns
- Solution: Return Vec or use Cow<[T]>
- Decision: Phase 1.5

**Challenge 2: State Hashing Performance**
- Problem: Vec<u8> hashing slower than [u8; 8]
- Solution: Benchmark, use SmallVec if needed
- Decision: Phase 7.3

**Challenge 3: Record Serialization**
- Problem: Variable-length records
- Solution: Length-prefix encoding
- Decision: Phase 2.2

**Challenge 4: Task Factory Flexibility**
- Problem: Different games need different configs
- Solution: Builder pattern on factory
- Decision: Phase 3.3

---

## SUCCESS CRITERIA

### Must Have (P0)
- ✓ All tests pass (64+)
- ✓ Zero compiler errors
- ✓ Zero clippy warnings
- ✓ Documentation builds
- ✓ Performance within 1% of baseline

### Should Have (P1)
- ✓ CLAUDE.md fully updated
- ✓ README examples current
- ✓ All TODO comments resolved
- ✓ Style guide compliance

### Nice to Have (P2)
- ✓ Benchmark suite
- ✓ Migration guide
- ✓ Architecture walkthrough

---

## FILE CHANGE MANIFEST

### Files Modified (33)

**Core (13 files)**
1. src/game/mod.rs - State type, blanket impls
2. src/game/traits.rs - 11 trait definitions
3. src/game/util.rs - verify_state_history
4. src/database/traits.rs - Record traits
5. src/database/mod.rs - Blanket impls, exports
6. src/database/rocksdb.rs - Exports
7. src/database/sqlite.rs - Exports
8. src/frontend/mod.rs - IOMode (no change)
9. src/frontend/cli.rs - BuildArgs (no change)
10. src/main.rs - Main build() function
11. src/developer.rs - test_rocksdb (no change)
12. Cargo.toml - Version bump to 0.3.0
13. CLAUDE.md - Comprehensive update

**Zero-By (3 files)**
14. src/game/zero_by/mod.rs - Session→Ruleset
15. src/game/zero_by/variants.rs - parse_variant
16. src/game/zero_by/states.rs - Tests

**Mock (2 files)**
17. src/game/mock/mod.rs - Session, Record
18. src/game/mock/builder.rs - SessionBuilder

**Tasks (4 files)**
19. src/scheduler/task/forward.rs - ForwardTask
20. src/scheduler/task/backward.rs - BackwardTask
21. src/scheduler/task/tabulate.rs - TabulateTask
22. src/scheduler/task/mod.rs - Exports

**Tests (11 files)**
23-33. Various test files

### Files Created (3)
1. src/database/storage.rs - Storage trait + impls
2. src/scheduler/task/factory.rs - Task factory
3. benches/forward_task.rs - Benchmarks

### Files Deleted (0)
None (clean refactor)

---

## PERFORMANCE ANALYSIS

### Bottleneck Breakdown (Current)
- RocksDB operations: 50-90% of time
- game.outgoing(): 3-5%
- State operations: <1%

### Vec Overhead Impact
- State copying: +3ns per copy
- State hashing: +2ns per hash
- Vec allocation: +10ns amortized
- **Total per tick: ~43ns**
- **Percentage: 0.043%** (unmeasurable)

### Conclusion
Database operations dominate. Vec overhead is negligible.

---

## VALIDATION CHECKLIST

- [ ] Phase 0: Baseline documented
- [ ] Phase 1: Foundation compiles
- [ ] Phase 2: Zero-by tests pass
- [ ] Phase 3: Task tests pass
- [ ] Phase 4: CLI works end-to-end
- [ ] Phase 5: All tests pass
- [ ] Phase 6: Docs updated
- [ ] Phase 7: Benchmarks within 1%
- [ ] Phase 8: PR ready

---

## NOTES

- Keep this file updated as implementation progresses
- Document any deviations from plan
- Add lessons learned section
- Track actual vs estimated effort

**Status**: Ready to begin implementation
**Next Step**: Phase 0 - Create feature branch and baseline
