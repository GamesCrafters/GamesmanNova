//! # Mock Task Builder Pattern Implementation
//!
//! TODO

use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use petgraph::Direction;
use petgraph::graph::NodeIndex;

use std::collections::HashMap;

use crate::core::developer::GraphBuilder;
use crate::core::scheduler::Dependencies;
use crate::core::scheduler::TaskID;
use crate::core::scheduler::task::mock::GlobalData;
use crate::core::scheduler::task::mock::TaskData;
use crate::core::scheduler::task::mock::TaskGraph;
use crate::core::scheduler::task::mock::TaskNode;

/* STRUCTURES */

pub struct TaskBuilder<'a> {
    pub source: Option<&'a TaskNode>,
    pub graph: Option<GraphBuilder<'a, TaskNode>>,
    pub name: Option<&'static str>,
}

/* IMPLEMENTATIONS */

impl<'a> TaskBuilder<'a> {
    pub fn new() -> Self {
        TaskBuilder {
            source: None,
            graph: None,
            name: None,
        }
    }

    pub fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn graph(mut self, graph: GraphBuilder<'a, TaskNode>) -> Self {
        self.graph = Some(graph);
        self
    }

    pub fn source(mut self, node: &'a TaskNode) -> Self {
        self.source = Some(node);
        self
    }

    pub fn build(self) -> Result<TaskGraph<'a>> {
        let source = self
            .source
            .ok_or_else(|| anyhow!("No source node specified"))?;

        let name = self
            .name
            .ok_or_else(|| anyhow!("No name specified"))?;

        let mut graph = self
            .graph
            .ok_or_else(|| anyhow!("No graph specified"))?;

        let root = Self::ensure_source(&mut graph, source)?;
        Self::check_acyclic(&graph.graph)?;
        Self::validate_configs(&graph)?;

        let compiled = Self::compile(&graph)?;
        let inserted = graph.inserted;
        let petgraph = graph.graph;

        Ok(TaskGraph {
            inserted,
            graph: petgraph,
            compiled,
            root: root.index() as TaskID,
            name,
        })
    }

    /* HELPER METHODS */

    fn ensure_source(
        graph: &mut GraphBuilder<'a, TaskNode>,
        node: &'a TaskNode,
    ) -> Result<NodeIndex> {
        let ptr = node as *const TaskNode;

        if let Some(&index) = graph.inserted.get(&ptr) {
            Ok(index)
        } else {
            let index = graph.graph.add_node(node);
            graph.inserted.insert(ptr, index);
            Ok(index)
        }
    }

    fn check_acyclic(graph: &petgraph::Graph<&TaskNode, ()>) -> Result<()> {
        let cycle = petgraph::algo::toposort(graph, None);

        if cycle.is_err() {
            bail!("Task graph contains dependency cycles");
        }

        Ok(())
    }

    fn validate_configs(graph: &GraphBuilder<'a, TaskNode>) -> Result<()> {
        for index in graph.graph.node_indices() {
            let node = graph.graph[index];

            if node.ticks == 0 {
                bail!("Task cannot have zero ticks");
            }

            if node.release > node.ticks {
                bail!(
                    "Task release ({}) cannot exceed ticks ({})",
                    node.release,
                    node.ticks
                );
            }
        }

        Ok(())
    }

    fn compile(graph: &GraphBuilder<'a, TaskNode>) -> Result<GlobalData> {
        let mut data = HashMap::new();

        for index in graph.graph.node_indices() {
            let tid = index.index() as TaskID;
            let node = graph.graph[index];
            let config = node.clone();

            let discovers = Self::extract_discoveries(graph, index);
            let dependencies = Self::extract_dependencies(graph, index);
            let task = TaskData {
                dependencies,
                discovers,
                config,
            };

            data.insert(tid, task);
        }

        Ok(GlobalData::new(data))
    }

    fn extract_discoveries(
        graph: &GraphBuilder<'a, TaskNode>,
        index: NodeIndex,
    ) -> Vec<TaskID> {
        graph
            .graph
            .neighbors_directed(index, Direction::Outgoing)
            .map(|n| n.index() as TaskID)
            .collect()
    }

    fn extract_dependencies(
        graph: &GraphBuilder<'a, TaskNode>,
        index: NodeIndex,
    ) -> Dependencies {
        graph
            .graph
            .neighbors_directed(index, Direction::Incoming)
            .map(|n| n.index() as TaskID)
            .collect()
    }
}

/* TESTS */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::scheduler::TaskOutcome;
    use crate::core::scheduler::task::mock::TaskNodeBuilder;

    /// Tests that building without a source node returns an error.
    #[test]
    fn reject_missing_source() -> Result<()> {
        let graph = GraphBuilder::new();
        let result = TaskBuilder::new()
            .name("missing-source")
            .graph(graph)
            .build();

        assert!(result.is_err());
        Ok(())
    }

    /// Tests that building without a name returns an error.
    #[test]
    fn reject_missing_name() -> Result<()> {
        let node = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let graph = GraphBuilder::new();
        let result = TaskBuilder::new()
            .source(&node)
            .graph(graph)
            .build();

        assert!(result.is_err());
        Ok(())
    }

    /// Tests that building without a graph returns an error.
    #[test]
    fn reject_missing_graph() -> Result<()> {
        let node = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let result = TaskBuilder::new()
            .name("missing-graph")
            .source(&node)
            .build();

        assert!(result.is_err());
        Ok(())
    }

    /// Tests that release exceeding ticks is rejected.
    #[test]
    fn reject_release_exceeds_ticks() -> Result<()> {
        let node = TaskNodeBuilder::default()
            .ticks(5)
            .release(10)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let graph = GraphBuilder::new();
        let result = TaskBuilder::new()
            .name("invalid-release")
            .graph(graph)
            .source(&node)
            .build();

        assert!(result.is_err());
        Ok(())
    }

    /// Tests that dependencies are correctly extracted from incoming edges.
    #[test]
    fn extract_dependencies_correctly() -> Result<()> {
        let dep1 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(1))
            .build()?;

        let dep2 = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(2))
            .build()?;

        let consumer = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let graph = GraphBuilder::new()
            .edge(&dep1, &consumer)
            .edge(&dep2, &consumer);

        let task_graph = TaskBuilder::new()
            .name("deps-test")
            .graph(graph)
            .source(&consumer)
            .build()?;

        let root_tid = task_graph.root;
        let consumer_data = task_graph
            .compiled
            .get(root_tid)
            .unwrap();

        assert_eq!(consumer_data.dependencies.len(), 2);
        Ok(())
    }

    /// Tests that discoveries are correctly extracted from outgoing edges.
    #[test]
    fn extract_discoveries_correctly() -> Result<()> {
        let parent = TaskNodeBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

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

        let graph = GraphBuilder::new()
            .edge(&parent, &child1)
            .edge(&parent, &child2);

        let task_graph = TaskBuilder::new()
            .name("discoveries-test")
            .graph(graph)
            .source(&parent)
            .build()?;

        let root_tid = task_graph.root;
        let parent_data = task_graph
            .compiled
            .get(root_tid)
            .unwrap();

        assert_eq!(parent_data.discovers.len(), 2);
        Ok(())
    }
}
