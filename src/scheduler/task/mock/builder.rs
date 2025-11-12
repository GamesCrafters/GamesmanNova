//! Builder pattern for constructing mock task graphs.

use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::sync::Arc;

use crate::developer::GraphBuilder;
use crate::scheduler::TaskID;
use crate::scheduler::task::mock::CompiledGraph;
use crate::scheduler::task::mock::Task;
use crate::scheduler::task::mock::TaskConfig;

/* STRUCTURES */

pub struct TaskBuilder<'a> {
    pub source: Option<&'a TaskConfig>,
    pub graph: Option<GraphBuilder<'a, TaskConfig>>,
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

    pub fn graph(mut self, graph: GraphBuilder<'a, TaskConfig>) -> Self {
        self.graph = Some(graph);
        self
    }

    pub fn source(mut self, node: &'a TaskConfig) -> Self {
        self.source = Some(node);
        self
    }

    pub fn build(self) -> Result<Task> {
        let source = self
            .source
            .ok_or_else(|| anyhow!("No source"))?;

        let name = self
            .name
            .ok_or_else(|| anyhow!("No name"))?;

        let mut input = self
            .graph
            .ok_or_else(|| anyhow!("No graph"))?;

        let root = Self::ensure_source(&mut input, source)?;

        Self::check_acyclic(&input.graph)?;
        Self::validate_configs(&input)?;

        let compiled = Self::compile(&input, root, name)?;

        let task = Task::new(compiled.root, compiled);

        Ok(task)
    }

    /* HELPER METHODS */

    fn ensure_source(
        graph: &mut GraphBuilder<'a, TaskConfig>,
        node: &'a TaskConfig,
    ) -> Result<NodeIndex> {
        let ptr = node as *const TaskConfig;

        if let Some(&index) = graph.inserted.get(&ptr) {
            Ok(index)
        } else {
            let index = graph.graph.add_node(node);
            graph.inserted.insert(ptr, index);
            Ok(index)
        }
    }

    fn check_acyclic(graph: &petgraph::Graph<&TaskConfig, ()>) -> Result<()> {
        let result = petgraph::algo::toposort(graph, None);

        if result.is_err() {
            bail!("Graph contains cycles");
        }

        Ok(())
    }

    fn validate_configs(graph: &GraphBuilder<'a, TaskConfig>) -> Result<()> {
        for index in graph.graph.node_indices() {
            let config = graph.graph[index];

            if config.ticks == 0 {
                bail!("Task cannot have zero ticks");
            }
        }

        Ok(())
    }

    fn compile(
        input: &GraphBuilder<'a, TaskConfig>,
        root: NodeIndex,
        name: &'static str,
    ) -> Result<Arc<CompiledGraph>> {
        let mut graph = petgraph::Graph::new();

        for index in input.graph.node_indices() {
            let config = input.graph[index].clone();
            graph.add_node(config);
        }

        for edge in input.graph.edge_references() {
            let src = edge.source();
            let dst = edge.target();
            graph.add_edge(src, dst, ());
        }

        let compiled = CompiledGraph {
            graph,
            name: name.to_string(),
            root: root.index() as TaskID,
        };

        let arc = Arc::new(compiled);
        Ok(arc)
    }
}

/* TESTS */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::TaskOutcome;
    use crate::scheduler::task::mock::TaskConfigBuilder;

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

    #[test]
    fn reject_missing_name() -> Result<()> {
        let config = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let graph = GraphBuilder::new();
        let result = TaskBuilder::new()
            .source(&config)
            .graph(graph)
            .build();

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn reject_missing_graph() -> Result<()> {
        let config = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

        let result = TaskBuilder::new()
            .name("missing-graph")
            .source(&config)
            .build();

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn extract_discoveries_correctly() -> Result<()> {
        let parent = TaskConfigBuilder::default()
            .ticks(1)
            .release(1)
            .outcome(TaskOutcome::Success(0))
            .build()?;

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

        let graph = GraphBuilder::new()
            .edge(&parent, &child1)
            .edge(&parent, &child2);

        let task = TaskBuilder::new()
            .name("discoveries-test")
            .graph(graph)
            .source(&parent)
            .build()?;

        let children = task.children();
        assert_eq!(children.len(), 2);

        Ok(())
    }
}
