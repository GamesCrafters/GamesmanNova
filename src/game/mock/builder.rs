//! Mock Extensive Game Builder Pattern Module
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
use std::collections::HashSet;

use crate::game::error::GameError::MockBuilderViolation;
use crate::game::mock::Session;
use crate::game::mock::Stage;
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
/// let session = SessionBuilder::new(2)?
///     .edge(&s0, &s1)?
///     .edge(&s0, &s2)?
///     .edge(&s1, &s2)?
///     .start(&s0)
///     .build()?;
/// ```
pub struct SessionBuilder<'a> {
    inserted: HashMap<State, NodeIndex>,
    players: PlayerCount,
    start: Option<NodeIndex>,
    game: Graph<&'a Node, ()>,
}

/// Encodes a unique state or position in a game, which may be able to be
/// transitioned into other states. Contains all information necessary for
/// solving the represented game (no additional bookkeeping needed).
pub struct Node {
    pub hash: State,
    pub data: Stage,
}

/* BUILDER IMPLEMENTATION */

impl<'a> SessionBuilder<'a> {
    /// Initialize a builder struct for a graph game with an empty graph, no
    /// starting state, and a number of `players` (failing if it is 0).
    pub fn new(players: PlayerCount) -> Result<Self> {
        if players < 1 {
            Err(MockBuilderViolation {
                hint: "Games with 0 players are not allowed.".into(),
            })?
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
    pub fn edge(mut self, from: &'a Node, to: &'a Node) -> Result<Self> {
        self.check_edge_addition(from, to)?;

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

    /// Indicate that `node` is the starting state for the game being built. The
    /// indicated `node` must have already been added to the builder as part of
    /// an edge (failing otherwise).
    pub fn start(mut self, node: &Node) -> Result<Self> {
        if let Some(&index) = self.inserted.get(&node.hash) {
            self.start = Some(index);
            Ok(self)
        } else {
            Err(MockBuilderViolation {
                hint: format!(
                    "There is no existing node with hash {} in the game.",
                    node.hash,
                ),
            })?
        }
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

    /// Fails if adding an edge between the nodes `from` and `to` would result
    /// in an invalid state for the game under construction.
    fn check_edge_addition(&self, from: &Node, to: &Node) -> Result<()> {
        match &from.data {
            Stage::Terminal(_) => Err(MockBuilderViolation {
                hint: format!(
                    "The terminal node with hash {} was used on the outgoing \
                    side of an edge.",
                    from.hash
                ),
            })?,
            Stage::Medial(turn) => {
                if turn >= &self.players {
                    Err(MockBuilderViolation {
                        hint: format!(
                            "The medial node with hash {} has a turn of {} but \
                            the game under construction is {}-player.",
                            from.hash,
                            turn,
                            self.players
                        ),
                    })?
                }
            },
        };

        match &to.data {
            Stage::Terminal(vector) => {
                if vector.len() != self.players {
                    Err(MockBuilderViolation {
                        hint: format!(
                            "The terminal node with hash {} has a utility \
                            vector with {} entries, but the game under \
                            construction is {}-player.",
                            to.hash,
                            vector.len(),
                            self.players
                        ),
                    })?
                }
            },
            Stage::Medial(turn) => {
                if turn >= &self.players {
                    Err(MockBuilderViolation {
                        hint: format!(
                            "The medial node with hash {} has a turn of {} but \
                            the game under construction is {}-player.",
                            to.hash,
                            turn,
                            self.players
                        ),
                    })?
                }
            },
        };

        Ok(())
    }

    /// Fails if no starting state was specified or if there exists no node in
    /// the game graph whose `hash` is equal to it.
    fn check_starting_state(&self) -> Result<NodeIndex> {
        if let Some(index) = self.start {
            Ok(index)
        } else {
            Err(MockBuilderViolation {
                hint: "No starting state was specified for the game.".into(),
            })?
        }
    }

    /// Fails if there does not exist a traversable path between `start` and any
    /// node marked as terminal in the game graph.
    fn check_terminal_state(&self, start: NodeIndex) -> Result<()> {
        let mut seen = HashSet::new();
        let mut stack = Vec::new();
        stack.push(start);

        while let Some(index) = stack.pop() {
            if !seen.contains(&index) {
                seen.insert(index);
                let curr = self.game[index];
                if curr.terminal() {
                    return Ok(());
                } else {
                    stack.extend(
                        self.game
                            .neighbors(index)
                            .filter(|n| !seen.contains(n)),
                    );
                }
            }
        }

        Err(MockBuilderViolation {
            hint: format!(
                "No terminal state is reachable from starting state {}.",
                self.game[start].hash
            ),
        })?
    }

    /// Fails if there exists a node marked as medial in the game graph which
    /// does not have any outgoing edges.
    fn check_outgoing_edges(&self, start: NodeIndex) -> Result<()> {
        if let Some(index) = self
            .game
            .node_indices()
            .find(|&i| {
                self.game[i].medial() && self.game.neighbors(i).count() == 0
            })
        {
            Err(MockBuilderViolation {
                hint: format!(
                    "The medial state {} has no outgoing edges, which would \
                    represent a contradiction.",
                    self.game[index].hash
                ),
            })?
        } else {
            Ok(())
        }
    }
}

/* NODE IMPLEMENTATION */

impl Node {
    /// Returns true if and only if `self` is a terminal node.
    #[inline]
    pub const fn terminal(&self) -> bool {
        if let Stage::Terminal(_) = self.data {
            true
        } else {
            false
        }
    }

    /// Returns true if and only if `self` is a medial node.
    #[inline]
    pub const fn medial(&self) -> bool {
        if let Stage::Medial(_) = self.data {
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::node;

    #[test]
    fn cannot_initialize_with_no_players() {
        assert!(SessionBuilder::new(0).is_err())
    }

    #[test]
    fn cannot_add_incorrect_utility_entries() {
        let m = node!(0, 0);
        let t = node!(0, [1, 2, 3]);

        let game = SessionBuilder::new(2)
            .unwrap()
            .edge(&m, &t);

        assert!(game.is_err())
    }

    #[test]
    #[should_panic]
    fn cannot_add_outgoing_edge_to_terminal_node() {
        let m1 = node!(0, 0);
        let m2 = node!(1, 1);

        let tn = node!(3, [1, 2, 3, 4]);

        SessionBuilder::new(4).unwrap()
            .edge(&tn, &m1).unwrap() // Panic
            .edge(&m1, &m2).unwrap()
            .edge(&tn, &m2).unwrap() // Panic
            .edge(&m2, &m1).unwrap()
            .edge(&tn, &m1).unwrap() // Panic
            .edge(&m2, &tn).unwrap()
            .edge(&tn, &m2).unwrap() // Panic
            .edge(&m1, &m2).unwrap();
    }

    #[test]
    fn cannot_build_graph_with_no_starting_state() {
        let m1 = node!(0, 0);
        let t1 = node!(1, [1, 2]);

        let game1 = SessionBuilder::new(2)
            .unwrap()
            .edge(&m1, &t1)
            .unwrap()
            .build();

        let game2 = SessionBuilder::new(1)
            .unwrap()
            .build();

        assert!(game1.is_err());
        assert!(game2.is_err());
    }

    #[test]
    fn cannot_build_game_with_no_accessible_end() -> Result<()> {
        let a = node!(0, 2);
        let b = node!(1, 1);
        let c = node!(2, 0);
        let d = node!(3, 1);

        let end = node!(4, [1, 2, 3]);

        let game = SessionBuilder::new(3)?
            .edge(&a, &b)?
            .edge(&c, &d)?
            .edge(&d, &end)?
            .start(&a)?
            .build();

        assert!(game.is_err());
        Ok(())
    }

    #[test]
    fn cannot_build_game_with_medial_traps() -> Result<()> {
        let a = node!(0, 0);
        let b = node!(1, 1);
        let c = node!(2, 2);
        let d = node!(3, 1);

        let trap = node!(4, 0);

        let end = node!(5, [1, 2, 3]);

        let game = SessionBuilder::new(3)?
            .edge(&a, &b)?
            .edge(&b, &c)?
            .edge(&c, &d)?
            .edge(&d, &end)?
            .edge(&b, &trap)?
            .start(&a)?
            .build();

        assert!(game.is_err());
        Ok(())
    }

    #[test]
    fn build_simple_acyclic_game() -> Result<()> {
        let a = node!(0, 0);
        let b = node!(1, 1);
        let c = node!(2, 0);
        let d = node!(3, 1);
        let e = node!(4, 0);
        let f = node!(5, 1);

        let t1 = node!(6, [1, 2]);
        let t2 = node!(7, [2, 1]);

        SessionBuilder::new(2)?
            .edge(&a, &b)?
            .edge(&b, &c)?
            .edge(&a, &c)?
            .edge(&c, &d)?
            .edge(&d, &e)?
            .edge(&b, &d)?
            .edge(&e, &f)?
            .edge(&c, &t1)?
            .edge(&f, &t2)?
            .start(&a)?
            .build()?;

        Ok(())
    }

    #[test]
    fn build_simple_cyclic_game() -> Result<()> {
        let a = node!(0, 0);
        let b = node!(1, 1);
        let c = node!(2, 0);
        let d = node!(3, 1);
        let e = node!(4, 0);
        let f = node!(5, 1);

        let t1 = node!(6, [1, 2]);
        let t2 = node!(7, [2, 1]);

        SessionBuilder::new(2)?
            .edge(&a, &b)?
            .edge(&b, &c)?
            .edge(&c, &a)?
            .edge(&c, &d)?
            .edge(&c, &t1)?
            .edge(&d, &b)?
            .edge(&d, &e)?
            .edge(&e, &f)?
            .edge(&f, &t2)?
            .start(&a)?
            .build()?;

        Ok(())
    }
}
