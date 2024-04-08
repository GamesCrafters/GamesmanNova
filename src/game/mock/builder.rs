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
use petgraph::Direction;
use petgraph::{graph::NodeIndex, Graph};

use std::collections::{HashMap, HashSet};

use crate::game::error::GameError::MockViolation;
use crate::game::mock::Node;
use crate::game::mock::Session;
use crate::model::PlayerCount;

/* DEFINITIONS */

/// Semantics. Used to identify whether the current player count is final.
type Finalized = bool;

/// Builder pattern for creating a graph game by progressively adding nodes and
/// edges and specifying a start node. Directed unweighed edges represent
/// represent state transitions, and nodes containing either turn information
/// or utility vectors store the information necessary to solve the game being
/// represented.
///
/// # Example
///
/// ```no_run
/// // Long-form node initialization
/// let s0 = Node::Medial(0);
/// let s1 = Node::Medial(1);
/// let s2 = Node::Terminal(vec![1, -1]);
///
/// // Macro node initialization (equivalent)
/// let s0 = node!(0);
/// let s1 = node!(1);
/// let s2 = node!([1, -1]);
///
/// let session = SessionBuilder::new("example")
///     .edge(&s0, &s1)?
///     .edge(&s0, &s2)?
///     .edge(&s1, &s2)?
///     .start(&s0)?
///     .build()?;
///
/// assert_eq!(session.players, 2);
/// ```
pub struct SessionBuilder<'a> {
    inserted: HashMap<*const Node, NodeIndex>,
    players: (PlayerCount, Finalized),
    start: Option<NodeIndex>,
    game: Graph<&'a Node, ()>,
    name: &'static str,
}

/* BUILDER IMPLEMENTATION */

impl<'a> SessionBuilder<'a> {
    /// Initialize a builder struct for a graph game with an empty graph, no
    /// starting state, and a given `name` that will be eventually used for
    /// the constructed game session's `id`.
    pub fn new(name: &'static str) -> Self {
        SessionBuilder {
            inserted: HashMap::new(),
            players: (0, false),
            start: None,
            game: Graph::new(),
            name,
        }
    }

    /// Create a new directed edge between nodes `from` and `to`. Fails if
    /// `from` is a terminal node, or if either `from` or `to` imply a player
    /// count that is incompatible with existing nodes.
    pub fn edge(mut self, from: &'a Node, to: &'a Node) -> Result<Self> {
        if let Node::Terminal(_) = from {
            Err(MockViolation {
                hint: format!(
                    "There was an attempt to add a terminal node on the \
                    outgoing side of an edge during the construction of the \
                    game '{}'.",
                    self.name,
                ),
            })?
        }

        self.update_player_count(from)?;
        self.update_player_count(to)?;

        let i = *self
            .inserted
            .entry(from as *const Node)
            .or_insert_with(|| self.game.add_node(from));

        let j = *self
            .inserted
            .entry(to as *const Node)
            .or_insert_with(|| self.game.add_node(to));

        self.game.update_edge(i, j, ());
        Ok(self)
    }

    /// Indicate that `node` is the starting state for the game being built. The
    /// indicated `node` (or a node with an identical hash) must have already
    /// been added to the game. Fails if there is no such existing node.
    pub fn start(mut self, node: &Node) -> Result<Self> {
        if let Some(index) = self
            .game
            .node_indices()
            .find(|&i| std::ptr::eq(self.game[i], node))
        {
            self.start = Some(index);
            Ok(self)
        } else {
            Err(MockViolation {
                hint: format!(
                    "There was an attempt to set the start state of mock game \
                    '{}', but the indicated start node has not been added to \
                    the game yet.",
                    self.name,
                ),
            })?
        }
    }

    /// Instantiate a `Session` encoding the constructed game graph. Fails if no
    /// starting state was specified, there exist non-terminal nodes with no
    /// outgoing edges, or no terminal nodes are reachable from the starting
    /// state (assuming it is valid).
    pub fn build(self) -> Result<Session<'a>> {
        let start = self.check_starting_state()?;
        self.check_terminal_state(start)?;
        self.check_outgoing_edges(start)?;
        let (players, _) = self.players;
        Ok(Session {
            players,
            start,
            game: self.game,
            name: self.name,
        })
    }

    /* HELPER METHODS */

    /// Infers and updates the player count of the game under construction based
    /// on the turn information or number of utility entries of `new`. Fails if
    /// adding `new` to the game would result in an invalid state.
    fn update_player_count(&mut self, new: &Node) -> Result<()> {
        let (old_count, finalized) = self.players;
        let new_count = match &new {
            Node::Terminal(vector) => {
                let result = vector.len();
                if result == 0 {
                    Err(MockViolation {
                        hint: format!(
                            "While constructing the game '{}', there was an \
                            attempt to add a terminal node with containing no \
                            utility entries. Games with no players are not \
                            allowed.",
                            self.name,
                        ),
                    })?
                };
                result
            },
            Node::Medial(turn) => turn + 1,
        };

        if finalized {
            if new.terminal() && old_count != new_count {
                Err(MockViolation {
                    hint: format!(
                        "While constructing the game '{}', a terminal node was \
                        added containing {} utility entries, but then a new \
                        one was added with {} entries. Utility entries must be \
                        consistent across all terminal nodes.",
                        self.name, old_count, new_count,
                    ),
                })?
            } else if new.medial() && new_count > old_count {
                Err(MockViolation {
                    hint: format!(
                        "While constructing the game '{}', a terminal node was \
                        added containing {} utility entries, but then a new \
                        medial node was added with a 0-indexed turn of {}, \
                        which is incompatible.",
                        self.name,
                        old_count,
                        new_count - 1,
                    ),
                })?
            }
        } else {
            if new.terminal() && new_count < old_count {
                Err(MockViolation {
                    hint: format!(
                        "While constructing the game '{}', a medial node was \
                        added with a 0-indexed turn of {}, but then a new \
                        terminal node was added with {} entries. All turn \
                        indicators must be able to index terminal nodes'\
                        utility entries.",
                        self.name,
                        old_count - 1,
                        new_count,
                    ),
                })?
            }
        }

        if new.terminal() {
            self.players = (new_count, true);
        } else if new.medial() && new_count > old_count {
            self.players = (new_count, false);
        }

        Ok(())
    }

    /// Fails if no starting state was specified for the constructed game.
    fn check_starting_state(&self) -> Result<NodeIndex> {
        if let Some(index) = self.start {
            Ok(index)
        } else {
            Err(MockViolation {
                hint: format!(
                    "No starting node was specified for the game '{}'.",
                    self.name,
                ),
            })?
        }
    }

    /// Fails if there does not exist a traversable path between `start` and any
    /// node marked as terminal in the game graph. Executes DFS from `start`
    /// until a terminal node is found.
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
                            .neighbors_directed(index, Direction::Outgoing)
                            .filter(|n| !seen.contains(n)),
                    );
                }
            }
        }

        Err(MockViolation {
            hint: format!(
                "No terminal node is reachable from the node marked as the \
                start in the game '{}'.",
                self.name
            ),
        })?
    }

    /// Fails if there exists a node marked as medial in the game graph which
    /// does not have any outgoing edges.
    fn check_outgoing_edges(&self, start: NodeIndex) -> Result<()> {
        println!(
            "{:?}",
            self.game
                .node_indices()
                .collect::<Vec<NodeIndex>>()
        );
        if self.game.node_indices().any(|i| {
            self.game[i].medial()
                && self
                    .game
                    .neighbors_directed(i, Direction::Outgoing)
                    .count()
                    .eq(&0)
        }) {
            Err(MockViolation {
                hint: format!(
                    "There exists a medial state with no outgoing edges in the \
                    constructed game '{}', which is a contradiction.",
                    self.name
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
        if let Node::Terminal(_) = self {
            true
        } else {
            false
        }
    }

    /// Returns true if and only if `self` is a medial node.
    #[inline]
    pub const fn medial(&self) -> bool {
        if let Node::Medial(_) = self {
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
    fn cannot_add_incorrect_utility_entries() -> Result<()> {
        let m1 = node!(0);
        let m2 = node!(2);
        let m3 = node!(0);

        let t1 = node!([1, 2]);
        let t2 = node!([3, 2, 1]);
        let t3 = node!([]);

        let game = SessionBuilder::new("bad utility 1")
            .edge(&m1, &t1)?
            .edge(&m1, &t2);

        assert!(game.is_err());

        let game = SessionBuilder::new("bad utility 2")
            .edge(&m1, &m2)?
            .edge(&m2, &t1);

        assert!(game.is_err());

        let game = SessionBuilder::new("bad utility 3")
            .edge(&m1, &m3)?
            .edge(&m3, &t3);

        assert!(game.is_err());

        Ok(())
    }

    #[test]
    fn cannot_add_incorrect_turn_information() -> Result<()> {
        let m1 = node!(0);
        let m2 = node!(2);
        let t1 = node!([1, -2]);
        let t2 = node!([-1, 2]);

        let game = SessionBuilder::new("bad turn")
            .edge(&m1, &t1)?
            .edge(&m1, &t2)?
            .edge(&m1, &m2);

        assert!(game.is_err());
        Ok(())
    }

    #[test]
    #[should_panic]
    fn cannot_add_outgoing_edge_to_terminal_node() {
        let m1 = node!(0);
        let m2 = node!(1);

        let t1 = node!([1, 2, 3, 4]);

        SessionBuilder::new("edge from terminal node")
            .edge(&t1, &m1).unwrap() // Panic
            .edge(&m1, &m2).unwrap()
            .edge(&t1, &m2).unwrap() // Panic
            .edge(&m2, &m1).unwrap()
            .edge(&t1, &m1).unwrap() // Panic
            .edge(&m2, &t1).unwrap()
            .edge(&t1, &m2).unwrap() // Panic
            .edge(&m1, &m2).unwrap();
    }

    #[test]
    fn cannot_build_graph_with_no_starting_state() -> Result<()> {
        let m1 = node!(0);
        let t1 = node!([1, 2]);

        let game1 = SessionBuilder::new("no starting state 1").build();
        let game2 = SessionBuilder::new("no starting state 2")
            .edge(&m1, &t1)?
            .build();

        assert!(game1.is_err());
        assert!(game2.is_err());
        Ok(())
    }

    #[test]
    fn cannot_build_game_with_no_accessible_end() -> Result<()> {
        let a = node!(2);
        let b = node!(1);
        let c = node!(0);
        let d = node!(1);

        let end = node!([1, 2, 3]);

        let game = SessionBuilder::new("no end")
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
        let a = node!(0);
        let b = node!(1);
        let c = node!(2);
        let d = node!(1);

        let trap = node!(0);
        let end = node!([1, 2, 3]);

        let game = SessionBuilder::new("trap game")
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
        let a = node!(0);
        let b = node!(1);
        let c = node!(0);
        let d = node!(1);
        let e = node!(0);
        let f = node!(1);

        let t1 = node!([1, 2]);
        let t2 = node!([2, 1]);

        let game = SessionBuilder::new("acyclic")
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

        let t1 = node!([1, 2, -1, 4]);
        let t2 = node!([2, 1, 9, -6]);

        let game = SessionBuilder::new("cyclic")
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

        assert_eq!(game.players, 4);

        Ok(())
    }
}
