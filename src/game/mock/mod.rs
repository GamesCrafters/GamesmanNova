//! Mock Extensive Game Module
//!
//! This module provides a way to represent extensive-form games by declaring
//! the game via a graph, assigning special conditions to nodes, and exposing it
//! to multiple solving interfaces for convenience.
//!
//! #### Authorship
//!
//! - Max Fierro 3/31/2024 (maxfierro@berkeley.edu)

use anyhow::{Context, Result};
use petgraph::Direction;
use petgraph::{graph::NodeIndex, Graph};

use crate::game::error::GameError::MockViolation;
use crate::game::util;
use crate::game::Bounded;
use crate::game::DTransition;
use crate::game::Game;
use crate::game::GameData;
use crate::game::Legible;
use crate::game::STransition;
use crate::game::Solvable;
use crate::interface::IOMode;
use crate::interface::SolutionMode;
use crate::model::PlayerCount;
use crate::model::State;
use crate::model::Turn;
use crate::model::Utility;
use crate::solver::MAX_TRANSITIONS;

/* RE-EXPORTS */

pub use builder::SessionBuilder;

/* SUBMODULES */

mod state;
mod builder;

/* GAME DATA */

const NAME: &'static str = "mock";
const AUTHORS: &'static str = "Max Fierro <maxfierro@berkeley.edu>";
const ABOUT: &'static str = "PLACEHOLDER";

const VARIANT_DEFAULT: &'static str = "N/A";
const VARIANT_PATTERN: &'static str = "N/A";
const VARIANT_PROTOCOL: &'static str =
"This implementation has no variants, as it can represent any extensive game.";

/* DEFINITIONS */

/// Represents an initialized session of an abstract graph game. This can be
/// constructed using `SessionBuilder`. Due to this, variants in this game are
/// meaningless, since `SessionBuilder` can achieve any game structure.
pub struct Session<'a> {
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

/* GAME IMPLEMENTATION */

impl Game for Session<'_> {
    fn initialize(_variant: Option<String>) -> Result<Self>
    where
        Self: Sized,
    {
        Err(MockViolation {
            hint:
                "Conventional initialization is not supported for mock games."
                    .into(),
        })?
    }

    fn id(&self) -> String {
        self.name.into()
    }

    fn forward(&mut self, history: Vec<String>) -> Result<()> {
        let state = util::verify_history_dynamic(self, history)
            .context("Malformed game state encoding.")?;

        self.get_node(state);

        Ok(())
    }

    fn info(&self) -> GameData {
        GameData {
            variant: VARIANT_DEFAULT.into(),
            name: NAME,
            authors: AUTHORS,
            about: ABOUT,

            variant_protocol: VARIANT_PROTOCOL,
            variant_pattern: VARIANT_PATTERN,
            variant_default: VARIANT_DEFAULT,

            state_protocol: state::STATE_PROTOCOL,
            state_pattern: state::STATE_PATTERN,
            state_default: state::STATE_DEFAULT,
        }
    }

    fn solve(&self, _mode: IOMode, _method: SolutionMode) -> Result<()> {
        Err(MockViolation {
            hint: "Conventional solving is not supported for mock games."
                .into(),
        })?
    }
}

/* STATE CODEC IMPLEMENTATION */

impl Legible<State> for Session<'_> {
    fn decode(&self, string: String) -> Result<State> {
        todo!()
    }

    fn encode(&self, state: State) -> String {
        todo!()
    }
}

/* TRAVERSAL IMPLEMENTATIONS */

impl Bounded<State> for Session<'_> {
    fn start(&self) -> State {
        self.start.index() as State
    }

    fn end(&self, state: State) -> bool {
        match self.get_node(state) {
            Node::Terminal(_) => true,
            Node::Medial(_) => false,
        }
    }
}

impl DTransition<State> for Session<'_> {
    fn prograde(&self, state: State) -> Vec<State> {
        self.get_adjacent(state, Direction::Outgoing)
    }

    fn retrograde(&self, state: State) -> Vec<State> {
        self.get_adjacent(state, Direction::Incoming)
    }
}

impl STransition<State, MAX_TRANSITIONS> for Session<'_> {
    fn prograde(&self, state: State) -> [Option<State>; MAX_TRANSITIONS] {
        let adjacent = self
            .get_adjacent(state, Direction::Outgoing)
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
            .get_adjacent(state, Direction::Incoming)
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

/* SOLVING IMPLEMENTATIONS */

/// Implements the `Solvable<N>` trait for different player counts.
macro_rules! solvable_for {
    ($($N:expr),*) => {
        $(impl Solvable<$N> for Session<'_> {
            fn utility(&self, state: State) -> [Utility; $N] {
                let from = match &self.get_node(state) {
                    Node::Terminal(vector) => vector,
                    Node::Medial(_) => panic!(
                        "Attempted to get utility of medial node."
                    ),
                };

                let mut result = [0; $N];
                result.copy_from_slice(&from[..$N]);
                result
            }

            fn turn(&self, state: State) -> Turn {
                match self.get_node(state) {
                    Node::Medial(turn) => *turn,
                    Node::Terminal(_) => panic!(
                        "Attempted to get turn of terminal node."
                    ),
                }
            }
        })*
    };
}

solvable_for![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

/* HELPER IMPLEMENTATIONS */

impl Session<'_> {
    /// Return the states adjacent to `state`, where `dir` specifies whether
    /// they should be connected by incoming or outgoing edges.
    fn get_adjacent(&self, state: State, dir: Direction) -> Vec<State> {
        self.game
            .neighbors_directed(NodeIndex::from(state as u32), dir)
            .map(|n| n.index() as State)
            .collect()
    }

    /// Returns a reference to the game node with `state`, or panics if there is
    /// no such node.
    fn get_node(&self, state: State) -> &Node {
        self.game[NodeIndex::from(state as u32)]
    }
}
