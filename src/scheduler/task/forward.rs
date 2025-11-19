//! # Explore Scheduler Task
//!
//! DFS-based exploration task for discovering game state graphs. Spawns child
//! tasks when crossing component boundaries, enabling parallel exploration of
//! independent graph components.

use std::any::Any;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::marker::PhantomData;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;

use crate::database::traits::SQLiteManager;
use crate::database::traits::SledManager;
use crate::game::Component;
use crate::game::DEFAULT_STATE_BYTES;
use crate::game::PlayerCount;
use crate::game::State;
use crate::game::traits::Implicit;
use crate::game::traits::IntegerUtility;
use crate::game::traits::Partition;
use crate::game::traits::Sequential;
use crate::game::traits::Variable;
use crate::scheduler::Dependencies;
use crate::scheduler::Task;
use crate::scheduler::TaskBuilder;
use crate::scheduler::TaskID;
use crate::scheduler::TaskIDBuilder;
use crate::scheduler::TaskCategory;
use crate::scheduler::TaskOutcome;
use crate::scheduler::TaskOutcomes;
use crate::scheduler::YieldIntention;
use crate::scheduler::YieldUpdate;
use crate::scheduler::YieldUpdateBuilder;
use crate::scheduler::traits::Executable;

/* STRUCTURES */

pub struct ForwardTask<
    G,
    const N: PlayerCount,
    const B: usize = DEFAULT_STATE_BYTES,
> {
    threshold: usize,
    component: Component,
    _phantom: PhantomData<[(); N]>,
    frontier: VecDeque<State<B>>,
    buffered: usize,
    explored: usize,
    progress: usize,
    pending: HashMap<Component, VecDeque<State<B>>>,
    visited: sled::Tree,
    game: G,
}

pub struct ForwardTaskBuilder<
    G,
    const N: PlayerCount,
    const B: usize = DEFAULT_STATE_BYTES,
> {
    threshold: Option<usize>,
    frontier: Option<VecDeque<State<B>>>,
    game: Option<G>,
}

/* IMPLEMENTATIONS */

impl<G, const N: PlayerCount, const B: usize> Default
    for ForwardTaskBuilder<G, N, B>
{
    fn default() -> Self {
        Self {
            threshold: None,
            frontier: None,
            game: None,
        }
    }
}

impl<G, const N: PlayerCount, const B: usize> ForwardTaskBuilder<G, N, B>
where
    G: SledManager<N, B>
        + SQLiteManager<N, B>
        + Implicit<B>
        + Sequential<N, B>
        + IntegerUtility<N, B>
        + Partition<B>
        + Variable
        + Clone
        + Send
        + 'static,
{
    pub fn threshold(mut self, value: usize) -> Self {
        self.threshold = Some(value);
        self
    }

    pub fn frontier(mut self, value: VecDeque<State<B>>) -> Self {
        self.frontier = Some(value);
        self
    }

    pub fn game(mut self, value: G) -> Self {
        self.game = Some(value);
        self
    }

    pub fn build(self) -> Result<ForwardTask<G, N, B>> {
        let game = self
            .game
            .context("game is required")?;

        let frontier = self
            .frontier
            .context("frontier is required")?;

        let threshold = self
            .threshold
            .context("threshold is required")?;

        let component = frontier
            .front()
            .map(|state| game.component(state))
            .context("frontier cannot be empty")?;

        let visited = game
            .sled_transaction()
            .context("Failed to open Sled tree for visited tracking")?;

        let mut progress = 0;
        for state in &frontier {
            if visited
                .insert(state, G::Record::default())
                .context(
                    "Failed to insert frontier state into Sled visited tree",
                )?
                .is_none()
            {
                progress += 1;
            }
        }

        Ok(ForwardTask {
            _phantom: PhantomData,
            buffered: 0,
            explored: 0,
            pending: HashMap::new(),
            threshold,
            component,
            frontier,
            progress,
            visited,
            game,
        })
    }
}

impl<G, const N: PlayerCount, const B: usize> ForwardTask<G, N, B>
where
    G: SledManager<N, B>
        + SQLiteManager<N, B>
        + Implicit<B>
        + Sequential<N, B>
        + IntegerUtility<N, B>
        + Partition<B>
        + Variable
        + Clone
        + Send
        + 'static,
{
    pub fn component(&self) -> Component {
        self.component
    }

    fn child(&self, frontier: VecDeque<State<B>>) -> Result<Task> {
        let child = ForwardTaskBuilder::<G, N, B>::default()
            .game(self.game.clone())
            .frontier(frontier)
            .threshold(self.threshold)
            .build()?;

        let comp = child.component();
        let task = TaskBuilder::default()
            .executable(child)
            .about(format!("Explore component {}", comp))
            .requires(Dependencies::new())
            .retriable(false)
            .size(None)
            .build()
            .context("Failed to build child explore task")?;

        Ok(task)
    }

    fn spawn(&mut self) -> Vec<Task> {
        self.buffered = 0;
        let pending: Vec<_> = self.pending.drain().collect();
        pending
            .into_iter()
            .map(|(_comp, frontier)| self.child(frontier))
            .collect::<Result<Vec<_>>>()
            .expect("Failed to spawn pending children")
    }

    fn process(&mut self, state: State<B>) {
        if self
            .visited
            .insert(state, G::Record::default())
            .context("Failed to insert state into Sled visited tree during DFS")
            .expect("Sled insert failed during forward exploration")
            .is_some()
        {
            return;
        }

        self.progress += 1;
        let comp = self.game.component(&state);
        if comp == self.component {
            self.frontier.push_back(state);
        } else {
            self.pending
                .entry(comp)
                .or_default()
                .push_back(state);
            self.buffered += 1;
        }
    }

    fn merge_visited(
        &mut self,
        visited: &sled::Tree,
        progress: usize,
    ) -> Result<()> {
        for entry in visited.iter() {
            let (key, _) = entry.context(
                "Failed to iterate over Sled visited tree during merge",
            )?;
            self.visited
                .insert(key, G::Record::default())
                .context(
                    "Failed to insert merged state into Sled visited tree",
                )?;
        }

        self.progress = self.progress.max(progress);
        Ok(())
    }

    fn merge_pending(
        &mut self,
        pending: &HashMap<Component, VecDeque<State<B>>>,
        buffered: usize,
    ) {
        for (comp, frontier) in pending {
            self.pending
                .entry(*comp)
                .or_default()
                .extend(frontier);
        }

        self.buffered += buffered;
    }

    fn merge_frontier(&mut self, frontier: &VecDeque<State<B>>) {
        let new: Vec<_> = frontier
            .iter()
            .filter(|s| {
                !self
                    .visited
                    .contains_key(*s)
                    .unwrap_or(false)
            })
            .filter(|s| !self.frontier.contains(*s))
            .copied()
            .collect();

        self.frontier.extend(new);
    }
}

impl<G, const N: PlayerCount, const B: usize> Executable
    for ForwardTask<G, N, B>
where
    G: SledManager<N, B>
        + SQLiteManager<N, B>
        + Implicit<B>
        + Sequential<N, B>
        + IntegerUtility<N, B>
        + Partition<B>
        + Variable
        + Clone
        + Send
        + 'static,
{
    fn tick(&mut self, _deps: TaskOutcomes) -> Option<YieldUpdate> {
        let Some(state) = self.frontier.pop_back() else {
            let remaining = if !self.pending.is_empty() {
                self.spawn()
            } else {
                Vec::new()
            };

            let outcome = TaskOutcome::Success(0);
            return Some(
                YieldUpdateBuilder::default()
                    .intention(YieldIntention::Suspended(outcome))
                    .discovered(remaining)
                    .build()
                    .expect("Failed to build suspended yield"),
            );
        };

        let successors = self.game.outgoing(&state);
        for next in successors {
            self.process(next);
        }

        self.explored += 1;
        let new = if self.buffered >= self.threshold {
            self.spawn()
        } else {
            Vec::new()
        };

        let update = YieldUpdateBuilder::default()
            .intention(YieldIntention::Ready)
            .discovered(new)
            .build()
            .expect("Failed to build ready yield");

        Some(update)
    }

    fn size(&self) -> Option<u64> {
        None
    }

    fn progress(&self) -> Option<u64> {
        Some(self.progress as u64)
    }

    fn merge(&mut self, other: Box<dyn Executable>) -> Result<()> {
        let other = other
            .as_any()
            .downcast_ref::<ForwardTask<G, N, B>>()
            .context("Cannot merge non-ExploreTask")?;

        if self.component != other.component {
            bail!(
                "Cannot merge tasks for different components: {} != {}",
                self.component,
                other.component
            );
        }

        self.merge_visited(&other.visited, other.progress)?;
        self.merge_pending(&other.pending, other.buffered);
        self.merge_frontier(&other.frontier);
        self.explored = self.explored.max(other.explored);

        Ok(())
    }

    fn id(&self) -> TaskID {
        TaskIDBuilder::default()
            .category(TaskCategory::Explore)
            .component(self.component)
            .build()
            .expect("TaskID builder should not fail with all fields provided")
    }
}

/* UTILITIES */

trait ExecutableExt {
    fn as_any(&self) -> &dyn Any;
}

impl ExecutableExt for dyn Executable {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}

/* TESTS */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::developer::GraphBuilder;
    use crate::game::mock::Node;
    use crate::game::mock::SessionBuilder;
    use crate::game::traits::Implicit;
    use crate::node;

    #[test]
    fn explore_tiny_game() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(0);
        let t1 = node![1; 1, -1];

        let graph = GraphBuilder::default()
            .edge(&s1, &s2)
            .edge(&s2, &s3)
            .edge(&s3, &t1);

        let game = SessionBuilder::default()
            .name("explore_tiny_game")
            .graph(graph)
            .source(&s1)
            .build()?;

        let start = game.source();

        let mut frontier = VecDeque::new();
        frontier.push_back(start);
        let mut task = ForwardTaskBuilder::<_, 8>::default()
            .game(game)
            .frontier(frontier)
            .threshold(100)
            .build()?;

        let empty = TaskOutcomes::new();

        let update1 = task.tick(empty.clone()).unwrap();
        assert!(matches!(update1.intention, YieldIntention::Ready));
        assert_eq!(task.explored, 1);

        let update2 = task.tick(empty.clone()).unwrap();
        assert!(matches!(update2.intention, YieldIntention::Ready));
        assert_eq!(task.explored, 2);

        let mut ticks = 2;
        loop {
            let update = task.tick(empty.clone()).unwrap();
            if matches!(update.intention, YieldIntention::Suspended(_)) {
                break;
            }
            ticks += 1;

            if ticks > 100 {
                bail!("Too many ticks - likely infinite loop");
            }
        }

        assert!(task.explored > 0);
        assert!(task.frontier.is_empty());

        Ok(())
    }

    #[test]
    fn no_revisit_states() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(0);
        let s4 = node!(1);
        let t1 = node![0; 1, -1];

        let graph = GraphBuilder::default()
            .edge(&s1, &s2)
            .edge(&s2, &s3)
            .edge(&s3, &s4)
            .edge(&s4, &t1)
            .edge(&s2, &s4);

        let game = SessionBuilder::default()
            .name("no_revisit_states")
            .graph(graph)
            .source(&s1)
            .build()?;

        let start = game.source();

        let mut frontier = VecDeque::new();
        frontier.push_back(start);
        let mut task = ForwardTaskBuilder::<_, 8>::default()
            .game(game)
            .frontier(frontier)
            .threshold(100)
            .build()?;

        let empty = TaskOutcomes::new();

        let mut prev = 1;

        for _ in 0..50 {
            let update = task.tick(empty.clone()).unwrap();

            let current = task.visited.len() as usize;
            assert!(current >= prev);
            prev = current;

            if matches!(update.intention, YieldIntention::Suspended(_)) {
                break;
            }
        }

        assert_eq!(task.explored, task.visited.len() as usize);
        Ok(())
    }

    #[test]
    fn merge_combines_state() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(0);
        let s4 = node!(1);
        let s5 = node!(0);
        let t1 = node![1; 1, -1];

        let graph = GraphBuilder::default()
            .edge(&s1, &s2)
            .edge(&s2, &s3)
            .edge(&s3, &s4)
            .edge(&s4, &s5)
            .edge(&s5, &t1);

        let game = SessionBuilder::default()
            .name("merge_combines_state")
            .graph(graph)
            .source(&s1)
            .build()?;

        let start = game.source();

        let mut frontier1 = VecDeque::new();
        frontier1.push_back(start);
        let mut task1 = ForwardTaskBuilder::<_, 8>::default()
            .game(game.clone())
            .frontier(frontier1)
            .threshold(100)
            .build()?;

        let mut frontier2 = VecDeque::new();
        frontier2.push_back(start);
        let task2 = ForwardTaskBuilder::<_, 8>::default()
            .game(game)
            .frontier(frontier2)
            .threshold(100)
            .build()?;

        let empty = TaskOutcomes::new();
        task1
            .tick(empty.clone())
            .context("First tick failed")?;

        task1
            .tick(empty)
            .context("Second tick failed")?;

        let before = task1.visited.len();
        let explored = task1.explored;
        task1.merge(Box::new(task2))?;

        assert!(task1.visited.len() >= before);
        assert_eq!(task1.explored, explored);

        Ok(())
    }

    #[test]
    fn reports_progress_and_size() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let t1 = node![1; 1, -1];

        let graph = GraphBuilder::default()
            .edge(&s1, &s2)
            .edge(&s2, &t1);

        let game = SessionBuilder::default()
            .name("reports_progress_and_size")
            .graph(graph)
            .source(&s1)
            .build()?;

        let start = game.source();

        let mut frontier = VecDeque::new();
        frontier.push_back(start);
        let task = ForwardTaskBuilder::<_, 8>::default()
            .game(game)
            .frontier(frontier)
            .threshold(100)
            .build()?;

        assert_eq!(task.progress(), Some(1));
        assert_eq!(task.size(), None);

        Ok(())
    }

    #[test]
    fn buffer_accumulates_before_spawning() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(0);
        let t1 = node![1; 1, -1];

        let graph = GraphBuilder::default()
            .edge(&s1, &s2)
            .edge(&s2, &s3)
            .edge(&s3, &t1);

        let game = SessionBuilder::default()
            .name("buffer_accumulates_before_spawning")
            .graph(graph)
            .source(&s1)
            .build()?;

        let start = game.source();

        let mut frontier = VecDeque::new();
        frontier.push_back(start);
        let mut task = ForwardTaskBuilder::<_, 8>::default()
            .game(game)
            .frontier(frontier)
            .threshold(1000)
            .build()?;

        let empty = TaskOutcomes::new();

        for _ in 0..10 {
            let update = task.tick(empty.clone()).unwrap();
            if matches!(update.intention, YieldIntention::Suspended(_)) {
                break;
            }
            assert_eq!(update.discovered.len(), 0);
        }

        assert_eq!(task.buffered, 0);

        Ok(())
    }

    #[test]
    fn immediate_spawn_with_threshold_one() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let t1 = node![1; 1, -1];

        let graph = GraphBuilder::default()
            .edge(&s1, &s2)
            .edge(&s2, &t1);

        let game = SessionBuilder::default()
            .name("immediate_spawn_with_threshold_one")
            .graph(graph)
            .source(&s1)
            .build()?;

        let start = game.source();

        let mut frontier = VecDeque::new();
        frontier.push_back(start);
        let mut task = ForwardTaskBuilder::<_, 8>::default()
            .game(game)
            .frontier(frontier)
            .threshold(1)
            .build()?;

        let empty = TaskOutcomes::new();

        for _ in 0..10 {
            let update = task.tick(empty.clone()).unwrap();
            if matches!(update.intention, YieldIntention::Suspended(_)) {
                break;
            }
        }

        assert!(task.pending.is_empty());
        assert_eq!(task.buffered, 0);

        Ok(())
    }

    #[test]
    fn multiple_frontier_initialization() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(0);
        let t1 = node![1; 1, -1];

        let graph = GraphBuilder::default()
            .edge(&s1, &s2)
            .edge(&s2, &s3)
            .edge(&s3, &t1);

        let game = SessionBuilder::default()
            .name("multiple_frontier_initialization")
            .graph(graph)
            .source(&s1)
            .build()?;

        let start1 = game.source();

        let mut frontier = VecDeque::new();
        frontier.push_back(start1);
        frontier.push_back(start1);
        frontier.push_back(start1);

        let task = ForwardTaskBuilder::<_, 8>::default()
            .game(game)
            .frontier(frontier.clone())
            .threshold(100)
            .build()?;

        assert_eq!(task.frontier.len(), frontier.len());
        assert_eq!(task.visited.len() as usize, 1);

        Ok(())
    }
}
