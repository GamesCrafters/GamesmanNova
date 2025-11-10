//! Mock Game Builder Pattern Implementations
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use petgraph::Direction;
use petgraph::Graph;
use petgraph::graph::NodeIndex;

use std::collections::HashSet;

use crate::core::database::Schema;
use crate::core::database::SchemaBuilder;
use crate::core::developer::GraphBuilder;
use crate::core::game::PlayerCount;
use crate::core::game::mock::Node;
use crate::core::game::mock::Session;

/* STRUCTURES */

pub struct SessionBuilder<'a> {
    pub source: Option<&'a Node>,
    pub graph: Option<GraphBuilder<'a, Node>>,
    pub name: Option<&'static str>,
}

/* IMPLEMENTATIONS */

impl<'a> SessionBuilder<'a> {
    pub fn new() -> Self {
        SessionBuilder {
            source: None,
            graph: None,
            name: None,
        }
    }

    pub fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn graph(mut self, graph: GraphBuilder<'a, Node>) -> Self {
        self.graph = Some(graph);
        self
    }

    pub fn source(mut self, node: &'a Node) -> Self {
        self.source = Some(node);
        self
    }

    pub fn build(self) -> Result<Session<'a>> {
        let source_node = self
            .source
            .ok_or_else(|| anyhow!("No source node specified for game"))?;

        let name = self
            .name
            .ok_or_else(|| anyhow!("No name specified for game"))?;

        let graph = self
            .graph
            .ok_or_else(|| anyhow!("No graph specified for game"))?;

        let players = Self::validate_player_counts(&graph, name)?;
        let source = Self::check_source_state(&graph, source_node, name)?;
        Self::check_terminal_state(&graph.graph, source, name)?;
        Self::check_outgoing_edges(&graph.graph, name)?;

        let schema = Self::schema(players, name)?;
        let inserted = graph.inserted;
        let game = graph.graph;

        Ok(Session {
            inserted,
            players,
            schema,
            source,
            game,
            name,
        })
    }

    /* HELPER METHODS */

    fn validate_player_counts(
        graph: &GraphBuilder<'a, Node>,
        name: &str,
    ) -> Result<PlayerCount> {
        let mut state: Option<(PlayerCount, bool)> = None;

        for index in graph.graph.node_indices() {
            let node = graph.graph[index];
            Self::check_terminal_edges(graph, index, node, name)?;

            let (count, terminal) = Self::extract_count(node, name)?;
            state = Self::update_count(state, count, terminal, name)?;
        }

        state
            .map(|(count, _)| count)
            .context("No nodes in graph")
    }

    fn check_terminal_edges(
        graph: &GraphBuilder<'a, Node>,
        index: NodeIndex,
        node: &Node,
        name: &str,
    ) -> Result<()> {
        let outgoing = graph
            .graph
            .neighbors_directed(index, Direction::Outgoing)
            .count();

        if node.terminal() && outgoing > 0 {
            bail! {
                "There was an attempt to add a terminal node on the outgoing \
                side of an edge during the construction of the game '{}'.",
                name,
            }
        }

        Ok(())
    }

    fn extract_count(node: &Node, name: &str) -> Result<(PlayerCount, bool)> {
        match node {
            Node::Medial(turn) => Ok((*turn + 1, false)),
            Node::Terminal(player, vector) => {
                let count = vector.len();
                if *player >= vector.len() {
                    bail! {
                        "While constructing the game '{}', there was an \
                        attempt to add a terminal node containing a turn that \
                        would not have a corresponding utility entry.",
                        name,
                    }
                }

                if count == 0 {
                    bail! {
                        "While constructing the game '{}', there was an \
                        attempt to add a terminal node containing no utility \
                        entries. Games with no players are not allowed.",
                        name,
                    }
                }

                Ok((count, true))
            },
        }
    }

    fn update_count(
        state: Option<(PlayerCount, bool)>,
        count: PlayerCount,
        terminal: bool,
        name: &str,
    ) -> Result<Option<(PlayerCount, bool)>> {
        if let Some((old, finalized)) = state {
            Self::validate_consistency(old, finalized, count, terminal, name)?;
            let updated = Self::merge_counts(old, finalized, count, terminal);
            Ok(Some(updated))
        } else {
            Ok(Some((count, terminal)))
        }
    }

    fn validate_consistency(
        old: PlayerCount,
        finalized: bool,
        new: PlayerCount,
        terminal: bool,
        name: &str,
    ) -> Result<()> {
        if finalized && terminal && old != new {
            bail! {
                "While constructing the game '{}', a terminal node was added \
                containing {} utility entries, but then a new one was added \
                with {} entries. Utility entries must be consistent across all \
                terminal nodes.",
                name, old, new,
            }
        }

        if finalized && !terminal && new > old {
            bail! {
                "While constructing the game '{}', a terminal node was added \
                containing {} utility entries, but then a new medial node was \
                added with a 0-indexed turn of {}, which is incompatible.",
                name, old, new - 1,
            }
        }

        if !finalized && terminal && new < old {
            bail! {
                "While constructing the game '{}', a medial node was added at \
                a 0-indexed turn of {}, but then a new terminal node was added \
                with {} entries. All turn indicators must be able to index \
                terminal nodes' utility entries.",
                name, old - 1, new,
            }
        }

        Ok(())
    }

    fn merge_counts(
        old: PlayerCount,
        finalized: bool,
        new: PlayerCount,
        terminal: bool,
    ) -> (PlayerCount, bool) {
        if terminal {
            (new, true)
        } else if !finalized && new > old {
            (new, false)
        } else {
            (old, finalized)
        }
    }

    fn check_source_state(
        graph: &GraphBuilder<'a, Node>,
        node: &Node,
        name: &str,
    ) -> Result<NodeIndex> {
        if let Some(&index) = graph
            .inserted
            .get(&(node as *const Node))
        {
            Ok(index)
        } else {
            bail! {
                "There was an attempt to set the source state of mock game \
                '{}', but the indicated source node has not been added to the \
                game yet.",
                name,
            }
        }
    }

    fn check_terminal_state(
        graph: &Graph<&Node, ()>,
        source: NodeIndex,
        name: &str,
    ) -> Result<()> {
        let mut seen = HashSet::new();
        let mut stack = Vec::new();
        stack.push(source);

        while let Some(index) = stack.pop() {
            if !seen.contains(&index) {
                seen.insert(index);
                let curr = graph[index];
                if curr.terminal() {
                    return Ok(());
                } else {
                    stack.extend(
                        graph
                            .neighbors_directed(index, Direction::Outgoing)
                            .filter(|n| !seen.contains(n)),
                    );
                }
            }
        }

        bail! {
            "No terminal node is reachable from the node marked as the source \
            in the game '{}'.",
            name
        }
    }

    fn check_outgoing_edges(
        graph: &Graph<&Node, ()>,
        name: &str,
    ) -> Result<()> {
        let trapped = |i| {
            let outgoing = graph
                .neighbors_directed(i, Direction::Outgoing)
                .count();

            graph[i].medial() && outgoing == 0
        };

        if graph.node_indices().any(trapped) {
            bail! {
                "There exists a medial state with no outgoing edges in the \
                constructed game '{}', which is a contradiction.",
                name
            }
        }

        Ok(())
    }

    fn schema(players: PlayerCount, table: &str) -> Result<Schema> {
        SchemaBuilder::new(table)
            .players(players)
            .key("state", "INTEGER")
            .column("remoteness", "INTEGER")
            .column("player", "INTEGER")
            .build()
    }
}

impl Node {
    /// Returns true if and only if `self` is a terminal node.
    #[inline]
    pub const fn terminal(&self) -> bool {
        matches!(self, Node::Terminal(_, _))
    }

    /// Returns true if and only if `self` is a medial node.
    #[inline]
    pub const fn medial(&self) -> bool {
        matches!(self, Node::Medial(_))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::node;

    const MODULE_NAME: &str = "mock-game-builder-tests";

    #[test]
    fn cannot_add_incorrect_utility_entries() -> Result<()> {
        let m1 = node!(0);
        let m2 = node!(2);
        let m3 = node!(0);

        let t1 = node![1; 1, 2];
        let t2 = node![2; 3, 2, 1];
        let t3 = Node::Terminal(1, vec![]);

        let graph1 = GraphBuilder::new()
            .edge(&m1, &t1)
            .edge(&m1, &t2);

        let game = SessionBuilder::new()
            .name("bad utility 1")
            .graph(graph1)
            .source(&m1)
            .build();

        assert!(game.is_err());

        let graph2 = GraphBuilder::new()
            .edge(&m1, &m2)
            .edge(&m2, &t1);

        let game = SessionBuilder::new()
            .name("bad utility 2")
            .graph(graph2)
            .source(&m1)
            .build();

        assert!(game.is_err());

        let graph3 = GraphBuilder::new()
            .edge(&m1, &m3)
            .edge(&m3, &t3);

        let game = SessionBuilder::new()
            .name("bad utility 3")
            .graph(graph3)
            .source(&m1)
            .build();

        assert!(game.is_err());

        Ok(())
    }

    #[test]
    fn cannot_add_incorrect_turn_information_medial() -> Result<()> {
        let m1 = node!(0);
        let m2 = node!(2);

        let t1 = node![0; 1, -2];
        let t2 = node![1; -1, 2];

        let graph = GraphBuilder::new()
            .edge(&m1, &t1)
            .edge(&m1, &t2)
            .edge(&m1, &m2);

        let game = SessionBuilder::new()
            .name("bad turn")
            .graph(graph)
            .source(&m1)
            .build();

        assert!(game.is_err());
        Ok(())
    }

    #[test]
    fn cannot_add_incorrect_turn_information_terminal() -> Result<()> {
        let m1 = node!(0);
        let m2 = node!(1);

        let t1 = node![0; 1, -2];
        let t2 = node![2; -1, 2];

        let graph = GraphBuilder::new()
            .edge(&m1, &m2)
            .edge(&m1, &t1)
            .edge(&m1, &t2);

        let game = SessionBuilder::new()
            .name("bad turn")
            .graph(graph)
            .source(&m1)
            .build();

        assert!(game.is_err());
        Ok(())
    }

    #[test]
    fn cannot_add_inconsistent_turn_information() -> Result<()> {
        let m1 = node!(0);
        let m2 = node!(1);

        let t1 = node![0; 1, -2];
        let t2 = node![1; -1];

        let graph = GraphBuilder::new()
            .edge(&m1, &m2)
            .edge(&m1, &t1)
            .edge(&m1, &t2);

        let game = SessionBuilder::new()
            .name("bad turn")
            .graph(graph)
            .source(&m1)
            .build();

        assert!(game.is_err());
        Ok(())
    }

    #[test]
    fn cannot_add_outgoing_edge_to_terminal_node() {
        let m1 = node!(0);
        let m2 = node!(1);

        let t1 = node![2; 1, 2, 3, 4];

        let graph = GraphBuilder::new()
            .edge(&t1, &m1)
            .edge(&m1, &m2)
            .edge(&t1, &m2)
            .edge(&m2, &m1)
            .edge(&t1, &m1)
            .edge(&m2, &t1)
            .edge(&t1, &m2)
            .edge(&m1, &m2);

        let game = SessionBuilder::new()
            .name("edge from terminal node")
            .graph(graph)
            .source(&m1)
            .build();

        assert!(game.is_err());
    }

    #[test]
    fn cannot_build_graph_with_no_source_state() -> Result<()> {
        let m1 = node!(0);
        let t1 = node![0; 1, 2];

        let game1 = SessionBuilder::new()
            .name("no source state 1")
            .build();

        assert!(game1.is_err());

        let graph2 = GraphBuilder::new().edge(&m1, &t1);
        let game2 = SessionBuilder::new()
            .name("no source state 2")
            .graph(graph2)
            .build();

        assert!(game2.is_err());

        Ok(())
    }

    #[test]
    fn cannot_build_game_with_no_accessible_sink() -> Result<()> {
        let a = node!(2);
        let b = node!(1);
        let c = node!(0);
        let d = node!(1);

        let sink = node![1; 1, 2, 3];

        let graph = GraphBuilder::new()
            .edge(&a, &b)
            .edge(&c, &d)
            .edge(&d, &sink);

        let game = SessionBuilder::new()
            .name("no sink")
            .graph(graph)
            .source(&a)
            .build();

        assert!(game.is_err());
        Ok(())
    }

    #[test]
    fn cannot_build_game_with_medial_traps() -> Result<()> {
        let a = node!(0);
        let b = node!(1);
        let c = node!(2);
        let d = node!(1);

        let trap = node!(0);
        let sink = node![0; 1, 2, 3];

        let graph = GraphBuilder::new()
            .edge(&a, &b)
            .edge(&b, &c)
            .edge(&c, &d)
            .edge(&d, &sink)
            .edge(&b, &trap);

        let game = SessionBuilder::new()
            .name("trap game")
            .graph(graph)
            .source(&a)
            .build();

        assert!(game.is_err());
        Ok(())
    }

    #[test]
    fn build_simple_acyclic_game() -> Result<()> {
        let a = node!(0);
        let b = node!(1);
        let c = node!(0);
        let d = node!(1);
        let e = node!(0);
        let f = node!(1);

        let t1 = node![1; 1, 2];
        let t2 = node![0; 2, 1];

        let graph = GraphBuilder::new()
            .edge(&a, &b)
            .edge(&b, &c)
            .edge(&a, &c)
            .edge(&c, &d)
            .edge(&d, &e)
            .edge(&b, &d)
            .edge(&e, &f)
            .edge(&c, &t1)
            .edge(&f, &t2);

        let game = SessionBuilder::new()
            .name("acyclic")
            .graph(graph)
            .source(&a)
            .build()?;

        game.visualize(MODULE_NAME)?;
        assert_eq!(game.players, 2);

        Ok(())
    }

    #[test]
    fn build_simple_cyclic_game() -> Result<()> {
        let a = node!(0);
        let b = node!(1);
        let c = node!(0);
        let d = node!(3);
        let e = node!(0);
        let f = node!(1);

        let t1 = node![2; 1, 2, -1, 4];
        let t2 = node![3; 2, 1, 9, -6];

        let graph = GraphBuilder::new()
            .edge(&a, &b)
            .edge(&b, &c)
            .edge(&c, &a)
            .edge(&c, &d)
            .edge(&c, &t1)
            .edge(&d, &b)
            .edge(&d, &e)
            .edge(&e, &f)
            .edge(&f, &t2);

        let game = SessionBuilder::new()
            .name("cyclic")
            .graph(graph)
            .source(&a)
            .build()?;

        game.visualize(MODULE_NAME)?;
        assert_eq!(game.players, 4);

        Ok(())
    }
}
