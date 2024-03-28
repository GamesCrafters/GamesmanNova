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
use std::collections::HashMap;

use crate::collection;
use crate::game::Bounded;
use crate::game::Codec;
use crate::game::Forward;
use crate::game::GameData;
use crate::game::Information;
use crate::game::Transition;
use crate::game::Variable;
use crate::interface::IOMode;
use crate::interface::Solution;
use crate::model::game::State;
use crate::model::game::Variant;
use crate::model::solver::SUtility;
use crate::solver::ClassicPuzzle;

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
/*
enum Face {
    Up(Orientation),
    Down(Orientation),
    Left(Orientation),
    Right(Orientation),
    Front(Orientation),
    Back(Orientation),
    None,
}
*/

/// Encodes the orientation information about each piece in the game. Since each
/// piece is cube-like, it is not enough to just have a face, since a cube with
/// its "Front" face up could still be oriented in one of four ways.
/*
enum Orientation {
    North,
    East,
    South,
    West,
}
*/

/// Encodes the state of a single piece on the game board. It achieves this by
/// storing a value from 0-5 for each of 3 faces. This allows us to determine
/// the exact orientation out of 24 possible. We can think of each number from
/// 0-5 as being one of the 6 possible colors for cross faces. The faces are
/// arranged such that each pair of opposite faces always sum to 5. Example:
///       1
///       |
///   3 - 0 - 2     (5 on back)
///       |
///       4
/// All 24 orientations will be some rotation of this structure. The relative
/// positions of faces does not change.
struct Orientation {
    front: u64,
    top: u64,
    right: u64,
}

/// Maps a "packed" orientation to a number from 0-23 which will be used by the
/// hash function.
/// The format is front_top_right
/// Each of these will be a value from 0-5, and have 3 bits allotted.
const ORIENTATION_MAP: HashMap<u64, u64> = collection! {
    0b000_001_010 => 0,
    0b000_011_001 => 1,
    0b000_001_011 => 2,
    0b000_010_100 => 3,
    0b001_101_010 => 4,
    0b001_011_001 => 5,
    0b001_000_001 => 6,
    0b001_010_000 => 7,
    0b010_001_101 => 8,
    0b010_000_001 => 9,
    0b010_100_000 => 10,
    0b010_101_100 => 11,
    0b011_001_000 => 12,
    0b011_101_001 => 13,
    0b011_100_101 => 14,
    0b011_000_100 => 15,
    0b100_000_010 => 16,
    0b100_011_000 => 17,
    0b100_101_011 => 18,
    0b100_010_101 => 19,
    0b101_001_011 => 20,
    0b101_010_001 => 21,
    0b101_100_010 => 22,
    0b101_011_100 => 23,
};

/// Opposite of ORIENTATION_MAP:
/// maps numbers 0-23 to the corresponding packed orientation
const ORIENTATION_UNMAP: HashMap<u64, u64> = collection! {
    0  => 0b000_001_010,
    1  => 0b000_011_001,
    2  => 0b000_001_011,
    3  => 0b000_010_100,
    4  => 0b001_101_010,
    5  => 0b001_011_001,
    6  => 0b001_000_001,
    7  => 0b001_010_000,
    8  => 0b010_001_101,
    9  => 0b010_000_001,
    10 => 0b010_100_000,
    11 => 0b010_101_100,
    12 => 0b011_001_000,
    13 => 0b011_101_001,
    14 => 0b011_100_101,
    15 => 0b011_000_100,
    16 => 0b100_000_010,
    17 => 0b100_011_000,
    18 => 0b100_101_011,
    19 => 0b100_010_101,
    20 => 0b101_001_011,
    21 => 0b101_010_001,
    22 => 0b101_100_010,
    23 => 0b101_011_100,
};

/// Converts an Orientation struct into its corresponding orientation hash,
/// which will be a number from 0-23
/// Makes use of ORIENTATION_MAP for the conversion
fn hash_orientation(o: &Orientation) -> u64 {
    let mut packed: u64 = 0;
    packed = (o.front << 6) | (o.top << 3) | o.right;
    return ORIENTATION_MAP[&packed];
}

/// Converts an orientation hash into an Orientation struct
/// Makes use of ORIENTATION_UNMAP for the conversion
fn unhash_orientation(h: &u64) -> Orientation {
    let packed: u64 = ORIENTATION_UNMAP[h];
    const FACE_MASK: u64 = 0b111;
    return Orientation {
        front: packed >> 6 & FACE_MASK,
        top: packed >> 3 & FACE_MASK,
        right: packed & FACE_MASK,
    };
}

/// Simple, inefficient hash function that converts a vector of piece
/// orientations and an empty space represented by an integer into a 64 bit
/// integer (State) which uniquely represents that state.
fn hash(rep: &Vec<Orientation>, empty: u64) -> State {
    const BIT_SHIFT: u64 = 5;
    let mut s: State = empty;
    let mut shift: u64 = 3;
    for o in rep {
        s |= hash_orientation(o) << shift;
        shift += BIT_SHIFT;
    }
    return s;
}

/// Reverse of hash(), extracts the orientation vector and empty space from a
/// State.
fn unhash(s: State) -> (Vec<Orientation>, u64) {
    const PIECE_SHIFT: u64 = 5;
    const EMPTY_SHIFT: u64 = 3;
    const PIECES: u64 = 8;
    const PIECE_MASK: u64 = 0b11111;
    const EMPTY_MASK: u64 = 0b111;
    let mut s_tmp: u64 = s;
    let mut curr: u64;
    let empty: u64 = s & EMPTY_MASK;
    s_tmp >>= EMPTY_SHIFT;
    let mut rep: Vec<Orientation> = Vec::new();
    for i in 0..PIECES {
        curr = s_tmp & PIECE_MASK;
        rep.push(unhash_orientation(&curr));
        s_tmp >>= PIECE_SHIFT;
    }
    return (rep, empty);
}

/// Module which contains all transition helper functions
/// There are 4 functions for rotating an individual piece which represents a
/// shift in the given direction as defined by the structure of the crossteaser
/// game board.
/// There are 4 functions for shifting the entire board in a given direction,
/// which makes use of the above single piece functions, but updates the
/// relative position of pieces and the empty space on the game board.
mod mov {
    use crate::game::crossteaser::Orientation;

    /// Transforms an individual piece orientation as if it was shifted right
    fn right(o: &Orientation) -> Orientation {
        return Orientation {
            front: o.right,
            top: o.top,
            right: 5 - o.front,
        };
    }

    /// Transforms an individual piece orientation as if it was shifted left
    fn left(o: &Orientation) -> Orientation {
        return Orientation {
            front: 5 - o.right,
            top: o.top,
            right: o.front,
        };
    }

    /// Transforms an individual piece orientation as if it was shifted up
    fn up(o: &Orientation) -> Orientation {
        return Orientation {
            front: 5 - o.top,
            top: o.front,
            right: o.right,
        };
    }

    /// Transforms an individual piece orientation as if it was shifted down
    fn down(o: &Orientation) -> Orientation {
        return Orientation {
            front: o.top,
            top: 5 - o.front,
            right: o.right,
        };
    }

    fn board_right() {
        todo!()
    }

    fn board_left() {
        todo!()
    }

    fn board_up() {
        todo!()
    }

    fn board_down() {
        todo!()
    }
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
    fn solve(&self, mode: IOMode, method: Solution) -> Result<()> {
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

impl Transition for Session {
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

impl ClassicPuzzle for Session {
    fn utility(&self, state: State) -> SUtility {
        todo!()
    }
}
