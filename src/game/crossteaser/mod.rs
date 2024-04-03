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

use anyhow::{Context, Result};
use std::collections::HashMap;

use crate::collection;
use crate::game::Bounded;
use crate::game::DTransition;
use crate::game::Game;
use crate::game::GameData;
use crate::game::Legible;
use crate::game::Solvable;
use crate::interface::IOMode;
use crate::interface::SolutionMode;
use crate::model::State;
use crate::model::Utility;
use variants::*;

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

/// Unhashed representation of a game state
/// It is simply a vector of (# pieces) Orientations
/// and an integer representing the location of the free space
struct UnhashedState {
    pieces: Vec<Orientation>,
    free: u64,
}

/// Maps a "packed" orientation to a number from 0-23 which will be used by the
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


// Constant bitmask values that will be commonly used for hashing/unhashing 
// game states.
const FACE_BITS: u64 = 3;
const FACE_BITMASK: u64 = 0b111;
const EMPTY_BITS: u64 = 3;
const EMPTY_BITMASK: u64 = 0b111;
const PIECE_BITS: u64 = 5;
const PIECE_BITMASK: u64 = 0b11111;


/// Converts an Orientation struct into its corresponding orientation hash,
/// which will be a number from 0-23
/// Makes use of ORIENTATION_MAP for the conversion
fn hash_orientation(o: &Orientation) -> u64 {
    let packed: u64 = (o.front << (FACE_BITS * 2)) | 
                      (o.top << FACE_BITS) | 
                      o.right;
    return ORIENTATION_MAP[&packed];
}

/// Converts an orientation hash into an Orientation struct
/// Makes use of ORIENTATION_UNMAP for the conversion
fn unhash_orientation(h: &u64) -> Orientation {
    let packed: u64 = ORIENTATION_UNMAP[h];
    return Orientation {
        front: packed >> (FACE_BITS * 2) & FACE_BITMASK,
        top: packed >> FACE_BITS & FACE_BITMASK,
        right: packed & FACE_BITMASK,
    };
}

impl UnhashedState {
    fn deep_copy(&self) -> UnhashedState {
        let mut new_pieces: Vec<Orientation> = Vec::new();
        for o in &self.pieces {
            new_pieces.push(
                Orientation {
                    front: o.front,
                    top: o.top,
                    right: o.right,
                }
            );
        }
        return UnhashedState {
            pieces: new_pieces,
            free: self.free,
        };
    }
}

impl Session {
    /// Returns the total number of game pieces in the current game variant
    /// based on Session
    fn get_pieces(&self) -> u64 {
        return self.length * self.width - self.free;
    }

    /// Simple, inefficient hash function that converts a vector of piece
    /// orientations and an empty space represented by an integer into a 64 bit
    /// integer (State) which uniquely represents that state.
    fn hash(&self, rep: &Vec<Orientation>, empty: u64) -> State {
        let mut s: State = empty;
        let mut shift: u64 = EMPTY_BITS;
        for o in rep {
            s |= hash_orientation(o) << shift;
            shift += PIECE_BITS;
        }
        return s;
    }

    /// Reverse of hash(), extracts the orientation vector and empty space from
    /// a State.
    fn unhash(&self, s: State) -> UnhashedState {
        let num_pieces: u64 = self.get_pieces();
        let mut s_tmp: u64 = s;
        let mut curr: u64;
        let empty: u64 = s & EMPTY_BITMASK;
        s_tmp >>= EMPTY_BITS;
        let mut rep: Vec<Orientation> = Vec::new();
        for i in 0..num_pieces {
            curr = s_tmp & PIECE_BITMASK;
            rep.push(unhash_orientation(&curr));
            s_tmp >>= PIECE_BITS;
        }
        return UnhashedState {
            pieces: rep,
            free: empty,
        };
    }

    fn board_right(&self, s: UnhashedState) -> UnhashedState {
        let mut new_state = s.deep_copy();
        if s.free % self.width != 0 {
            let to_move: usize = s.free as usize - 1;
            new_state.pieces[to_move] = mov::right(&new_state.pieces[to_move]);
            new_state.free = to_move as u64;
        }
        return new_state;
    }

    fn board_left(&self, s: UnhashedState) -> UnhashedState {
        let mut new_state = s.deep_copy();
        if s.free % self.width != self.width - 1 {
            let to_move: usize = s.free as usize + 1;
            new_state.pieces[to_move] = mov::left(&new_state.pieces[to_move]);
            new_state.free = to_move as u64;
        }
        return new_state;
    }

    fn board_up(&self, s: UnhashedState) -> UnhashedState {
        let mut new_state = s.deep_copy();
        if s.free / self.width != self.length - 1 {
            let to_move: usize = (s.free + self.width) as usize;
            new_state.pieces[to_move] = mov::up(&new_state.pieces[to_move]);
            new_state.free = to_move as u64;
        }
        return new_state;
    }

    fn board_down(&self, s: UnhashedState) -> UnhashedState {
        let mut new_state = s.deep_copy();
        if s.free / self.width != 0 {
            let to_move: usize = (s.free - self.width) as usize;
            new_state.pieces[to_move] = mov::down(&new_state.pieces[to_move]);
            new_state.free = to_move as u64;
        }
        return new_state;
    }
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
    pub fn right(o: &Orientation) -> Orientation {
        return Orientation {
            front: o.right,
            top: o.top,
            right: 5 - o.front,
        };
    }

    /// Transforms an individual piece orientation as if it was shifted left
    pub fn left(o: &Orientation) -> Orientation {
        return Orientation {
            front: 5 - o.right,
            top: o.top,
            right: o.front,
        };
    }

    /// Transforms an individual piece orientation as if it was shifted up
    pub fn up(o: &Orientation) -> Orientation {
        return Orientation {
            front: 5 - o.top,
            top: o.front,
            right: o.right,
        };
    }

    /// Transforms an individual piece orientation as if it was shifted down
    pub fn down(o: &Orientation) -> Orientation {
        return Orientation {
            front: o.top,
            top: 5 - o.front,
            right: o.right,
        };
    }
}

/// Represents an instance of a Crossteaser game session, which is specific to
/// a valid variant of the game.
/// length is number of rows
/// width is number of columns
pub struct Session {
    variant: Option<String>,
    length: u64,
    width: u64,
    free: u64,
}

impl Game for Session {
    fn initialize(variant: Option<String>) -> Result<Self> {
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

    fn forward(&mut self, history: Vec<String>) -> Result<()> {
        todo!()
    }
}

/* TRAVERSAL IMPLEMENTATIONS */

impl Bounded<State> for Session {
    fn start(&self) -> State {
        todo!()
    }

    fn end(&self, state: State) -> bool {
        todo!()
    }
}

impl DTransition<State> for Session {
    fn prograde(&self, state: State) -> Vec<State> {
        todo!()
    }

    fn retrograde(&self, state: State) -> Vec<State> {
        todo!()
    }
}

/* SUPPLEMENTAL IMPLEMENTATIONS */

impl Legible<State> for Session {
    fn decode(&self, string: String) -> Result<State> {
        todo!()
    }

    fn encode(&self, state: State) -> String {
        todo!()
    }
}

/* SOLVING IMPLEMENTATIONS */

impl Solvable<1> for Session {
    fn utility(&self, state: State) -> [Utility; 1] {
        todo!()
    }

    fn turn(&self, state: crate::model::State) -> crate::model::Turn {
        todo!()
    }
}
