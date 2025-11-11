//! Mock task implementation for scheduler testing.
//!
//! Tasks discover children according to a predefined graph, with configurable
//! release timing and execution duration.

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use derive_builder::Builder;
use petgraph::Graph;
use petgraph::dot::Config;
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;

use std::any::Any;
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

// Backwards compatibility alias
pub use TaskConfigBuilder as TaskNodeBuilder;

/* SUBMODULES */

mod builder;

/* STRUCTURES */

/// Configuration for a single task in the declaration graph.
#[derive(Clone, Debug, Builder)]
#[builder(pattern = "owned")]
pub struct TaskConfig {
    /// The outcome this task returns when complete
    pub outcome: TaskOutcome,

    /// The tick at which this task starts releasing children
    pub release: usize,

    /// Total ticks this task runs for before completion
    pub ticks: usize,

    /// Whether this task can be retried after Error state
    #[builder(default)]
    pub retriable: bool,

    /// Human-readable description
    #[builder(default, setter(into))]
    pub about: String,

    /// Estimated size for scheduler policy
    #[builder(default)]
    pub size: Option<u64>,
}

/// Compiled graph containing all task configurations and relationships.
pub(crate) struct CompiledGraph {
    /// Graph of task configurations
    pub(crate) graph: Graph<TaskConfig, ()>,

    /// Name of this graph for visualization
    pub(crate) name: String,

    /// Root task ID
    pub(crate) root: TaskID,
}

/// Lightweight executable task handle.
pub struct Task {
    /// Unique task identifier
    tid: TaskID,

    /// Number of ticks executed
    progress: usize,

    /// Number of children discovered
    released: usize,

    /// Shared graph data
    graph: Arc<CompiledGraph>,
}

/* IMPLEMENTATIONS */

impl CompiledGraph {
    fn config(&self, tid: TaskID) -> &TaskConfig {
        let index = NodeIndex::new(tid as usize);
        &self.graph[index]
    }

    fn children(&self, tid: TaskID) -> Vec<TaskID> {
        let index = NodeIndex::new(tid as usize);
        self.graph
            .neighbors(index)
            .map(|n| n.index() as TaskID)
            .collect()
    }
}

impl Task {
    pub fn new(tid: TaskID, graph: Arc<CompiledGraph>) -> Self {
        Self {
            tid,
            progress: 0,
            released: 0,
            graph,
        }
    }

    fn config(&self) -> &TaskConfig {
        self.graph.config(self.tid)
    }

    fn children(&self) -> Vec<TaskID> {
        self.graph.children(self.tid)
    }

    fn remaining(&self) -> usize {
        let total = self.children().len();
        total - self.released
    }

    fn unsatisfied(&self, deps: &TaskOutcomes) -> Option<Dependencies> {
        let children = self.children();
        let missing: Dependencies = children
            .into_iter()
            .filter(|tid| !deps.contains_key(tid))
            .collect();

        if missing.is_empty() { None } else { Some(missing) }
    }

    fn discover_one(&mut self) -> Vec<SchedulerTask> {
        let children = self.children();

        if let Some(&tid) = children.get(self.released) {
            self.released += 1;
            vec![self.build_child(tid)]
        } else {
            vec![]
        }
    }

    fn discover_all(&self) -> Vec<SchedulerTask> {
        let children = self.children();
        children
            .into_iter()
            .skip(self.released)
            .map(|tid| self.build_child(tid))
            .collect()
    }

    fn build_child(&self, tid: TaskID) -> SchedulerTask {
        let child = Task::new(tid, Arc::clone(&self.graph));
        let config = self.graph.config(tid);

        let requires: Dependencies = Dependencies::new();

        SchedulerTaskBuilder::default()
            .tid(tid)
            .executable(Box::new(child) as Box<dyn Executable>)
            .retriable(config.retriable)
            .about(config.about.clone())
            .size(config.size)
            .requires(requires)
            .build()
            .expect("Failed to build scheduler task")
    }

    fn handle_release(
        &mut self,
        num: usize,
        children: Vec<TaskID>,
    ) -> YieldUpdate {
        self.progress += 1;

        let config = self.config();
        let last = self.progress >= config.ticks;

        if last {
            let all = self.discover_all();
            self.released = num;
            let deps = children.into_iter().collect();
            return YieldUpdate::with_waiting(deps, all);
        }

        let one = self.discover_one();

        if self.released == num {
            let deps = children.into_iter().collect();
            return YieldUpdate::with_waiting(deps, one);
        }

        YieldUpdate::with_ready(one)
    }

    pub fn visualize(&self, module: &str) -> Result<()> {
        let dot = format!("{}", self);
        visualize_graph(&dot, &self.graph.name, module)
    }

    pub fn name(&self) -> &str {
        &self.graph.name
    }

    pub fn root_task(&self) -> Result<SchedulerTask> {
        let root_tid = self.graph.root;
        let root = Task::new(root_tid, Arc::clone(&self.graph));
        let config = self.graph.config(root_tid);

        let requires: Dependencies = Dependencies::new();

        let task = SchedulerTaskBuilder::default()
            .tid(root_tid)
            .executable(Box::new(root) as Box<dyn Executable>)
            .retriable(config.retriable)
            .about(config.about.clone())
            .size(config.size)
            .requires(requires)
            .build()
            .context("Failed to build root task")?;

        Ok(task)
    }
}

impl Executable for Task {
    fn tick(&mut self, deps: TaskOutcomes) -> Option<YieldUpdate> {
        let children = self.children();
        let num = children.len();
        let remaining = self.remaining();

        // check if waiting for children to complete
        if num > 0
            && remaining == 0
            && let Some(waiting) = self.unsatisfied(&deps)
        {
            return Some(YieldUpdate::new_waiting(waiting));
        }

        // handle release (before completion check)
        let config = self.config();
        if remaining > 0 && self.progress >= config.release {
            return Some(self.handle_release(num, children));
        }

        // check completion
        if self.progress >= config.ticks {
            return Some(YieldUpdate::new_suspended(config.outcome, vec![]));
        }

        // normal tick
        self.progress += 1;
        Some(YieldUpdate::new_ready())
    }

    fn size(&self) -> Option<u64> {
        self.config().size
    }

    fn progress(&self) -> Option<u64> {
        Some(self.progress as u64)
    }

    fn merge(&mut self, other: Box<dyn Executable>) -> Result<()> {
        let other = other
            .as_any()
            .downcast_ref::<Task>()
            .context("Cannot merge non-Task")?;

        if self.tid != other.tid {
            bail!("Cannot merge different task IDs");
        }

        let compat = outcomes_compatible(
            &self.config().outcome,
            &other.config().outcome,
        );

        if !compat {
            bail!("Incompatible outcomes");
        }

        self.progress = self.progress.max(other.progress);
        self.released = self.released.max(other.released);

        Ok(())
    }
}

impl Clone for Task {
    fn clone(&self) -> Self {
        Self {
            tid: self.tid,
            progress: self.progress,
            released: self.released,
            graph: Arc::clone(&self.graph),
        }
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let format = |_, n: (NodeIndex, &TaskConfig)| {
            let (index, config) = n;
            let tid = index.index() as TaskID;

            let outcome = format_outcome(&config.outcome);

            let timing = if config.release < config.ticks {
                format!("{}t r{}", config.ticks, config.release)
            } else {
                format!("{}t", config.ticks)
            };

            let size_str = config
                .size
                .map(|s| format!(" s{}", s))
                .unwrap_or_default();

            let about_str = if config.about.is_empty() {
                String::new()
            } else {
                format!("\\n{}", config.about)
            };

            let label = format!(
                "T{} ({}{})\\n{}{}",
                tid, timing, size_str, outcome, about_str
            );

            let mut attrs = format!("label=\"{}\" style=filled ", label);

            if tid == self.graph.root {
                attrs += "shape=doublecircle fillcolor=navajowhite3";
            } else {
                attrs += "shape=circle fillcolor=lightsteelblue";
            }

            attrs
        };

        write!(
            f,
            "{:?}",
            Dot::with_attr_getters(
                &self.graph.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, _| String::new(),
                &format,
            )
        )
    }
}

/* TRAIT IMPLEMENTATIONS */

impl dyn Executable {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}

/* HELPER FUNCTIONS */

fn outcomes_compatible(a: &TaskOutcome, b: &TaskOutcome) -> bool {
    match (a, b) {
        (TaskOutcome::Success(x), TaskOutcome::Success(y)) => x == y,
        (TaskOutcome::Failure(x), TaskOutcome::Failure(y)) => x == y,
        (TaskOutcome::Error, TaskOutcome::Error) => true,
        _ => false,
    }
}

fn format_outcome(outcome: &TaskOutcome) -> String {
    match outcome {
        TaskOutcome::Success(c) => format!("Success({})", c),
        TaskOutcome::Failure(c) => format!("Failure({})", c),
        TaskOutcome::Error => "Error".to_string(),
    }
}

impl YieldUpdate {
    fn new_waiting(deps: Dependencies) -> Self {
        YieldUpdateBuilder::default()
            .intention(YieldIntention::Waiting(deps))
            .discovered(Vec::new())
            .build()
            .expect("Failed to build waiting")
    }

    fn new_ready() -> Self {
        YieldUpdateBuilder::default()
            .intention(YieldIntention::Ready)
            .discovered(Vec::new())
            .build()
            .expect("Failed to build ready")
    }

    fn new_suspended(
        outcome: TaskOutcome,
        discovered: Vec<SchedulerTask>,
    ) -> Self {
        YieldUpdateBuilder::default()
            .intention(YieldIntention::Suspended(outcome))
            .discovered(discovered)
            .build()
            .expect("Failed to build suspended")
    }

    fn with_ready(discovered: Vec<SchedulerTask>) -> Self {
        YieldUpdateBuilder::default()
            .intention(YieldIntention::Ready)
            .discovered(discovered)
            .build()
            .expect("Failed to build ready")
    }

    fn with_waiting(
        deps: Dependencies,
        discovered: Vec<SchedulerTask>,
    ) -> Self {
        YieldUpdateBuilder::default()
            .intention(YieldIntention::Waiting(deps))
            .discovered(discovered)
            .build()
            .expect("Failed to build waiting")
    }
}

/* TESTS */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::developer::GraphBuilder;

    const MODULE: &str = "mock-task-tests";

    #[test]
    fn build_single_task() -> Result<()> {
        let t1 = TaskConfigBuilder::default()
            .ticks(5)
            .release(5)
            .outcome(TaskOutcome::Success(0))
            .about("root task")
            .build()?;

        let graph = GraphBuilder::new();
        let task = TaskBuilder::new()
            .name("single")
            .graph(graph)
            .source(&t1)
            .build()?;

        task.visualize(MODULE)?;
        assert_eq!(task.name(), "single");
        assert_eq!(task.tid, 0);

        Ok(())
    }

    #[test]
    fn build_linear_chain() -> Result<()> {
        let t1 = TaskConfigBuilder::default()
            .ticks(2)
            .release(2)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let t2 = TaskConfigBuilder::default()
            .ticks(3)
            .release(3)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let t3 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&t1, &t2)
            .edge(&t2, &t3);

        let task = TaskBuilder::new()
            .name("linear")
            .graph(graph)
            .source(&t1)
            .build()?;

        task.visualize(MODULE)?;
        assert_eq!(task.graph.graph.node_count(), 3);

        Ok(())
    }

    #[test]
    fn build_tree_structure() -> Result<()> {
        let root = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let left = TaskConfigBuilder::default()
            .ticks(2)
            .release(2)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let right = TaskConfigBuilder::default()
            .ticks(2)
            .release(2)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let leaf1 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(3))
            .build()?;

        let leaf2 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(4))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&root, &left)
            .edge(&root, &right)
            .edge(&left, &leaf1)
            .edge(&right, &leaf2);

        let task = TaskBuilder::new()
            .name("tree")
            .graph(graph)
            .source(&root)
            .build()?;

        task.visualize(MODULE)?;
        assert_eq!(task.graph.graph.node_count(), 5);

        Ok(())
    }

    #[test]
    fn build_diamond_dependencies() -> Result<()> {
        let start = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let left = TaskConfigBuilder::default()
            .ticks(5)
            .release(5)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let right = TaskConfigBuilder::default()
            .ticks(10)
            .release(10)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let end = TaskConfigBuilder::default()
            .ticks(15)
            .release(15)
            .outcome(TaskOutcome::Success(3))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&start, &left)
            .edge(&start, &right)
            .edge(&left, &end)
            .edge(&right, &end);

        let task = TaskBuilder::new()
            .name("diamond")
            .graph(graph)
            .source(&start)
            .build()?;

        task.visualize(MODULE)?;
        let end_children = task.graph.children(3);

        assert_eq!(end_children.len(), 0);
        Ok(())
    }

    #[test]
    fn reuse_same_node() -> Result<()> {
        let shared = TaskConfigBuilder::default()
            .ticks(2)
            .release(2)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let t1 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let t2 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&t1, &shared)
            .edge(&t2, &shared);

        let task = TaskBuilder::new()
            .name("reused")
            .graph(graph)
            .source(&t1)
            .build()?;

        task.visualize(MODULE)?;
        assert_eq!(task.graph.graph.node_count(), 3);

        Ok(())
    }

    #[test]
    fn reject_cycle() -> Result<()> {
        let t1 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let t2 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let t3 = TaskConfigBuilder::default()
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

    #[test]
    fn reject_zero_ticks() -> Result<()> {
        let bad = TaskConfigBuilder::default()
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

    #[test]
    fn handle_disconnected_source() -> Result<()> {
        let t1 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let t2 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let graph = GraphBuilder::new().edge(&t1, &t2);
        let isolated = TaskConfigBuilder::default()
            .ticks(5)
            .release(5)
            .outcome(TaskOutcome::Success(99))
            .build()?;

        let task = TaskBuilder::new()
            .name("disconnected")
            .graph(graph)
            .source(&isolated)
            .build()?;

        task.visualize(MODULE)?;
        assert_eq!(task.graph.graph.node_count(), 3);
        Ok(())
    }

    #[test]
    fn create_root_task() -> Result<()> {
        let t1 = TaskConfigBuilder::default()
            .ticks(3)
            .release(3)
            .outcome(TaskOutcome::Success(42))
            .about("test task")
            .size(Some(100))
            .retriable(true)
            .build()?;

        let graph = GraphBuilder::new();
        let task = TaskBuilder::new()
            .name("root-test")
            .graph(graph)
            .source(&t1)
            .build()?;

        task.visualize(MODULE)?;
        assert_eq!(task.tid, 0);
        assert_eq!(task.config().about, "test task");
        assert_eq!(task.config().size, Some(100));
        assert!(task.config().retriable);

        Ok(())
    }

    #[test]
    fn test_waits_for_dependencies() -> Result<()> {
        let dep1 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .about("dependency 1")
            .build()?;

        let dep2 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .about("dependency 2")
            .build()?;

        let consumer = TaskConfigBuilder::default()
            .ticks(4)
            .release(0)
            .outcome(TaskOutcome::Success(0))
            .about("consumer")
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&consumer, &dep1)
            .edge(&consumer, &dep2);

        let task = TaskBuilder::new()
            .name("waits-for-deps")
            .graph(graph)
            .source(&consumer)
            .build()?;

        task.visualize(MODULE)?;

        let mut executable = Box::new(task);

        let empty = TaskOutcomes::new();

        // First tick: discover first child, return Ready
        let update1 = executable
            .tick(empty.clone())
            .ok_or(anyhow!(""))?;
        assert!(matches!(update1.intention, YieldIntention::Ready));
        assert_eq!(update1.discovered.len(), 1);

        // Second tick: discover second child, return Waiting for both
        let update2 = executable
            .tick(empty)
            .ok_or(anyhow!(""))?;
        let dep_tids: Vec<TaskID> = match &update2.intention {
            YieldIntention::Waiting(unsatisfied) => {
                assert_eq!(unsatisfied.len(), 2);
                unsatisfied
                    .iter()
                    .copied()
                    .collect()
            },
            _ => panic!("Expected Waiting on second tick"),
        };
        assert_eq!(update2.discovered.len(), 1);

        // Provide one dependency outcome
        let partial: TaskOutcomes = [(dep_tids[0], TaskOutcome::Success(1))]
            .into_iter()
            .collect();
        let update3 = executable
            .tick(partial)
            .ok_or(anyhow!(""))?;

        match update3.intention {
            YieldIntention::Waiting(ref unsatisfied) => {
                assert_eq!(unsatisfied.len(), 1);
                assert!(unsatisfied.contains(&dep_tids[1]));
            },
            _ => panic!("Expected Waiting with one dep"),
        }

        // Provide all dependency outcomes
        let full: TaskOutcomes = [
            (dep_tids[0], TaskOutcome::Success(1)),
            (dep_tids[1], TaskOutcome::Success(2)),
        ]
        .into_iter()
        .collect();

        // After all deps satisfied, should return Ready
        let update4 = executable
            .tick(full.clone())
            .ok_or(anyhow!(""))?;
        match update4.intention {
            YieldIntention::Ready => {},
            _ => panic!("Expected Ready after deps satisfied"),
        }

        // Continue execution
        let update5 = executable
            .tick(full.clone())
            .ok_or(anyhow!(""))?;
        match update5.intention {
            YieldIntention::Ready => {},
            _ => panic!("Expected Ready during execution"),
        }

        // Final tick to complete
        let update6 = executable
            .tick(full)
            .ok_or(anyhow!(""))?;
        match update6.intention {
            YieldIntention::Suspended(TaskOutcome::Success(0)) => {},
            _ => panic!("Expected Suspended after completion"),
        }

        Ok(())
    }

    #[test]
    fn test_staged_release() -> Result<()> {
        let child1 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let child2 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let child3 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(3))
            .build()?;

        let parent = TaskConfigBuilder::default()
            .ticks(5)
            .release(2)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&parent, &child1)
            .edge(&parent, &child2)
            .edge(&parent, &child3);

        let task = TaskBuilder::new()
            .name("staged-release")
            .graph(graph)
            .source(&parent)
            .build()?;

        task.visualize(MODULE)?;

        let mut executable = Box::new(task);
        let empty = TaskOutcomes::new();

        let update1 = executable
            .tick(empty.clone())
            .ok_or(anyhow!(""))?;
        assert!(matches!(update1.intention, YieldIntention::Ready));
        assert_eq!(update1.discovered.len(), 0);

        let update2 = executable
            .tick(empty.clone())
            .ok_or(anyhow!(""))?;
        assert!(matches!(update2.intention, YieldIntention::Ready));
        assert_eq!(update2.discovered.len(), 0);

        let update3 = executable
            .tick(empty.clone())
            .ok_or(anyhow!(""))?;
        assert!(matches!(update3.intention, YieldIntention::Ready));
        assert_eq!(update3.discovered.len(), 1);

        let update4 = executable
            .tick(empty.clone())
            .ok_or(anyhow!(""))?;
        assert!(matches!(update4.intention, YieldIntention::Ready));
        assert_eq!(update4.discovered.len(), 1);

        let update5 = executable
            .tick(empty)
            .ok_or(anyhow!(""))?;
        assert!(matches!(
            update5.intention,
            YieldIntention::Waiting(_)
        ));
        assert_eq!(update5.discovered.len(), 1);

        Ok(())
    }

    #[test]
    fn test_release_timeout() -> Result<()> {
        let child1 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let child2 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let child3 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(3))
            .build()?;

        let parent = TaskConfigBuilder::default()
            .ticks(3)
            .release(2)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&parent, &child1)
            .edge(&parent, &child2)
            .edge(&parent, &child3);

        let task = TaskBuilder::new()
            .name("release-timeout")
            .graph(graph)
            .source(&parent)
            .build()?;

        task.visualize(MODULE)?;

        let mut executable = Box::new(task);
        let empty = TaskOutcomes::new();

        let update1 = executable
            .tick(empty.clone())
            .ok_or(anyhow!(""))?;
        assert!(matches!(update1.intention, YieldIntention::Ready));
        assert_eq!(update1.discovered.len(), 0);

        let update2 = executable
            .tick(empty.clone())
            .ok_or(anyhow!(""))?;
        assert!(matches!(update2.intention, YieldIntention::Ready));
        assert_eq!(update2.discovered.len(), 0);

        let update3 = executable
            .tick(empty)
            .ok_or(anyhow!(""))?;
        assert!(matches!(
            update3.intention,
            YieldIntention::Waiting(_)
        ));
        assert_eq!(update3.discovered.len(), 3);

        Ok(())
    }

    #[test]
    fn test_immediate_release() -> Result<()> {
        let child1 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let child2 = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let parent = TaskConfigBuilder::default()
            .ticks(3)
            .release(0)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&parent, &child1)
            .edge(&parent, &child2);

        let task = TaskBuilder::new()
            .name("immediate-release")
            .graph(graph)
            .source(&parent)
            .build()?;

        task.visualize(MODULE)?;

        let mut executable = Box::new(task);
        let empty = TaskOutcomes::new();

        let update1 = executable
            .tick(empty.clone())
            .ok_or(anyhow!(""))?;
        assert!(matches!(update1.intention, YieldIntention::Ready));
        assert_eq!(update1.discovered.len(), 1);

        let update2 = executable
            .tick(empty)
            .ok_or(anyhow!(""))?;
        assert_eq!(update2.discovered.len(), 1);
        assert!(matches!(
            update2.intention,
            YieldIntention::Waiting(_)
        ));

        Ok(())
    }
}
