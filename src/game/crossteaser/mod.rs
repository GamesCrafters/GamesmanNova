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
//!
//! - Max Fierro, 11/5/2023 (maxfierro@berkeley.edu)
//! - Cindy Xu, 11/28/2023

use super::{AcyclicallySolvable, Automaton, Game, GameData, Solvable};
use crate::{
    errors::NovaError,
    implement,
    interfaces::terminal::cli::IOMode,
    models::{State, Utility, Variant},
    solvers::acyclic::AcyclicSolver,
};
use nalgebra::{SMatrix, SVector};
use variants::*;

/* SUBMODULES */

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
    fn initialize(variant: Option<Variant>) -> Result<Self, NovaError> {
        if let Some(v) = variant {
            parse_variant(v)
        } else {
            parse_variant(VARIANT_DEFAULT.to_owned())
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
        GameData {
            name: NAME.to_owned(),
            authors: AUTHORS.to_owned(),
            about: ABOUT.to_owned(),
            category: CATEGORY.to_owned(),
            variant_protocol: VARIANT_PROTOCOL.to_owned(),
            variant_pattern: VARIANT_PATTERN.to_owned(),
            variant_default: VARIANT_DEFAULT.to_owned(),
        }
    }

    fn solve(&self, mode: Option<IOMode>) -> Result<(), NovaError> {
        <Self as AcyclicSolver<1>>::solve(self, mode);
        Ok(())
    }
}

impl Automaton<State> for Session {
    fn start(&self) -> State {
        todo!()
    }

    fn transition(&self, state: State) -> Vec<State> {
        todo!()
    }

    fn accepts(&self, state: State) -> bool {
        todo!()
    }
}

/* SOLVABLE DECLARATIONS */

implement! { for Session =>
    AcyclicallySolvable<1>
}

impl Solvable<1> for Session {
    fn weights(&self) -> SMatrix<Utility, 1, 1> {
        SMatrix::<Utility, 1, 1>::identity()
    }

    fn utility(&self, state: State) -> Option<SVector<Utility, 1>> {
        if !self.accepts(state) {
            None
        } else {
            Some(SVector::<Utility, 1>::from_element(1))
        }
    }

    fn coalesce(&self, state: State) -> SVector<Utility, 1> {
        SVector::<Utility, 1>::from_element(1)
    }
}
