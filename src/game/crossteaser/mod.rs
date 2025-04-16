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

use anyhow::{Context, Result};

use crate::game::Codec;
use crate::game::Forward;
use crate::game::GameData;
use crate::game::Implicit;
use crate::game::Information;
use crate::game::State;
use crate::game::Variable;
use crate::game::Variant;
use crate::game::crossteaser::variants::*;
use crate::interface::IOMode;
use crate::solver::ClassicPuzzle;
use crate::solver::Game;
use crate::solver::SUtility;

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
    variant: String,
    length: u64,
    width: u64,
    free: u64,
}

impl Session {
    pub fn new(variant: Option<Variant>) -> Result<Self> {
        if let Some(v) = variant {
            Self::variant(v)
        } else {
            Ok(Self::default())
        }
    }

    pub fn solve(&self, mode: IOMode) -> Result<()> {
        todo!()
    }
}

/* INFORMATION IMPLEMENTATIONS */

impl Information for Session {
    fn info() -> GameData {
        todo!()
    }
}

/* VARIANCE IMPLEMENTATION */

impl Default for Session {
    fn default() -> Self {
        parse_variant(VARIANT_DEFAULT.to_owned())
            .expect("Failed to parse default game variant.")
    }
}

impl Variable for Session {
    fn variant(variant: Variant) -> Result<Self> {
        parse_variant(variant).context("Malformed game variant.")
    }

    fn variant_string(&self) -> Variant {
        self.variant.to_owned()
    }
}

/* TRAVERSAL IMPLEMENTATIONS */

impl Implicit for Session {
    fn adjacent(&self, state: State) -> Vec<State> {
        todo!()
    }

    fn source(&self) -> State {
        todo!()
    }

    fn sink(&self, state: State) -> bool {
        todo!()
    }
}

/* STATE RESOLUTION IMPLEMENTATIONS */

impl Codec for Session {
    fn decode(&self, string: String) -> Result<State> {
        todo!()
    }

    fn encode(&self, state: State) -> Result<String> {
        todo!()
    }
}

impl Forward for Session {
    fn set_verified_start(&mut self, state: State) {
        todo!()
    }
}

/* SOLVING IMPLEMENTATIONS */

impl Game<1> for Session {
    fn turn(&self, state: State) -> super::Player {
        todo!()
    }
}

impl ClassicPuzzle for Session {
    fn utility(&self, state: State) -> SUtility {
        todo!()
    }
}
