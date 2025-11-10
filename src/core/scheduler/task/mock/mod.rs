//! # Mock Task Implementation
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use derive_builder::Builder;
use petgraph::Graph;
use petgraph::dot::Config;
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use crate::core::developer::visualize_graph;
use crate::core::scheduler::Dependencies;
use crate::core::scheduler::Task as SchedulerTask;
use crate::core::scheduler::TaskBuilder as SchedulerTaskBuilder;
use crate::core::scheduler::TaskID;
use crate::core::scheduler::TaskOutcome;
use crate::core::scheduler::TaskOutcomes;
use crate::core::scheduler::YieldIntention;
use crate::core::scheduler::YieldUpdate;
use crate::core::scheduler::YieldUpdateBuilder;
use crate::traits::scheduler::Executable;

/* RE-EXPORTS */

pub use builder::TaskBuilder;

/* SUBMODULES */

mod builder;

/* STRUCTURES */

/// Configuration for a single task node in the declaration graph.
/// This is the unit users declare and reference when building graphs.
#[derive(Clone, Debug, Builder)]
#[builder(pattern = "owned")]
pub struct TaskNode {
    pub outcome: TaskOutcome,
    pub release: usize,
    pub ticks: usize,

    #[builder(default)]
    pub retriable: bool,

    #[builder(default, setter(into))]
    pub about: String,

    #[builder(default)]
    pub size: Option<u64>,
}

/// Compiled task data with relationships extracted from the graph.
/// Each graph node position gets its own TaskData instance.
struct TaskData {
    dependencies: Dependencies,
    discovers: Vec<TaskID>,
    config: TaskNode,
}

/// Shared compiled task data wrapped in Arc for Send compatibility.
#[derive(Clone)]
pub struct GlobalData {
    data: Arc<HashMap<TaskID, TaskData>>,
}

/// The executable mock task that performs work and discovers children.
pub struct Task {
    progress: usize,
    released: usize,
    global: GlobalData,
    tid: TaskID,
    verified: bool,
}

/// A validated and compiled task graph with visualization support.
pub struct TaskGraph<'a> {
    pub compiled: GlobalData,
    pub inserted: HashMap<*const TaskNode, NodeIndex>,
    pub graph: Graph<&'a TaskNode, ()>,
    pub root: TaskID,
    pub name: &'static str,
}

/* IMPLEMENTATIONS */

impl YieldUpdate {
    /// Creates a Waiting yield update with no discoveries.
    fn new_waiting(deps: Dependencies) -> Self {
        YieldUpdateBuilder::default()
            .intention(YieldIntention::Waiting(deps))
            .discovered(Vec::new())
            .build()
            .expect("Failed to build waiting update")
    }

    /// Creates a Ready yield update with no discoveries.
    fn new_ready() -> Self {
        YieldUpdateBuilder::default()
            .intention(YieldIntention::Ready)
            .discovered(Vec::new())
            .build()
            .expect("Failed to build ready update")
    }

    /// Creates Suspended yield update with given outcome and discoveries.
    fn new_suspended(
        outcome: TaskOutcome,
        discovered: Vec<SchedulerTask>,
    ) -> Self {
        YieldUpdateBuilder::default()
            .intention(YieldIntention::Suspended(outcome))
            .discovered(discovered)
            .build()
            .expect("Failed to build suspended update")
    }

    /// Creates a Ready yield update with the given discoveries.
    fn with_ready(discovered: Vec<SchedulerTask>) -> Self {
        YieldUpdateBuilder::default()
            .intention(YieldIntention::Ready)
            .discovered(discovered)
            .build()
            .expect("Failed to build ready update")
    }

    /// Creates Waiting yield update with given dependencies and discoveries.
    fn with_waiting(
        deps: Dependencies,
        discovered: Vec<SchedulerTask>,
    ) -> Self {
        YieldUpdateBuilder::default()
            .intention(YieldIntention::Waiting(deps))
            .discovered(discovered)
            .build()
            .expect("Failed to build waiting update")
    }
}

impl GlobalData {
    /// Wraps task data in Arc for shared access across tasks.
    fn new(data: HashMap<TaskID, TaskData>) -> Self {
        Self {
            data: Arc::new(data),
        }
    }

    /// Retrieves task data by ID from the shared graph data.
    fn get(&self, tid: TaskID) -> Option<&TaskData> {
        self.data.get(&tid)
    }
}

impl Task {
    /// Creates a new task executable with given ID and graph data.
    pub fn new(tid: TaskID, graph: GlobalData) -> Self {
        Self {
            tid,
            global: graph,
            progress: 0,
            released: 0,
            verified: false,
        }
    }

    /// Returns this task's compiled configuration and relationships.
    fn data(&self) -> &TaskData {
        self.global
            .get(self.tid)
            .expect("Task not in graph")
    }

    /// Returns true if task has completed all its ticks.
    fn at_completion(&self) -> bool {
        self.progress >= self.data().config.ticks
    }

    /// Returns true if task has reached the release tick.
    fn at_release(&self) -> bool {
        self.progress >= self.data().config.release
    }

    /// Returns the number of children not yet released.
    fn count_remaining(&self) -> usize {
        self.data().discovers.len() - self.released
    }

    /* EXECUTION */

    /// Checks if dependencies are satisfied on first execution.
    fn verify(&mut self, deps: &TaskOutcomes) -> Option<YieldUpdate> {
        if self.verified {
            return None;
        }

        let unsatisfied = self.unsatisfied_deps(deps)?;
        Some(YieldUpdate::new_waiting(unsatisfied))
    }

    /// Returns any dependencies missing from the provided outcomes.
    fn unsatisfied_deps(
        &self,
        available: &TaskOutcomes,
    ) -> Option<Dependencies> {
        let required = &self.data().dependencies;
        let missing: Dependencies = required
            .iter()
            .filter(|tid| !available.contains_key(tid))
            .copied()
            .collect();

        if missing.is_empty() { None } else { Some(missing) }
    }

    /// Executes one tick of work and handles release logic.
    fn work(&mut self) -> YieldUpdate {
        self.verified = true;
        self.progress += 1;

        let completed = self.at_completion();
        let releasing = self.at_release();
        let remaining = self.count_remaining();

        if completed && remaining > 0 {
            return self.finish_remaining();
        }

        if releasing && remaining > 0 {
            return self.release_one();
        }

        if completed {
            return self.suspend();
        }

        YieldUpdate::new_ready()
    }

    /// Releases the next child and waits if this is the last.
    fn release_one(&mut self) -> YieldUpdate {
        let child = self.discover_next();
        self.released += 1;

        if self.released == self.data().discovers.len() {
            let deps = self.all_children();
            YieldUpdate::with_waiting(deps, child)
        } else {
            YieldUpdate::with_ready(child)
        }
    }

    /// Releases all remaining children and waits for them.
    fn finish_remaining(&self) -> YieldUpdate {
        let children = self.discover_remaining();
        let deps = self.all_children();
        YieldUpdate::with_waiting(deps, children)
    }

    /// Suspends the task with its configured outcome.
    fn suspend(&self) -> YieldUpdate {
        let outcome = self.data().config.outcome.clone();
        YieldUpdate::new_suspended(outcome, vec![])
    }

    /* DISCOVERY */

    /// Returns an iterator over all child task IDs.
    fn child_tids(&self) -> impl Iterator<Item = TaskID> + '_ {
        self.data()
            .discovers
            .iter()
            .copied()
    }

    /// Builds the next child task to be discovered.
    fn discover_next(&self) -> Vec<SchedulerTask> {
        let tids: Vec<TaskID> = self.child_tids().collect();
        let build = |&tid| self.build(tid).ok();
        tids.get(self.released)
            .into_iter()
            .filter_map(build)
            .collect()
    }

    /// Builds all children that have not yet been released.
    fn discover_remaining(&self) -> Vec<SchedulerTask> {
        let build = |tid| self.build(tid).ok();
        self.child_tids()
            .skip(self.released)
            .filter_map(build)
            .collect()
    }

    /// Returns all child task IDs as a dependency set.
    fn all_children(&self) -> Dependencies {
        self.child_tids().collect()
    }

    /// Constructs a scheduler task for the given child ID.
    fn build(&self, tid: TaskID) -> Result<SchedulerTask> {
        let data = self
            .global
            .get(tid)
            .context("Discovered task not in graph")?;

        build_scheduler_task(tid, data, self.global.clone())
    }
}

impl Executable for Task {
    fn tick(&mut self, deps: TaskOutcomes) -> YieldUpdate {
        if let Some(waiting) = self.verify(&deps) {
            return waiting;
        }

        self.work()
    }

    fn size(&self) -> Option<u64> {
        self.data().config.size
    }

    fn merge(&mut self, other: Box<dyn Executable>) -> Result<()> {
        let other = other
            .as_any()
            .downcast_ref::<Task>()
            .context("Cannot merge with non-mock Task")?;

        if self.tid != other.tid {
            bail!("Cannot merge tasks with different IDs");
        }

        let compatible = outcomes_compatible(
            &self.data().config.outcome,
            &other.data().config.outcome,
        );

        if !compatible {
            bail!("Cannot merge tasks with incompatible outcomes");
        }

        self.progress = self.progress.max(other.progress);
        self.released = self.released.max(other.released);
        self.verified = self.verified || other.verified;

        Ok(())
    }
}

impl<'a> TaskGraph<'a> {
    /// Return the name of this task graph.
    pub fn name(&self) -> &str {
        self.name
    }

    /// Return a reference to the declaration graph.
    pub fn graph(&self) -> &Graph<&TaskNode, ()> {
        &self.graph
    }

    /// Create the root task to register with the scheduler.
    pub fn root_task(&self) -> Result<SchedulerTask> {
        let data = self
            .compiled
            .get(self.root)
            .context("Root task not in compiled graph")?;

        build_scheduler_task(self.root, data, self.compiled.clone())
    }

    /// Creates an SVG visualization of the task graph in the visuals directory
    /// under the development data directory at the project root.
    pub fn visualize(&self, module: &str) -> Result<()> {
        let graph = format!("{}", self);
        visualize_graph(&graph, self.name(), module)
    }
}

/* TRAIT IMPLEMENTATIONS */

impl dyn Executable {
    /// Downcasts executable trait object to concrete type for merging.
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}

/* IMPL EXTERNAL TRAIT */

impl Display for TaskGraph<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format_node = |_, n: (NodeIndex, &&TaskNode)| {
            let (index, node) = n;
            let tid = index.index() as TaskID;
            let mut attrs = String::new();

            let outcome = format_outcome(&node.outcome);
            let label = if node.release < node.ticks {
                format!(
                    "T{} ({}t r{})\\n{}",
                    tid, node.ticks, node.release, outcome
                )
            } else {
                format!("T{} ({}t)\\n{}", tid, node.ticks, outcome)
            };
            attrs += &format!("label=\"{}\" ", label);
            attrs += "style=filled ";

            if tid == self.root {
                attrs += "shape=doublecircle ";
                attrs += "fillcolor=navajowhite3 ";
            } else {
                attrs += "shape=circle ";
                attrs += "fillcolor=lightsteelblue ";
            }

            attrs
        };

        write!(
            f,
            "{:?}",
            Dot::with_attr_getters(
                &self.graph(),
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, _| String::new(),
                &format_node,
            )
        )
    }
}

/* HELPER FUNCTIONS */

/// Constructs a scheduler task with executable from compiled graph data.
fn build_scheduler_task(
    tid: TaskID,
    data: &TaskData,
    graph: GlobalData,
) -> Result<SchedulerTask> {
    let executable: Box<dyn Executable> = Box::new(Task::new(tid, graph));

    SchedulerTaskBuilder::default()
        .tid(tid)
        .executable(executable)
        .retriable(data.config.retriable)
        .about(data.config.about.clone())
        .size(data.config.size)
        .requires(data.dependencies.clone())
        .build()
        .context("Failed to build scheduler task")
}

/// Checks if two task outcomes are compatible for merging.
fn outcomes_compatible(a: &TaskOutcome, b: &TaskOutcome) -> bool {
    match (a, b) {
        (TaskOutcome::Success(x), TaskOutcome::Success(y)) => x == y,
        (TaskOutcome::Failure(x), TaskOutcome::Failure(y)) => x == y,
        (TaskOutcome::Error, TaskOutcome::Error) => true,
        _ => false,
    }
}

/// Formats a task outcome for display in visualizations.
fn format_outcome(outcome: &TaskOutcome) -> String {
    match outcome {
        TaskOutcome::Success(code) => format!("Success({})", code),
        TaskOutcome::Failure(code) => format!("Failure({})", code),
        TaskOutcome::Error => "Error".to_string(),
    }
}

/* TESTS */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::developer::GraphBuilder;

    const MODULE: &str = "mock-task-tests";

    /// Tests building a graph with a single task node.
    #[test]
    fn build_single_task() -> Result<()> {
        let t1 = TaskNodeBuilder::default()
            .ticks(5)
            .release(5)
            .outcome(TaskOutcome::Success(0))
            .about("root task")
            .build()?;

        let graph = GraphBuilder::new();
        let tg = TaskBuilder::new()
            .name("single")
            .graph(graph)
            .source(&t1)
            .build()?;

        tg.visualize(MODULE)?;
        assert_eq!(tg.name(), "single");
        assert_eq!(tg.root, 0);

        Ok(())
    }

    /// Tests building a linear chain of dependent tasks.
    #[test]
    fn build_linear_chain() -> Result<()> {
        let t1 = TaskNodeBuilder::default()
            .ticks(2)
            .release(2)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let t2 = TaskNodeBuilder::default()
            .ticks(3)
            .release(3)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let t3 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&t1, &t2)
            .edge(&t2, &t3);

        let tg = TaskBuilder::new()
            .name("linear")
            .graph(graph)
            .source(&t1)
            .build()?;

        tg.visualize(MODULE)?;
        assert_eq!(tg.graph.node_count(), 3);

        Ok(())
    }

    /// Tests building a tree with multiple levels of children.
    #[test]
    fn build_tree_structure() -> Result<()> {
        let root = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let left = TaskNodeBuilder::default()
            .ticks(2)
            .release(2)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let right = TaskNodeBuilder::default()
            .ticks(2)
            .release(2)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let leaf1 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(3))
            .build()?;

        let leaf2 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(4))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&root, &left)
            .edge(&root, &right)
            .edge(&left, &leaf1)
            .edge(&right, &leaf2);

        let tg = TaskBuilder::new()
            .name("tree")
            .graph(graph)
            .source(&root)
            .build()?;

        tg.visualize(MODULE)?;
        assert_eq!(tg.graph.node_count(), 5);

        Ok(())
    }

    /// Tests building a diamond pattern with converging dependencies.
    #[test]
    fn build_diamond_dependencies() -> Result<()> {
        let start = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let left = TaskNodeBuilder::default()
            .ticks(5)
            .release(5)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let right = TaskNodeBuilder::default()
            .ticks(10)
            .release(10)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let end = TaskNodeBuilder::default()
            .ticks(15)
            .release(15)
            .outcome(TaskOutcome::Success(3))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&start, &left)
            .edge(&start, &right)
            .edge(&left, &end)
            .edge(&right, &end);

        let tg = TaskBuilder::new()
            .name("diamond")
            .graph(graph)
            .source(&start)
            .build()?;

        tg.visualize(MODULE)?;
        let end_data = tg
            .compiled
            .get(3)
            .context("End task not found")?;

        assert_eq!(end_data.dependencies.len(), 2);
        Ok(())
    }

    /// Tests that the same node can be referenced multiple times.
    #[test]
    fn reuse_same_node() -> Result<()> {
        let shared = TaskNodeBuilder::default()
            .ticks(2)
            .release(2)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let t1 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let t2 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&t1, &shared)
            .edge(&t2, &shared);

        let tg = TaskBuilder::new()
            .name("reused")
            .graph(graph)
            .source(&t1)
            .build()?;

        tg.visualize(MODULE)?;
        assert_eq!(tg.graph.node_count(), 3);

        Ok(())
    }

    /// Tests that cyclical dependencies are detected and rejected.
    #[test]
    fn reject_cycle() -> Result<()> {
        let t1 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let t2 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let t3 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&t1, &t2)
            .edge(&t2, &t3)
            .edge(&t3, &t1);

        let result = TaskBuilder::new()
            .name("cycle")
            .graph(graph)
            .source(&t1)
            .build();

        assert!(result.is_err());
        Ok(())
    }

    /// Tests that tasks with zero ticks are rejected.
    #[test]
    fn reject_zero_ticks() -> Result<()> {
        let bad = TaskNodeBuilder::default()
            .ticks(0)
            .release(0)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let graph = GraphBuilder::new();
        let result = TaskBuilder::new()
            .name("zero-ticks")
            .graph(graph)
            .source(&bad)
            .build();

        assert!(result.is_err());
        Ok(())
    }

    /// Tests that a source node disconnected from the graph is handled.
    #[test]
    fn handle_disconnected_source() -> Result<()> {
        let t1 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let t2 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let graph = GraphBuilder::new().edge(&t1, &t2);
        let isolated = TaskNodeBuilder::default()
            .ticks(5)
            .release(5)
            .outcome(TaskOutcome::Success(99))
            .build()?;

        let tg = TaskBuilder::new()
            .name("disconnected")
            .graph(graph)
            .source(&isolated)
            .build()?;

        tg.visualize(MODULE)?;
        assert_eq!(tg.graph.node_count(), 3);
        Ok(())
    }

    /// Tests creating a scheduler task from the root node.
    #[test]
    fn create_root_task() -> Result<()> {
        let t1 = TaskNodeBuilder::default()
            .ticks(3)
            .release(3)
            .outcome(TaskOutcome::Success(42))
            .about("test task")
            .size(Some(100))
            .retriable(true)
            .build()?;

        let graph = GraphBuilder::new();
        let tg = TaskBuilder::new()
            .name("root-test")
            .graph(graph)
            .source(&t1)
            .build()?;

        tg.visualize(MODULE)?;
        let task = tg.root_task()?;
        assert_eq!(task.tid, 0);
        assert_eq!(task.about, "test task");
        assert_eq!(task.size, Some(100));
        assert!(task.retriable);

        Ok(())
    }

    /// Tests that tasks wait for dependencies before executing.
    #[test]
    fn test_waits_for_dependencies() -> Result<()> {
        let dep1 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .about("dependency 1")
            .build()?;

        let dep2 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .about("dependency 2")
            .build()?;

        let consumer = TaskNodeBuilder::default()
            .ticks(2)
            .release(2)
            .outcome(TaskOutcome::Success(0))
            .about("consumer")
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&dep1, &consumer)
            .edge(&dep2, &consumer);

        let tg = TaskBuilder::new()
            .name("waits-for-deps")
            .graph(graph)
            .source(&consumer)
            .build()?;

        tg.visualize(MODULE)?;

        let task = tg.root_task()?;
        let mut executable = task.executable;

        let empty_deps = TaskOutcomes::new();
        let update1 = executable.tick(empty_deps);

        let dep_tids: Vec<TaskID> = match &update1.intention {
            YieldIntention::Waiting(unsatisfied) => {
                assert_eq!(unsatisfied.len(), 2);
                unsatisfied
                    .iter()
                    .copied()
                    .collect()
            },
            _ => panic!("Expected Waiting intention on first tick"),
        };

        let partial_deps: TaskOutcomes =
            [(dep_tids[0], TaskOutcome::Success(1))]
                .into_iter()
                .collect();
        let update2 = executable.tick(partial_deps);

        match update2.intention {
            YieldIntention::Waiting(ref unsatisfied) => {
                assert_eq!(unsatisfied.len(), 1);
                assert!(unsatisfied.contains(&dep_tids[1]));
            },
            _ => panic!("Expected Waiting with one dependency"),
        }

        let full_deps: TaskOutcomes = [
            (dep_tids[0], TaskOutcome::Success(1)),
            (dep_tids[1], TaskOutcome::Success(2)),
        ]
        .into_iter()
        .collect();

        let update3 = executable.tick(full_deps.clone());
        match update3.intention {
            YieldIntention::Ready => {},
            _ => panic!("Expected Ready after dependencies satisfied"),
        }

        let update4 = executable.tick(full_deps);
        match update4.intention {
            YieldIntention::Suspended(TaskOutcome::Success(0)) => {},
            _ => panic!("Expected Suspended after completion"),
        }

        Ok(())
    }

    /// Tests releasing children one per tick with final Waiting.
    #[test]
    fn test_staged_release() -> Result<()> {
        let child1 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let child2 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let child3 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(3))
            .build()?;

        let parent = TaskNodeBuilder::default()
            .ticks(5)
            .release(2)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&parent, &child1)
            .edge(&parent, &child2)
            .edge(&parent, &child3);

        let tg = TaskBuilder::new()
            .name("staged-release")
            .graph(graph)
            .source(&parent)
            .build()?;

        tg.visualize(MODULE)?;

        let task = tg.root_task()?;
        let mut executable = task.executable;
        let empty = TaskOutcomes::new();

        let update1 = executable.tick(empty.clone());
        assert!(matches!(update1.intention, YieldIntention::Ready));
        assert_eq!(update1.discovered.len(), 0);

        let update2 = executable.tick(empty.clone());
        assert!(matches!(update2.intention, YieldIntention::Ready));
        assert_eq!(update2.discovered.len(), 1);

        let update3 = executable.tick(empty.clone());
        assert!(matches!(update3.intention, YieldIntention::Ready));
        assert_eq!(update3.discovered.len(), 1);

        let update4 = executable.tick(empty.clone());
        assert!(matches!(
            update4.intention,
            YieldIntention::Waiting(_)
        ));

        assert_eq!(update4.discovered.len(), 1);
        Ok(())
    }

    /// Tests releasing all remaining children when running out of ticks.
    #[test]
    fn test_release_timeout() -> Result<()> {
        let child1 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let child2 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let child3 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(3))
            .build()?;

        let parent = TaskNodeBuilder::default()
            .ticks(3)
            .release(2)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&parent, &child1)
            .edge(&parent, &child2)
            .edge(&parent, &child3);

        let tg = TaskBuilder::new()
            .name("release-timeout")
            .graph(graph)
            .source(&parent)
            .build()?;

        tg.visualize(MODULE)?;

        let task = tg.root_task()?;
        let mut executable = task.executable;
        let empty = TaskOutcomes::new();

        let update1 = executable.tick(empty.clone());
        assert!(matches!(update1.intention, YieldIntention::Ready));
        assert_eq!(update1.discovered.len(), 0);

        let update2 = executable.tick(empty.clone());
        assert!(matches!(update2.intention, YieldIntention::Ready));
        assert_eq!(update2.discovered.len(), 1);

        let update3 = executable.tick(empty);
        assert!(matches!(
            update3.intention,
            YieldIntention::Waiting(_)
        ));

        assert_eq!(update3.discovered.len(), 2);
        Ok(())
    }

    /// Tests immediate release starting at tick 0.
    #[test]
    fn test_immediate_release() -> Result<()> {
        let child1 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let child2 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let parent = TaskNodeBuilder::default()
            .ticks(3)
            .release(0)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&parent, &child1)
            .edge(&parent, &child2);

        let tg = TaskBuilder::new()
            .name("immediate-release")
            .graph(graph)
            .source(&parent)
            .build()?;

        tg.visualize(MODULE)?;

        let task = tg.root_task()?;
        let mut executable = task.executable;
        let empty = TaskOutcomes::new();

        let update1 = executable.tick(empty.clone());
        assert!(matches!(update1.intention, YieldIntention::Ready));
        assert_eq!(update1.discovered.len(), 1);

        let update2 = executable.tick(empty.clone());
        assert!(matches!(
            update2.intention,
            YieldIntention::Waiting(_)
        ));
        assert_eq!(update2.discovered.len(), 1);

        Ok(())
    }
}
