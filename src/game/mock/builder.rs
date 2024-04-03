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

use crate::game::error::GameError::MockViolation;
use crate::game::mock::Session;
use crate::game::mock::Stage;
use crate::model::{PlayerCount, State};

/* DEFINITIONS */

/// Used to identify whether the current player count is final.
type Final = bool;

/// Used to identify which state decided the current player count.
type Decider = State;

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
/// // Macro node initialization (equivalent)
/// let s0 = node!(0, 0);
/// let s1 = node!(1, 1);
/// let s2 = node!(2, [1, -1]);
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
    inserted: HashMap<State, NodeIndex>,
    players: (PlayerCount, Decider, Final),
    start: Option<NodeIndex>,
    game: Graph<&'a Node, ()>,
    name: &'static str,
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
    /// starting state, and a given `name` that will be eventually used for
    /// the constructed game session's `id`.
    pub fn new(name: &'static str) -> Self {
        SessionBuilder {
            inserted: HashMap::new(),
            players: (0, 0, false),
            start: None,
            game: Graph::new(),
            name,
        }
    }

    /// Insert an edge into the game graph under construction. If either of the
    /// nodes contains a previously inserted `hash` belonging to another node,
    /// the existing node will be used for the edge, and the additional fields
    /// in the new node will be ignored (unless they would cause failure).
    ///
    /// # Player count inference
    ///
    /// The player count of the game will be updated according to the turn or
    /// utility information in `from` and `to`. This is done such that on any
    /// successful call to build, the player count will be the number of utility
    /// entries in all successfully added terminal nodes.
    ///
    /// # Failure
    ///
    /// Fails if `from` is a terminal node, if a terminal node with a utility
    /// vector of incoherent size is added, or if a medial node with incoherent
    /// turn information is added.
    pub fn edge(mut self, from: &'a Node, to: &'a Node) -> Result<Self> {
        if let Stage::Terminal(_) = from.data {
            Err(MockViolation {
                hint: format!(
                    "There was an attempt to add a terminal node with hash {} \
                    on the outgoing side of an edge during the construction of \
                    the game '{}'.",
                    from.hash, self.name,
                ),
            })?
        }

        self.update_player_count(from)?;
        self.update_player_count(to)?;

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
    /// indicated `node` (or a node with an identical hash) must have already
    /// been added to the game. Fails if there is no such existing node.
    pub fn start(mut self, node: &Node) -> Result<Self> {
        if let Some(&index) = self.inserted.get(&node.hash) {
            self.start = Some(index);
            Ok(self)
        } else {
            Err(MockViolation {
                hint: format!(
                    "There was an attempt to set the start state of mock game \
                    '{}' to {}, but there is no existing node with that hash.",
                    self.name, node.hash,
                ),
            })?
        }
    }

    /// Instantiate a `Session` encoding the constructed game graph. Fails if no
    /// starting state was specified, the specified starting state does not
    /// correspond to any node `hash`, there exist non-terminal nodes with no
    /// outgoing edges, or no terminal nodes are reachable from the starting
    /// state (assuming it is valid).
    pub fn build(self) -> Result<Session<'a>> {
        let start = self.check_starting_state()?;
        self.check_terminal_state(start)?;
        self.check_outgoing_edges(start)?;
        let (players, _, _) = self.players;
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
        let (old_count, decider, finalized) = self.players;
        let new_count = match &new.data {
            Stage::Terminal(vector) => {
                let result = vector.len();
                if result == 0 {
                    Err(MockViolation {
                        hint: format!(
                            "While constructing the game '{}', there was an \
                            attempt to add a terminal node with hash '{}' \
                            containing no utility entries. Games with no \
                            players are not allowed.",
                            self.name, new.hash,
                        ),
                    })?
                };
                result
            },
            Stage::Medial(turn) => turn + 1,
        };

        if finalized {
            if new.terminal() && old_count != new_count {
                Err(MockViolation {
                    hint: format!(
                        "While constructing the game '{}', a terminal node \
                        with hash {} was added containing {} utility entries, \
                        but then a new one with hash {} was added with {} \
                        entries. Utility entries must be consistent across all \
                        terminal nodes.",
                        self.name, decider, old_count, new.hash, new_count,
                    ),
                })?
            } else if new.medial() && new_count > old_count {
                Err(MockViolation {
                    hint: format!(
                        "While constructing the game '{}', a terminal node \
                        with hash {} was added containing {} utility entries, \
                        but then a new medial node with hash {} was added with \
                        a 0-indexed turn of {}, which is incompatible.",
                        self.name,
                        decider,
                        old_count,
                        new.hash,
                        new_count - 1,
                    ),
                })?
            }
        } else {
            if new.terminal() && new_count < old_count {
                Err(MockViolation {
                    hint: format!(
                        "While constructing the game '{}', a medial node with \
                        hash {} was added with a 0-indexed turn of {}, but \
                        then a new terminal node with hash {} was added with \
                        {} entries. All turn indicators must be able to index \
                        terminal nodes' utility entries.",
                        self.name,
                        decider,
                        old_count - 1,
                        new.hash,
                        new_count,
                    ),
                })?
            }
        }

        if new.terminal() {
            self.players = (new_count, new.hash, true);
        } else if new.medial() && new_count > old_count {
            self.players = (new_count, new.hash, false);
        }

        Ok(())
    }

    /// Fails if no starting state was specified or if there exists no node in
    /// the game graph whose `hash` is equal to it.
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
                            .neighbors(index)
                            .filter(|n| !seen.contains(n)),
                    );
                }
            }
        }

        Err(MockViolation {
            hint: format!(
                "No terminal node is reachable from the starting node with \
                hash {} in the game '{}'.",
                self.game[start].hash, self.name
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
            Err(MockViolation {
                hint: format!(
                    "The medial state {} has no outgoing edges, which would \
                    present a contradiction in the game '{}'.",
                    self.game[index].hash, self.name
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
    fn cannot_add_incorrect_utility_entries() -> Result<()> {
        let m1 = node!(0, 0);
        let m2 = node!(1, 2);
        let m3 = node!(2, 0);

        let t1 = node!(3, [1, 2]);
        let t2 = node!(4, [3, 2, 1]);
        let t3 = node!(5, []);

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
        let m1 = node!(0, 0);
        let m2 = node!(1, 2);
        let t1 = node!(2, [1, -2]);
        let t2 = node!(3, [-1, 2]);

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
        let m1 = node!(0, 0);
        let m2 = node!(1, 1);

        let t1 = node!(3, [1, 2, 3, 4]);

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
        let m1 = node!(0, 0);
        let t1 = node!(1, [1, 2]);

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
        let a = node!(0, 2);
        let b = node!(1, 1);
        let c = node!(2, 0);
        let d = node!(3, 1);

        let end = node!(4, [1, 2, 3]);

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
        let a = node!(0, 0);
        let b = node!(1, 1);
        let c = node!(2, 2);
        let d = node!(3, 1);

        let trap = node!(4, 0);
        let end = node!(5, [1, 2, 3]);

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
        let a = node!(0, 0);
        let b = node!(1, 1);
        let c = node!(2, 0);
        let d = node!(3, 1);
        let e = node!(4, 0);
        let f = node!(5, 1);

        let t1 = node!(6, [1, 2]);
        let t2 = node!(7, [2, 1]);

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
        let a = node!(0, 0);
        let b = node!(1, 1);
        let c = node!(2, 0);
        let d = node!(3, 3);
        let e = node!(4, 0);
        let f = node!(5, 1);

        let t1 = node!(6, [1, 2, -1, 4]);
        let t2 = node!(7, [2, 1, 9, -6]);

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
