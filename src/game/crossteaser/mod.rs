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
//! - Michael Setchko Palmerlee, 3/22/2024 (michaelsp@berkeley.edu)

use anyhow::{Context, Result};

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

/// Maps a number (index) from 0-23 to a "packed" 9-bit orientation
/// The format of the packed orientation is front_top_right
/// Each of these will be a value from 0-5, and have 3 bits allotted.
const ORIENTATION_MAP: [u64; 24] = [
    0b000_010_100, // 1
    0b000_100_011, // 2
    0b000_001_010, // 3
    0b000_011_001, // 4
    0b001_010_000, // 5
    0b001_000_011, // 6
    0b001_101_010, // 7
    0b001_011_101, // 8
    0b010_100_000, // 9
    0b010_000_001, // 10
    0b010_101_100, // 11
    0b010_001_101, // 12
    0b011_000_100, // 13
    0b011_100_101, // 14
    0b011_001_000, // 15
    0b011_101_001, // 16
    0b100_000_010, // 17
    0b100_010_101, // 18
    0b100_011_000, // 19
    0b100_101_011, // 20
    0b101_100_010, // 21
    0b101_010_001, // 22
    0b101_011_100, // 23
    0b101_001_011, // 24
];

/// Defines a sequence of axis-wise rotations which will return an orientation
/// to orientation 1 (index 0 in ORIENTATION_MAP).
/// An axis-wise rotation on axis x is a rotation on a piece along the axis
/// from face x to face 5 - x.
/// Format: each rotation is 5 bits of format abc_de
/// abc: axis on which the rotation is performed
/// de: direction of the rotation. 01 = cw, 11 = ccw.
/// Order of transformations is right to left.
/// Used for symmetries.
/// NOTE: This can likely be improved by combining transformations in a 
/// clever way.
const TRANSFORM_MAP: [u64; 24] = [
    0b0,                    // 1
    0b000_01,               // 2
    0b000_11,               // 3
    0b000_01_000_01,        // 4
    0b010_01,               // 5
    0b010_01_001_01,        // 6
    0b000_11_010_01,        // 7
    0b000_01_000_01_011_11, // 8
    0b010_01_000_01,        // 9
    0b010_01_001_01_000_01, // 10
    0b000_11_010_01_101_11, // 11
    0b000_11_001_11,        // 12
    0b100_11,               // 13
    0b100_11_011_01,        // 14
    0b100_11_010_11,        // 15
    0b000_01_000_01_001_01, // 16
    0b000_11_010_11,        // 17
    0b010_11,               // 18
    0b000_01_000_01_011_01, // 19
    0b000_01_011_01,        // 20
    0b100_11_011_01_100_11, // 21
    0b010_01_010_01,        // 22
    0b001_01_001_01,        // 23
    0b000_01_011_01_011_01, // 24
];

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
/// Uses patterns in the binary representations of an orientation to generate
/// a unique & minimal hash
/// ~1.85x faster than indexing into an array like unhash_orientation()
/// Maybe I can figure out an efficient reverse function at some point :)
fn hash_orientation(o: &Orientation) -> u64 {
    return (o.front << 2) | ((o.top & 1) << 1) | (o.right & 1);
}

/// Converts an orientation hash into an Orientation struct
/// Makes use of ORIENTATION_UNMAP for the conversion
fn unhash_orientation(h: u64) -> Orientation {
    let packed: u64 = ORIENTATION_MAP[h as usize];
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
            new_pieces.push(Orientation {
                front: o.front,
                top: o.top,
                right: o.right,
            });
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
    fn hash(&self, s: &UnhashedState) -> State {
        let mut new_s: State = s.free;
        let mut shift: u64 = EMPTY_BITS;
        for o in &s.pieces {
            new_s |= hash_orientation(o) << shift;
            shift += PIECE_BITS;
        }
        return new_s;
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
            rep.push(unhash_orientation(curr));
            s_tmp >>= PIECE_BITS;
        }
        return UnhashedState {
            pieces: rep,
            free: empty,
        };
    }

    /// Adjusts the entire board with a "right" move
    /// Adjusts empty space accordingly
    /// Makes use of mov::right()
    fn board_right(&self, s: &UnhashedState) -> UnhashedState {
        let mut new_state = s.deep_copy();
        if s.free % self.width != 0 {
            let to_move: usize = s.free as usize - 1;
            let dest: usize = s.free as usize;
            new_state.pieces[dest] = mov::right(&new_state.pieces[to_move]);
            new_state.free = to_move as u64;
        }
        return new_state;
    }

    /// Adjusts the entire board with a "left" move
    /// Adjusts empty space accordingly
    /// Makes use of mov::left()
    fn board_left(&self, s: &UnhashedState) -> UnhashedState {
        let mut new_state = s.deep_copy();
        if s.free % self.width != self.width - 1 {
            let to_move: usize = s.free as usize + 1;
            let dest: usize = s.free as usize;
            new_state.pieces[dest] = mov::left(&new_state.pieces[to_move]);
            new_state.free = to_move as u64;
        }
        return new_state;
    }

    /// Adjusts the entire board with a "up" move
    /// Adjusts empty space accordingly
    /// Makes use of mov::up()
    fn board_up(&self, s: &UnhashedState) -> UnhashedState {
        let mut new_state = s.deep_copy();
        if s.free / self.width != self.length - 1 {
            let to_move: usize = (s.free + self.width) as usize;
            let dest: usize = s.free as usize;
            new_state.pieces[dest] = mov::up(&new_state.pieces[to_move]);
            new_state.free = to_move as u64;
        }
        return new_state;
    }

    /// Adjusts the entire board with a "down" move
    /// Adjusts empty space accordingly
    /// Makes use of mov::down()
    fn board_down(&self, s: &UnhashedState) -> UnhashedState {
        let mut new_state = s.deep_copy();
        if s.free / self.width != 0 {
            let to_move: usize = (s.free - self.width) as usize;
            let dest: usize = s.free as usize;
            new_state.pieces[dest] = mov::down(&new_state.pieces[to_move]);
            new_state.free = to_move as u64;
        }
        return new_state;
    }

    fn r180(&self, s: &UnhashedState) -> UnhashedState {
        let mut new_s: UnhashedState = s.deep_copy();
        new_s.pieces.reverse();
        new_s.free = self.get_pieces() - new_s.free - 1;
        return new_s;
    }

    fn mirror(&self, s: &UnhashedState) -> UnhashedState {
        let mut new_s: UnhashedState = s.deep_copy();
        todo!()
    }

    fn apply_transformations(&self, o: &Orientation, t: u64) -> Orientation {
        let mut t_list: u64 = t;
        let mut transform: u64;
        let mut axis: u64;
        let mut new_o: Orientation = Orientation {
            front: o.front,
            top: o.top,
            right: o.right,
        };
        while t_list & 0b11 != 0 {
            transform = t_list & 0b11;
            t_list >>= 2;
            axis = t_list & 0b111;
            t_list >>= 3;
            if transform == 0b01 {
                new_o = mov::cw_on_axis(o, axis);
            } else if transform == 0b11 {
                new_o = mov::cw_on_axis(o, 5 - axis);
            }
        }
        return new_o;
    }

    fn canonical(&self, s: &UnhashedState) -> UnhashedState {
        let mut new_pieces: Vec<Orientation> = Vec::new();
        let pos: u64 = hash_orientation(&s.pieces[0]);
        let transform_list: u64 = TRANSFORM_MAP[pos as usize];
        for o in &s.pieces {
            new_pieces.push(self.apply_transformations(o, transform_list));
        }
        return UnhashedState {
            pieces: new_pieces,
            free: s.free,
        };
    }
 }

/// Module which contains all transition helper functions
/// There are 4 functions for rotating an individual piece which represents a
/// shift in the given direction as defined by the structure of the crossteaser
/// game board.
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

    /// Transforms an individual piece orientation as if it was rotated
    /// clockwise along the front-bottom axis. Used for symmetries
    pub fn cw_front(o: &Orientation) -> Orientation {
        return Orientation {
            front: o.front,
            top: 5 - o.right,
            right: o.top,
        };
    }

    /// Transforms an individual piece orientation as if it was rotated
    /// counter-clockwise along the front-back axis. Used for symmetries
    pub fn ccw_front(o: &Orientation) -> Orientation {
        return Orientation {
            front: o.front,
            top: o.right,
            right: 5 - o.top,
        };
    }

    /// Transforms an individual piece orientation as if it was rotated
    /// clockwise along the top-bottom axis. Used for symmetries
    pub fn cw_top(o: &Orientation) -> Orientation {
        return Orientation {
            front: o.right,
            top: o.top,
            right: 5 - o.front,
        };
    }

    /// Transforms an individual piece orientation as if it was rotated
    /// counter-clockwise along the top-bottom axis. Used for symmetries
    pub fn ccw_top(o: &Orientation) -> Orientation {
        return Orientation {
            front: 5 - o.right,
            top: o.top,
            right: o.front,
        };
    }

    /// Transforms an individual piece orientation as if it was rotated
    /// clockwise along the right-left. Used for symmetries
    pub fn cw_right(o: &Orientation) -> Orientation {
        return Orientation {
            front: 5 - o.top,
            top: o.front,
            right: o.right,
        };
    }

    /// Transforms an individual piece orientation as if it was rotated
    /// counter-clockwise along the right-left axis. Used for symmetries
    pub fn ccw_right(o: &Orientation) -> Orientation {
        return Orientation {
            front: o.top,
            top: 5 - o.front,
            right: o.right,
        };
    }

    /// Performs a clowckwise orientation on an orientation
    /// along the specified axis.
    /// I am trying to figure out a way to make this faster.
    pub fn cw_on_axis(o: &Orientation, axis: u64) -> Orientation {
        if axis == o.front {
            return cw_front(o);
        } else if axis == 5 - o.front {
            return ccw_front(o);
        } else if axis == o.top {
            return cw_top(o);
        } else if axis == 5 - o.top {
            return ccw_top(o);
        } else if axis == o.right {
            return cw_right(o);
        } else if axis == 5 - o.right {
            return ccw_right(o);
        } else {
            panic!("Invalid Orientation");
        }
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
        let s: UnhashedState = self.unhash(state);
        let mut states: Vec<State> = Vec::new();
        if s.free / self.width != self.length - 1 {
            states.push(self.hash(&self.canonical(&self.board_up(&s))));
        }
        if s.free / self.width != 0 {
            states.push(self.hash(&self.canonical(&self.board_down(&s))));
        }
        if s.free % self.width != 0 {
            states.push(self.hash(&self.canonical(&self.board_right(&s))));
        }
        if s.free % self.width != self.width - 1 {
            states.push(self.hash(&self.canonical(&self.board_left(&s))));
        }
        return states;
    }

    fn retrograde(&self, state: State) -> Vec<State> {
        return self.prograde(state);
    }
}

/* SUPPLEMENTAL IMPLEMENTATIONS */

impl Legible<State> for Session {
    fn decode(&self, string: String) -> Result<State> {
        todo!()
    }

    fn encode(&self, state: State) -> String {
        let s: UnhashedState = self.unhash(state);
        let mut v: Vec<Vec<String>> = Vec::new();
        for _i in 0..self.length {
            v.push(Vec::new());
        }
        let mut out: String = String::new();
        let mut row: usize;
        for (i, o) in s.pieces.iter().enumerate() {
            row = i / self.width as usize;
            if i as u64 == s.free {
                v[row].push("E".to_string());
            } else {
                v[row].push(hash_orientation(&o).to_string());
            }
        }
        for i in 0..self.length {
            out.push('|');
            out.push_str(&v[i as usize].join(" "));
        }
        out.push('|');
        return out;
    }
}

/* SOLVING IMPLEMENTATIONS */

impl Solvable<1> for Session {
    fn utility(&self, state: State) -> [Utility; 1] {
        if !self.end(state) {
            panic!("Cannot assess utility of non-terminal state!");
        } else {
            return [1];
        }
    }

    fn turn(&self, _state: crate::model::State) -> crate::model::Turn {
        return 0;
    }
}
