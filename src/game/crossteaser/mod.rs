//! # Crossteaser Game Module
//!
//! The following definition is from this great website [1]: Crossteaser is a
//! puzzle which consists of a transparent frame enclosing 8 identical coloured
//! pieces arranged as if in a 3 by 3 grid. These pieces are three dimensional
//! crosses, each with six perpendicular arms of different colors. The frame
//! has three vertical slots at the front and three horizontal slots at the back
//! through which the front and back arms of the pieces stick. There is one
//! space in the 3 by 3 grid, and any adjacent piece can be moved into the
//! vacant space, but the slots in the frame force a piece to roll over as it
//! moves. By rolling the pieces around they get mixed up. The aim is to arrange
//! them so that the space is in the middle and that the crosses all have
//! matching orientation.
//!
//! [1]: https://www.jaapsch.net/puzzles/crosstsr.htm
//!
//! #### Authorship
//! - Max Fierro, 11/5/2023 (maxfierro@berkeley.edu)
//! - Cindy Xu, 11/28/2023

use anyhow::{Context, Result};

use crate::game::Bounded;
use crate::game::Codec;
use crate::game::DTransition;
use crate::game::Extensive;
use crate::game::Forward;
use crate::game::Game;
use crate::game::GameData;
use crate::game::GeneralSum;
use crate::interface::IOMode;
use crate::interface::SolutionMode;
use crate::model::SimpleUtility;
use crate::model::State;
use crate::model::Turn;
use crate::model::Utility;
use variants::*;

use super::ClassicPuzzle;
use super::SimpleSum;

/* SUBMODULES */

mod states;
mod variants;

/* GAME DATA */

const NAME: &str = "crossteaser";
const AUTHORS: &str = "Max Fierro <maxfierro@berkeley.edu>";
const CATEGORY: &str = "Single-player puzzle";
const ABOUT: &str = "PLACEHOLDER";

/* GAME IMPLEMENTATION */

/// Encodes the state of a piece in the game board. For reference, a cube has
/// six faces (up, down, etc.), and a cube with face A on top can be oriented
/// in one of four ways (north, south, etc.).
enum Face {
    Up(Orientation),
    Down(Orientation),
    Left(Orientation),
    Right(Orientation),
    Front(Orientation),
    Back(Orientation),
    None,
}

/// Encodes the orientation information about each piece in the game. Since each
/// piece is cube-like, it is not enough to just have a face, since a cube with
/// its "Front" face up could still be oriented in one of four ways.
enum Orientation {
    North,
    East,
    South,
    West,
}

/// Represents an instance of a Crossteaser game session, which is specific to
/// a valid variant of the game.
pub struct Session {
    variant: Option<String>,
    length: u64,
    width: u64,
    free: u64,
}

impl Game for Session {
    fn new(variant: Option<String>) -> Result<Self> {
        if let Some(v) = variant {
            parse_variant(v).context("Malformed game variant.")
        } else {
            Ok(parse_variant(VARIANT_DEFAULT.to_owned()).unwrap())
        }
    }

    fn id(&self) -> String {
        if let Some(variant) = self.variant.clone() {
            format!("{}.{}", NAME, variant)
        } else {
            NAME.to_owned()
        }
    }

    fn info(&self) -> GameData {
        todo!()
    }

    fn solve(&self, mode: IOMode, method: SolutionMode) -> Result<()> {
        todo!()
    }
}

/* TRAVERSAL IMPLEMENTATIONS */

impl DTransition for Session {
    fn prograde(&self, state: State) -> Vec<State> {
        todo!()
    }

    fn retrograde(&self, state: State) -> Vec<State> {
        todo!()
    }
}

/* STATE RESOLUTION IMPLEMENTATIONS */

impl Bounded for Session {
    fn start(&self) -> State {
        todo!()
    }

    fn end(&self, state: State) -> bool {
        todo!()
    }
}

impl Codec for Session {
    fn decode(&self, string: String) -> Result<State> {
        todo!()
    }

    fn encode(&self, state: State) -> String {
        todo!()
    }
}

impl Forward for Session {
    fn forward(&mut self, history: Vec<String>) -> Result<()> {
        todo!()
    }
}

/* SOLVING IMPLEMENTATIONS */

impl ClassicPuzzle for Session {
    fn utility(&self, state: State) -> SimpleUtility {
        todo!()
    }
}
