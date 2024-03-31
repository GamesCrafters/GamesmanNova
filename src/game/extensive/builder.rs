//! Extensive Game Builder Pattern Module
//!
//! This module provides an implementation of a declarative builder pattern for
//! an extensive-form game `Session`, which allows the construction of a graph
//! of nodes representing game states.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/31/2024

use anyhow::Result;
use petgraph::{graph::NodeIndex, Graph};

use std::collections::HashMap;

use crate::game::extensive::Node;
use crate::game::extensive::Session;
use crate::game::extensive::Stage;
use crate::model::{PlayerCount, State};

/* DEFINITIONS */

/// Builder pattern for creating a graph game by progressively adding nodes and
/// edges and specifying a start node. `Node`s are unique by their `hash` field
/// and contain additional information, and edges are directed and unweighted.
///
/// # Example
///
/// ```no_run
/// // Long-form node initialization
/// let s0 = Node { 0, Stage::Medial(0) };
/// let s1 = Node { 1, Stage::Medial(1) };
/// let s2 = Node { 2, Stage::Terminal(vec![1, -1]) };
///
/// // Macro node initialization
/// let s0 = node!(0, 0);
/// let s1 = node!(1, 1);
/// let s2 = node!(2, [1, -1]);
///
/// let session = SessionBuilder::new()?
///     .edge(&s0, &s1)?
///     .edge(&s0, &s2)?
///     .edge(&s1, &s2)?
///     .start(&s0)
///     .build()?;
/// ```
pub struct SessionBuilder<'a> {
    inserted: HashMap<State, NodeIndex>,
    players: PlayerCount,
    start: Option<State>,
    game: Graph<&'a Node, ()>,
}

/* IMPLEMENTATION */

impl<'a> SessionBuilder<'a> {
    /// Initialize a builder struct for a graph game with an empty graph, no
    /// starting state, and a number of `players` (failing if it is 0).
    pub fn new(players: PlayerCount) -> Result<Self> {
        if players < 1 {
            Err(todo!())
        } else {
            Ok(SessionBuilder {
                inserted: HashMap::new(),
                players,
                start: None,
                game: Graph::new(),
            })
        }
    }

    /// Insert an edge into the game graph under construction. If either of the
    /// nodes contains a previously inserted `hash` belonging to another node,
    /// the existing node will be used for the edge, and the additional fields
    /// in the new node will be ignored. Fails if `from` is a terminal node, or
    /// if a terminal node with a utility vector of incoherent size is added.
    pub fn edge(&mut self, from: &'a Node, to: &'a Node) -> Result<&mut Self> {
        if let Stage::Terminal(vector) = &to.data {
            if vector.len() != self.players {
                return Err(todo!());
            }
        } else if let Stage::Terminal(_) = &from.data {
            return Err(todo!());
        }

        let i = *self
            .inserted
            .entry(from.hash)
            .or_insert_with(|| self.game.add_node(from));

        let j = *self
            .inserted
            .entry(to.hash)
            .or_insert_with(|| self.game.add_node(to));

        self.game.add_edge(i, j, ());
        Ok(self)
    }

    /// Indicate that the node with a `state` hash is the starting state or
    /// position in the underlying game. The last specified start state before
    /// a call to `build` will be the only start state for the game (if it
    /// corresponds to a valid node in the constructed graph).
    pub fn start(&mut self, node: &Node) -> &mut Self {
        self.start = Some(node.hash);
        self
    }

    /// Instantiate a `Session` encoding the constructed game graph. Fails if
    /// the number of players is set to 0, if no starting state was specified,
    /// the specified starting state does not correspond to any node `hash`,
    /// there exist non-terminal nodes with no outgoing edges, or no terminal
    /// nodes are reachable from the starting state (assuming it is valid).
    pub fn build(self) -> Result<Session<'a>> {
        let index = self.check_starting_state()?;
        self.check_terminal_state(index)?;
        self.check_outgoing_edges(index)?;

        Ok(Session {
            players: self.players,
            start: index,
            game: self.game,
        })
    }

    /* VALIDATION METHODS */

    /// Fails if no starting state was specified or if there exists no node in
    /// the game graph whose `hash` is equal to it.
    fn check_starting_state(&self) -> Result<NodeIndex> {
        if let Some(state) = self.start {
            if let Some(&index) = self.inserted.get(&state) {
                Ok(index)
            } else {
                Err(todo!())
            }
        } else {
            Err(todo!())
        }
    }

    /// Fails if there does not exist a path between `start` and any node marked
    /// as terminal in the game graph.
    fn check_terminal_state(&self, start: NodeIndex) -> Result<()> {
        todo!()
    }

    /// Fails if there exists a node marked as medial in the game graph which
    /// does not have any outgoing edges.
    fn check_outgoing_edges(&self, start: NodeIndex) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn cannot_add_outgoing_edge_to_terminal_node() {
        todo!()
    }

    #[test]
    fn cannot_build_graph_with_no_starting_state() {
        todo!()
    }

    #[test]
    fn cannot_build_game_with_no_accessible_end() {
        todo!()
    }

    #[test]
    fn cannot_build_game_with_medial_node_traps() {
        todo!()
    }

    #[test]
    fn build_simple_acyclic_game() {
        todo!()
    }

    #[test]
    fn build_simple_cyclic_game() {
        todo!()
    }

    #[test]
    fn build_complicated_acyclic_game() {
        todo!()
    }

    #[test]
    fn build_complicated_cyclic_game() {
        todo!()
    }
}
