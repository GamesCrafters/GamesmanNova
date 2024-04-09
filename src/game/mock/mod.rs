//! Mock Extensive Game Module
//!
//! This module provides a way to represent extensive-form games by declaring
//! the game via a graph and assigning special conditions to nodes. This makes
//! creating example games a matter of simply declaring them and wrapping them
//! in any necessary external interface implementations.
//!
//! #### Authorship
//!
//! - Max Fierro 3/31/2024 (maxfierro@berkeley.edu)

use anyhow::{Context, Result};
use petgraph::dot::{Config, Dot};
use petgraph::Direction;
use petgraph::{graph::NodeIndex, Graph};

use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

use crate::game::Bounded;
use crate::game::DTransition;
use crate::game::STransition;
use crate::model::PlayerCount;
use crate::model::State;
use crate::model::Turn;
use crate::model::Utility;
use crate::solver::MAX_TRANSITIONS;

/* RE-EXPORTS */

pub use builder::SessionBuilder;

/* SUBMODULES */

mod example;
mod builder;

/* DEFINITIONS */

/// Represents an initialized session of an abstract graph game. This can be
/// constructed using `SessionBuilder`.
pub struct Session<'a> {
    inserted: HashMap<*const Node, NodeIndex>,
    players: PlayerCount,
    start: NodeIndex,
    game: Graph<&'a Node, ()>,
    name: &'static str,
}

/// Indicates whether a game state node is terminal (there are no outgoing moves
/// or edges) or medial (it is possible to transition out of it). Nodes in the
/// terminal stage have an associated utility vector, and medial nodes have a
/// turn encoding whose player's action is pending.
#[derive(Debug)]
pub enum Node {
    Terminal(Vec<Utility>),
    Medial(Turn),
}

/* IMPLEMENTATION */

impl<'a> Session<'a> {
    /// Return a name or identifier corresponding to this game.
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Return the number of players in this game.
    pub fn players(&self) -> PlayerCount {
        self.players
    }

    /// Return the state hash being internally used for `node`.
    pub fn state(&self, node: &Node) -> Option<State> {
        if let Some(index) = self
            .inserted
            .get(&(node as *const Node))
        {
            Some(index.index() as State)
        } else {
            None
        }
    }

    /// Sends an SVG of the game graph to STDOUT. Requires an installation of
    /// graphviz 'dot', failing if the 'dot' command is not in the user PATH.
    pub fn image(&self) -> Result<()> {
        let mut dot = Command::new("dot")
            .arg("-Tsvg")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to execute 'dot' command.")?;

        if let Some(mut stdin) = dot.stdin.take() {
            let graph = format!("{}", self);
            stdin.write_all(graph.as_bytes())?;
        }

        if let Some(stdout) = dot.stdout.take() {
            let mut reader = io::BufReader::new(stdout);
            let mut output = String::new();
            reader.read_to_string(&mut output)?;
            print!("{}", output);
        }

        dot.wait()?;
        Ok(())
    }

    /* HELPER METHODS */

    /// Return the states adjacent to `state`, where `dir` specifies whether
    /// they should be connected by incoming or outgoing edges.
    fn transition(&self, state: State, dir: Direction) -> Vec<State> {
        self.game
            .neighbors_directed(NodeIndex::from(state as u32), dir)
            .map(|n| n.index() as State)
            .collect()
    }

    /// Returns a reference to the game node with `state`, or panics if there is
    /// no such node.
    fn node(&self, state: State) -> &Node {
        self.game[NodeIndex::from(state as u32)]
    }
}

/* UTILITY IMPLEMENTATIONS */

impl DTransition<State> for Session<'_> {
    fn prograde(&self, state: State) -> Vec<State> {
        self.transition(state, Direction::Outgoing)
    }

    fn retrograde(&self, state: State) -> Vec<State> {
        self.transition(state, Direction::Incoming)
    }
}

impl STransition<State, MAX_TRANSITIONS> for Session<'_> {
    fn prograde(&self, state: State) -> [Option<State>; MAX_TRANSITIONS] {
        let adjacent = self
            .transition(state, Direction::Outgoing)
            .iter()
            .map(|&h| Some(h))
            .collect::<Vec<Option<State>>>();

        if adjacent.len() > MAX_TRANSITIONS {
            panic!("Exceeded maximum transition count.")
        }

        let mut result = [None; MAX_TRANSITIONS];
        result.copy_from_slice(&adjacent[..MAX_TRANSITIONS]);
        result
    }

    fn retrograde(&self, state: State) -> [Option<State>; MAX_TRANSITIONS] {
        let adjacent = self
            .transition(state, Direction::Incoming)
            .iter()
            .map(|&h| Some(h))
            .collect::<Vec<Option<State>>>();

        if adjacent.len() > MAX_TRANSITIONS {
            panic!("Exceeded maximum transition count.")
        }

        let mut result = [None; MAX_TRANSITIONS];
        result.copy_from_slice(&adjacent[..MAX_TRANSITIONS]);
        result
    }
}

impl Bounded<State> for Session<'_> {
    fn start(&self) -> State {
        self.start.index() as State
    }

    fn end(&self, state: State) -> bool {
        match self.node(state) {
            Node::Terminal(_) => true,
            Node::Medial(_) => false,
        }
    }
}

impl Display for Session<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            Dot::with_attr_getters(
                &self.game,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, _| String::new(),
                &|_, n| {
                    let (_, node) = n;
                    let mut attrs = String::new();
                    match node {
                        Node::Medial(turn) => {
                            attrs += &format!("label=P{} ", turn);
                            attrs += "style=filled  ";
                            if self.start() == self.state(node).unwrap() {
                                attrs += "shape=doublecircle ";
                                attrs += "fillcolor=navajowhite3 ";
                            } else {
                                attrs += "shape=circle ";
                                attrs += "fillcolor=lightsteelblue ";
                            }
                        },
                        Node::Terminal(util) => {
                            attrs += &format!("label=\"{:?}\" ", util);
                            attrs += "shape=plain ";
                        },
                    }
                    attrs
                }
            )
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::node;
    use anyhow::Result;

    #[test]
    fn get_unique_node_states() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(2);
        let s4 = node!(1);
        let s5 = node!(0);

        let t1 = node![1, 2, 3];
        let t2 = node![3, 2, 1];

        let g = SessionBuilder::new("sample")
            .edge(&s1, &s2)?
            .edge(&s2, &s3)?
            .edge(&s3, &s4)?
            .edge(&s4, &s5)?
            .edge(&s4, &t1)?
            .edge(&s5, &t2)?
            .start(&s1)?
            .build()?;

        let states = vec![
            g.state(&s1),
            g.state(&s2),
            g.state(&s3),
            g.state(&s4),
            g.state(&s5),
            g.state(&t1),
            g.state(&t2),
        ];

        let contains_none = states.iter().any(Option::is_none);
        assert!(!contains_none);

        let states: Vec<State> = states
            .iter()
            .map(|s| s.unwrap())
            .collect();

        let repeats = states.iter().any(|&i| {
            states[(1 + i as usize)..]
                .iter()
                .any(|&j| i == j)
        });

        assert!(!repeats);
        Ok(())
    }

    #[test]
    fn verify_start_and_end_states() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(2);

        let t1 = node![1, 2, 3];
        let t2 = node![3, 2, 1];

        let g = SessionBuilder::new("sample")
            .edge(&s1, &s2)?
            .edge(&s2, &s3)?
            .edge(&s2, &t1)?
            .edge(&s3, &t2)?
            .start(&s1)?
            .build()?;

        let start = g.state(&s1).unwrap();
        let end1 = g.state(&t1).unwrap();
        let end2 = g.state(&t2).unwrap();

        assert_eq!(g.start(), start);
        assert!(g.end(end1));
        assert!(g.end(end2));
        Ok(())
    }

    #[test]
    fn verify_state_transition() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let s3 = node!(2);

        let t1 = node![1, 2, 3];
        let t2 = node![3, 2, 1];

        let g = SessionBuilder::new("sample")
            .edge(&s1, &s2)?
            .edge(&s1, &s3)?
            .edge(&s2, &t1)?
            .edge(&s3, &t2)?
            .start(&s1)?
            .build()?;

        let s1_state = g.state(&s1).unwrap();
        let s2_state = g.state(&s2).unwrap();
        let s3_state = g.state(&s3).unwrap();

        let t1_state = g.state(&t1).unwrap();
        let t2_state = g.state(&t2).unwrap();

        let s1_pro = g.transition(s1_state, Direction::Outgoing);
        let s2_pro = g.transition(s2_state, Direction::Outgoing);
        let t2_ret = g.transition(t2_state, Direction::Incoming);

        assert!(s1_pro.len() == 2);
        assert!(s2_pro.len() == 1);
        assert!(t2_ret.len() == 1);

        assert!(s1_pro.contains(&s3_state));
        assert!(s1_pro.contains(&s2_state));

        assert!(s2_pro.contains(&t1_state));
        assert!(t2_ret.contains(&s3_state));

        Ok(())
    }

    #[test]
    fn get_game_name() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(1);
        let t1 = node![-1, 2];
        let g = SessionBuilder::new("gaming")
            .edge(&s1, &s2)?
            .edge(&s2, &t1)?
            .start(&s1)?
            .build()?;

        assert_eq!(g.name(), "gaming");
        Ok(())
    }

    #[test]
    fn get_player_count() -> Result<()> {
        let s1 = node!(0);
        let s2 = node!(5);
        let t1 = node![1, -2, 3, -4, 5, -6, 7];
        let g = SessionBuilder::new("7 player game")
            .edge(&s1, &s2)?
            .edge(&s2, &t1)?
            .start(&s1)?
            .build()?;

        assert_eq!(g.players(), 7);
        Ok(())
    }
}
