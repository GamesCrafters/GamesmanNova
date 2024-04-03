//! Mock Extensive Game Module
//!
//! This module provides a way to represent extensive-form games by declaring
//! the game via a graph, assigning special conditions to nodes, and exposing it
//! to multiple solving interfaces for convenience.
//!
//! #### Authorship
//!
//! - Max Fierro 3/31/2024 (maxfierro@berkeley.edu)

use anyhow::Result;
use petgraph::{graph::NodeIndex, Graph};

use crate::game::Bounded;
use crate::game::DTransition;
use crate::game::Game;
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

pub use builder::Node;
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
pub enum Stage {
    Terminal(Vec<Utility>),
    Medial(Turn),
}

/* GAME IMPLEMENTATION */

impl Game for Session<'_> {
    fn initialize(variant: Option<String>) -> Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn id(&self) -> String {
        todo!()
    }

    fn forward(&mut self, history: Vec<String>) -> Result<()> {
        todo!()
    }

    fn info(&self) -> super::GameData {
        todo!()
    }

    fn solve(&self, mode: IOMode, method: SolutionMode) -> Result<()> {
        todo!()
    }
}

/* TRAVERSAL IMPLEMENTATIONS */

impl Bounded<State> for Session<'_> {
    fn start(&self) -> State {
        todo!()
    }

    fn end(&self, state: State) -> bool {
        todo!()
    }
}

impl DTransition<State> for Session<'_> {
    fn prograde(&self, state: State) -> Vec<State> {
        todo!()
    }

    fn retrograde(&self, state: State) -> Vec<State> {
        todo!()
    }
}

impl STransition<State, MAX_TRANSITIONS> for Session<'_> {
    fn prograde(&self, state: State) -> [Option<State>; MAX_TRANSITIONS] {
        todo!()
    }

    fn retrograde(&self, state: State) -> [Option<State>; MAX_TRANSITIONS] {
        todo!()
    }
}

/* SUPPLEMENTAL IMPLEMENTATIONS */

impl Legible<State> for Session<'_> {
    fn decode(&self, string: String) -> Result<State> {
        todo!()
    }

    fn encode(&self, state: State) -> String {
        todo!()
    }
}

/* SOLVING IMPLEMENTATIONS */

impl<const N: PlayerCount> Solvable<N> for Session<'_> {
    fn utility(&self, state: State) -> [Utility; N] {
        todo!()
    }

    fn turn(&self, state: State) -> Turn {
        todo!()
    }
}
