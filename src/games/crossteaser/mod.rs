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
//! - YOUR NAME HERE

use super::{AcyclicallySolvable, DynamicAutomaton, Game, GameData, Solvable};
use crate::{
    errors::NovaError,
    implement,
    interfaces::terminal::cli::IOMode,
    models::{State, Utility, Variant},
    solvers::acyclic::AcyclicSolver,
};
use nalgebra::{SMatrix, SVector};

/* SUBMODULES */

mod variants;

/* GAME DATA */

const NAME: &str = "crossteaser";
const AUTHORS: &str = "Max Fierro <maxfierro@berkeley.edu>, YOUR NAME";
const CATEGORY: &str = "Single-player puzzle";
const ABOUT: &str = "PLACEHOLDER";

/* GAME IMPLEMENTATION */

enum Face {
    Up,
    Down,
    Left,
    Right,
    Front,
    Back,
    None,
}

pub struct Session {
    variant: Option<String>,
    length: u32,
    width: u32,
    free: u64,
}

impl Game for Session {
    fn initialize(variant: Option<Variant>) -> Result<Self, NovaError> {
        todo!()
    }

    fn id(&self) -> String {
        todo!()
    }

    fn info(&self) -> GameData {
        todo!()
    }

    fn solve(&self, mode: Option<IOMode>) -> Result<(), NovaError> {
        <Self as AcyclicSolver<1>>::solve(self, mode);
        Ok(())
    }
}

impl DynamicAutomaton<State> for Session {
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
        todo!()
    }

    fn utility(&self, state: State) -> Option<SVector<Utility, 1>> {
        todo!()
    }

    fn coalesce(&self, state: State) -> SVector<Utility, 1> {
        todo!()
    }
}
