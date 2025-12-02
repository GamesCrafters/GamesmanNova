//! # Explore Scheduler Task
//!
//! DFS-based exploration task for discovering game state graphs. Spawns child
//! tasks when crossing component boundaries, enabling parallel exploration of
//! independent graph components.

use std::any::Any;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;

use crate::database::storage::Storage;
use crate::game::Component;
use crate::game::State;
use crate::game::traits::Implicit;
use crate::game::traits::Partition;
use crate::game::traits::Variable;
use crate::scheduler::TaskCategory;
use crate::scheduler::TaskID;
use crate::scheduler::TaskIDBuilder;
use crate::scheduler::TaskOutcome;
use crate::scheduler::TaskOutcomes;
use crate::scheduler::orchestration::Task;
use crate::scheduler::traits::Executable;
use crate::scheduler::traits::YieldIntention;
use crate::scheduler::traits::YieldUpdate;

/* STRUCTURES */

pub struct ForwardTask<G, R> {
    threshold: usize,
    component: Component,
    frontier: VecDeque<State>,
    buffered: usize,
    explored: usize,
    progress: usize,
    pending: HashMap<Component, VecDeque<State>>,
    storage: Arc<dyn Storage<R>>,
    ruleset: G,
}

pub struct ForwardTaskBuilder<G, R> {
    threshold: Option<usize>,
    frontier: Option<VecDeque<State>>,
    storage: Option<Arc<dyn Storage<R>>>,
    ruleset: Option<G>,
}

/* IMPLEMENTATIONS */

impl<G, R> Default for ForwardTaskBuilder<G, R> {
    fn default() -> Self {
        Self {
            threshold: None,
            frontier: None,
            storage: None,
            ruleset: None,
        }
    }
}

impl<G, R> ForwardTaskBuilder<G, R>
where
    G: Implicit + Variable + Partition + Clone + Send + 'static,
    R: Default + Into<Vec<u8>> + Clone + Send + Sync + 'static,
{
    pub fn threshold(mut self, value: usize) -> Self {
        self.threshold = Some(value);
        self
    }

    pub fn frontier(mut self, value: VecDeque<State>) -> Self {
        self.frontier = Some(value);
        self
    }

    pub fn ruleset(mut self, value: G) -> Self {
        self.ruleset = Some(value);
        self
    }

    pub fn storage(mut self, value: Arc<dyn Storage<R>>) -> Self {
        self.storage = Some(value);
        self
    }

    pub fn build(self) -> Result<ForwardTask<G, R>> {
        let ruleset = self
            .ruleset
            .context("game is required")?;

        let frontier = self
            .frontier
            .unwrap_or_else(|| VecDeque::from(vec![ruleset.source()]));

        let threshold = self
            .threshold
            .context("threshold is required")?;

        let component = frontier
            .front()
            .map(|state| ruleset.component(state))
            .context("frontier cannot be empty")?;

        let storage = self
            .storage
            .context("storage is required")?;

        let mut progress = 0;
        for state in &frontier {
            let record = R::default();
            if storage
                .get(state)
                .context("Storage get failure")?
                .is_none()
            {
                storage
                    .put(state, &record)
                    .context("Storage put failure")?;

                progress += 1;
            }
        }

        Ok(ForwardTask {
            buffered: 0,
            explored: 0,
            pending: HashMap::new(),
            threshold,
            component,
            frontier,
            progress,
            storage,
            ruleset,
        })
    }
}

impl<G, R> ForwardTask<G, R>
where
    G: Implicit + Variable + Partition + Clone + Send + 'static,
    R: Default + Into<Vec<u8>> + Clone + Send + Sync + 'static,
{
    pub fn component(&self) -> Component {
        self.component
    }

    fn child(&self, frontier: VecDeque<State>) -> Result<Task> {
        let child = ForwardTaskBuilder::<G, R>::default()
            .storage(Arc::clone(&self.storage))
            .ruleset(self.ruleset.clone())
            .threshold(self.threshold)
            .frontier(frontier)
            .build()?;

        let about = format!("Forward pass of variant {}", self.ruleset.name());
        let task = Task {
            executable: Box::new(child),
            dependencies: HashSet::new(),
            retriable: true,
            about,
            size: None,
        };

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

    fn process(&mut self, state: State) {
        let record = R::default();
        if self
            .storage
            .get(&state)
            .context("Failed to check storage during DFS")
            .expect("Storage get failed during forward exploration")
            .is_some()
        {
            return;
        }

        self.storage
            .put(&state, &record)
            .context("Failed to insert state into storage during DFS")
            .expect("Storage put failed during forward exploration");

        self.progress += 1;
        let comp = self.ruleset.component(&state);
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

    fn merge_pending(
        &mut self,
        pending: &HashMap<Component, VecDeque<State>>,
        buffered: usize,
    ) {
        for (comp, frontier) in pending {
            self.pending
                .entry(*comp)
                .or_default()
                .extend(frontier.iter().cloned());
        }

        self.buffered += buffered;
    }

    fn merge_frontier(&mut self, frontier: &VecDeque<State>) {
        for state in frontier {
            let is_visited = self
                .storage
                .get(state)
                .ok()
                .flatten()
                .is_some();

            if !is_visited && !self.frontier.contains(state) {
                self.frontier
                    .push_back(state.clone());
            }
        }
    }
}

impl<G, R> Executable for ForwardTask<G, R>
where
    G: Implicit + Variable + Partition + Clone + Send + 'static,
    R: Default + Into<Vec<u8>> + Clone + Send + Sync + 'static,
{
    fn tick(&mut self, _deps: TaskOutcomes) -> Option<YieldUpdate> {
        let Some(state) = self.frontier.pop_back() else {
            let remaining = if !self.pending.is_empty() {
                self.spawn()
            } else {
                Vec::new()
            };

            let outcome = TaskOutcome::Success(0);
            return Some(YieldUpdate {
                intention: YieldIntention::Suspended(outcome),
                discovered: remaining,
            });
        };

        let successors = self.ruleset.outgoing(&state);
        for next in successors {
            self.process(next);
        }

        self.explored += 1;
        if self.buffered >= self.threshold {
            return Some(YieldUpdate {
                intention: YieldIntention::Ready,
                discovered: self.spawn(),
            });
        }

        None
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
            .downcast_ref::<ForwardTask<G, R>>()
            .context("Cannot merge non-ExploreTask")?;

        if self.component != other.component {
            bail!(
                "Cannot merge tasks for different components: {} != {}",
                self.component,
                other.component
            );
        }

        self.progress = self.progress.max(other.progress);
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
    use crate::database::storage::InMemoryStorage;
    use crate::developer::GraphBuilder;
    use crate::game::mock;
    use crate::game::mock::Node;
    use crate::game::mock::SessionBuilder;
    use std::sync::Arc;

    use crate::game::traits::Implicit;
    use crate::node;

    /// Test helper: Tick task until it yields control back to scheduler.
    ///
    /// The cooperative scheduling API returns None for internal ticks and Some
    /// when yielding. This helper abstracts that for tests that don't care about
    /// internal tick granularity.
    fn tick_until_yield<G, R>(
        task: &mut ForwardTask<G, R>,
        deps: TaskOutcomes,
    ) -> YieldUpdate
    where
        G: Implicit + Variable + Partition + Clone + Send + 'static,
        R: Default + Into<Vec<u8>> + Clone + Send + Sync + 'static,
    {
        std::iter::repeat_with(|| task.tick(deps.clone()))
            .find_map(|x| x)
            .expect("Task should eventually yield")
    }

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
        let storage = Arc::new(InMemoryStorage::<mock::Record>::new());

        let mut frontier = VecDeque::new();
        frontier.push_back(start);
        let mut task = ForwardTaskBuilder::<_, mock::Record>::default()
            .ruleset(game)
            .storage(storage)
            .frontier(frontier)
            .threshold(100)
            .build()?;

        let empty = TaskOutcomes::new();

        let update = tick_until_yield(&mut task, empty.clone());

        assert!(matches!(
            update.intention,
            YieldIntention::Suspended(TaskOutcome::Success(_))
        ));
        assert!(
            task.explored > 0,
            "Should have explored some states"
        );
        assert!(
            task.frontier.is_empty(),
            "Frontier should be exhausted"
        );
        assert!(
            task.pending.is_empty(),
            "No cross-component work in single-component game"
        );

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
            .edge(&s2, &s4); // Creates a diamond - s2 reaches s4 via two paths

        let game = SessionBuilder::default()
            .name("no_revisit_states")
            .graph(graph)
            .source(&s1)
            .build()?;

        let start = game.source();
        let storage = Arc::new(InMemoryStorage::<mock::Record>::new());

        let mut frontier = VecDeque::new();
        frontier.push_back(start);
        let mut task = ForwardTaskBuilder::<_, mock::Record>::default()
            .ruleset(game)
            .storage(storage)
            .frontier(frontier)
            .threshold(100)
            .build()?;

        let empty = TaskOutcomes::new();

        let update = tick_until_yield(&mut task, empty.clone());

        assert!(matches!(
            update.intention,
            YieldIntention::Suspended(TaskOutcome::Success(_))
        ));
        assert!(
            task.explored > 0,
            "Should have explored all reachable states"
        );
        assert!(task.frontier.is_empty());

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
        let storage1 = Arc::new(InMemoryStorage::<mock::Record>::new());
        let storage2 = Arc::new(InMemoryStorage::<mock::Record>::new());

        let mut frontier1 = VecDeque::new();
        frontier1.push_back(start.clone());
        let mut task1 = ForwardTaskBuilder::<_, mock::Record>::default()
            .ruleset(game.clone())
            .storage(storage1)
            .frontier(frontier1)
            .threshold(100)
            .build()?;

        let mut frontier2 = VecDeque::new();
        frontier2.push_back(start);
        let task2 = ForwardTaskBuilder::<_, mock::Record>::default()
            .ruleset(game)
            .storage(storage2)
            .frontier(frontier2)
            .threshold(100)
            .build()?;

        let empty = TaskOutcomes::new();

        let _ = tick_until_yield(&mut task1, empty.clone());

        let explored = task1.explored;

        task1.merge(Box::new(task2))?;

        assert_eq!(
            task1.explored, explored,
            "Merge should preserve max explored count"
        );

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
        let storage = Arc::new(InMemoryStorage::<mock::Record>::new());

        let mut frontier = VecDeque::new();
        frontier.push_back(start);
        let task = ForwardTaskBuilder::<_, mock::Record>::default()
            .ruleset(game)
            .storage(storage)
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
        let storage = Arc::new(InMemoryStorage::<mock::Record>::new());

        let mut frontier = VecDeque::new();
        frontier.push_back(start);
        let mut task = ForwardTaskBuilder::<_, mock::Record>::default()
            .ruleset(game)
            .storage(storage)
            .frontier(frontier)
            .threshold(1000)
            .build()?;

        let empty = TaskOutcomes::new();

        let update = tick_until_yield(&mut task, empty);

        assert!(matches!(
            update.intention,
            YieldIntention::Suspended(TaskOutcome::Success(_))
        ));
        assert_eq!(
            task.buffered, 0,
            "Single-component game has no cross-component buffering"
        );
        assert!(task.pending.is_empty());

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
        let storage = Arc::new(InMemoryStorage::<mock::Record>::new());

        let mut frontier = VecDeque::new();
        frontier.push_back(start);
        let mut task = ForwardTaskBuilder::<_, mock::Record>::default()
            .ruleset(game)
            .storage(storage)
            .frontier(frontier)
            .threshold(1)
            .build()?;

        let empty = TaskOutcomes::new();

        let update = tick_until_yield(&mut task, empty);

        assert!(matches!(
            update.intention,
            YieldIntention::Suspended(TaskOutcome::Success(_))
        ));
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
        let storage = Arc::new(InMemoryStorage::<mock::Record>::new());

        let mut frontier = VecDeque::new();
        frontier.push_back(start1.clone());
        frontier.push_back(start1.clone());
        frontier.push_back(start1);

        let task = ForwardTaskBuilder::<_, mock::Record>::default()
            .ruleset(game)
            .storage(storage)
            .frontier(frontier.clone())
            .threshold(100)
            .build()?;

        assert_eq!(task.frontier.len(), frontier.len());
        assert_eq!(task.explored, 0);
        assert_eq!(task.progress, 1);

        Ok(())
    }
}
