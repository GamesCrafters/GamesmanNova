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
//! - Michael Setchko Palmerlee, 3/22/2024 (michaelsp@berkeley.edu)

use anyhow::{Context, Result};
use bitvec::prelude::{BitArray, Msb0};
use bitvec::view::BitView;
use bitvec::{bitarr, bitvec};

use crate::game::Bounded;
use crate::game::ClassicPuzzle;
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
#[cfg(test)]
mod test;

/* GAME DATA */

const NAME: &str = "crossteaser";
const AUTHORS: &str = "Max Fierro <maxfierro@berkeley.edu>, \
Michael Setchko Palmerlee <michaelsp@berkeley.edu>";
const CATEGORY: &str = "Single-player puzzle";
const ABOUT: &str = "Puzzle played on a 3x3 board which has 9 spaces. 8 of \
these are filled with pieces, one is empty. The pieces are three-dimensional \
crosses, as if a cube had its 6 faces extruded. Each of the 6 sections has a \
different color. The pieces bordering the empty space can be shifted to the \
empty space, but will rotate based on the restrictions of the board. Every \
time a piece is moved, its orientation changes. The goal is to arrange the \
pieces such that they all have identical orientation and the empty space is \
at the center.";

/* GAME IMPLEMENTATION */

/// Maps a number (index) from 0-23 to a "packed" 9-bit orientation
/// The format of the packed orientation is front_top_right
/// Each of these will be a value from 0-5, and have 3 bits allotted.
const ORIENTATION_MAP: [u16; 24] = [
    0b000_010_100, // 0
    0b000_100_011, // 1
    0b000_001_010, // 2
    0b000_011_001, // 3
    0b001_010_000, // 4
    0b001_000_011, // 5
    0b001_101_010, // 6
    0b001_011_101, // 7
    0b010_100_000, // 8
    0b010_000_001, // 9
    0b010_101_100, // 10
    0b010_001_101, // 11
    0b011_000_100, // 12
    0b011_100_101, // 13
    0b011_001_000, // 14
    0b011_101_001, // 15
    0b100_000_010, // 16
    0b100_010_101, // 17
    0b100_011_000, // 18
    0b100_101_011, // 19
    0b101_100_010, // 20
    0b101_010_001, // 21
    0b101_011_100, // 22
    0b101_001_011, // 23
];

/// Defines a sequence of axis-wise rotations which will return an orientation
/// to orientation 1 (index 0 in ORIENTATION_MAP).
/// An axis-wise rotation on axis x is a rotation on a piece along the axis
/// from face x to face 5 - x.
/// Format: each rotation is 5 bits of format abc_de.
/// abc: axis on which the rotation is performed.
/// de: direction of the rotation. 0b01 = cw, 0b10 = 180, 0b11 = ccw.
/// Order of transformations is right to left. Used for symmetries.
const TRANSFORM_MAP: [u64; 24] = [
    0b0,             // 0
    0b000_01,        // 1
    0b000_11,        // 2
    0b000_10,        // 3
    0b010_01,        // 4
    0b010_01_001_01, // 5
    0b000_11_010_01, // 6
    0b000_10_011_11, // 7
    0b010_01_000_01, // 8
    0b000_10_001_11, // 9
    0b000_10_001_11, // 10
    0b000_11_001_11, // 11
    0b100_11,        // 12
    0b100_11_011_01, // 13
    0b100_11_010_11, // 14
    0b000_10_001_01, // 15
    0b000_11_010_11, // 16
    0b010_11,        // 17
    0b000_10_011_01, // 18
    0b000_01_011_01, // 19
    0b100_10_101_01, // 20
    0b010_10,        // 21
    0b001_10,        // 22
    0b000_01_011_10, // 23
];

const FACE_BITS: u64 = 3;
const PIECE_SIZE: u64 = 24;

/// Encodes the state of a single piece on the game board. It achieves this by
/// storing a value from 0-5 for each of 3 faces. This allows us to determine
/// the exact orientation out of 24 possible. We can think of each number from
/// 0-5 as being one of the 6 possible colors for cross faces. The faces are
/// arranged such that each pair of opposite faces always sum to 5. Example:
/// ```
///       2
///       |
///   1 - 0 - 4     (5 on back)
///       |
///       3
/// ```
/// The above orientation will be defined as the "default" state
/// and is at index 0 in the ORIENTATION_MAP array.
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

/// Represents an instance of a Crossteaser game session, which is specific to
/// a valid variant of the game.
/// length is number of rows
/// width is number of columns
pub struct Session {
    variant: Option<String>,
    length: u64,
    width: u64,
    free: u64,
    num_pieces: u64,
}

impl Session {
    fn new(
        variant: Option<String>,
        length: u64,
        width: u64,
        free: u64,
    ) -> Self {
        Session {
            variant,
            length,
            width,
            free,
            num_pieces: length * width - free,
        }
    }
}

/// Converts an Orientation struct into its corresponding orientation hash,
/// which will be a number from 0-23
/// Uses patterns in the binary representations of an orientation to generate
/// a unique & minimal hash
#[inline]
const fn hash_orientation(o: &Orientation) -> u64 {
    (o.front << 2) | ((o.top & 1) << 1) | (o.right & 1)
}

/// Converts an orientation hash into an Orientation struct
/// Makes use of ORIENTATION_UNMAP for the conversion
#[inline]
const fn unhash_orientation(h: u64) -> Orientation {
    let packed = ORIENTATION_MAP[h as usize].view_bits::<Msb0>();
    Orientation {
        front: packed[..3],
        top: packed[3..6],
        right: packed[6..],
    }
}

impl Clone for UnhashedState {
    fn clone(&self) -> UnhashedState {
        let new_pieces = self
            .pieces
            .iter()
            .map(|o| o.clone())
            .collect();
        UnhashedState {
            pieces: new_pieces,
            free: self.free,
        }
    }
}

impl Session {
    /// Returns the total number of game pieces in the current game variant
    /// based on Session
    fn get_pieces(&self) -> u64 {
        return self.length * self.width - self.free;
    }

    /// Simple hash function that converts a vector of piece
    /// orientations and an empty space represented by an integer into a 64 bit
    /// integer (State) which uniquely represents that state.
    /// Uses minimal space for all theoretical states, does not optimize
    /// for obtainable states. There will be a lot of unused hashes here
    /// because fewer than half of the theoretical states are obtainable in
    /// 3x3 crossteaser.
    fn hash(&self, s: UnhashedState) -> State {
        let mut hashed_state: State = s.free;
        let mut mult: u64 = self.width * self.length;
        for o in s.pieces {
            hashed_state += hash_orientation(&o) * mult;
            mult *= PIECE_SIZE;
        }
        return hashed_state;
    }

    /// Reverse of hash(), extracts the orientation vector and empty space from
    /// a State.
    fn unhash(&self, s: State) -> UnhashedState {
        let num_pieces: u64 = self.get_pieces();
        let mut curr_piece: u64;
        let mut temp_state: u64 = s;
        let space_count: u64 = self.width * self.length;
        let empty: u64 = s % space_count;
        temp_state /= space_count;
        let mut rep: Vec<Orientation> = Vec::new();
        for _ in 0..num_pieces {
            curr_piece = temp_state % PIECE_SIZE;
            temp_state /= PIECE_SIZE;
            rep.push(unhash_orientation(curr_piece));
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
        let mut new_state = s.clone();
        if s.free % self.width != 0 {
            let to_move: usize = s.free as usize - 1;
            new_state.pieces[to_move].right();
            new_state.free = to_move as u64;
        }
        return new_state;
    }

    /// Adjusts the entire board with a "left" move
    /// Adjusts empty space accordingly
    /// Makes use of mov::left()
    fn board_left(&self, s: &UnhashedState) -> UnhashedState {
        let mut new_state = s.clone();
        if s.free % self.width != self.width - 1 {
            let to_move: usize = s.free as usize;
            new_state.pieces[to_move].left();
            new_state.free = s.free + 1;
        }
        return new_state;
    }

    /// Adjusts the entire board with a "up" move
    /// Adjusts empty space accordingly
    /// Makes use of mov::up()
    fn board_up(&self, s: &UnhashedState) -> UnhashedState {
        let mut new_state = s.clone();
        if s.free / self.width != self.length - 1 {
            let to_move: usize = (s.free + self.width) as usize;
            let dest: usize = s.free as usize;
            let mut piece: Orientation = new_state
                .pieces
                .remove(to_move - 1);
            piece.up();
            new_state
                .pieces
                .insert(dest, piece);
            new_state.free = to_move as u64;
        }
        return new_state;
    }

    /// Adjusts the entire board with a "down" move
    /// Adjusts empty space accordingly
    /// Makes use of mov::down()
    fn board_down(&self, s: &UnhashedState) -> UnhashedState {
        let mut new_state = s.clone();
        if s.free / self.width != 0 {
            let to_move: usize = (s.free - self.width) as usize;
            let dest: usize = s.free as usize - 1;
            let mut piece: Orientation = new_state.pieces.remove(to_move);
            piece.down();
            new_state
                .pieces
                .insert(dest, piece);
            new_state.free = to_move as u64;
        }
        return new_state;
    }

    /// Rotates entire board clockwise 90 degrees. Used only for
    /// combined flip board + rotate 90 symmetry.
    fn board_cw(&self, s: &UnhashedState) -> UnhashedState {
        if self.width != self.length {
            panic!("Cannot rotate board with unequal dimensions 90 degrees");
        }
        let mut pieces: Vec<Orientation> = Vec::new();
        for col in 0..self.width {
            for row in (0..self.length).rev() {
                let mut pos: u64 = row * self.width + col;
                if pos != s.free {
                    if pos > s.free {
                        pos -= 1;
                    }
                    let mut piece = s.pieces[pos as usize].clone();
                    piece.cw_front();
                    pieces.push(piece);
                }
            }
        }
        let row: u64 = s.free / self.width;
        let col: u64 = s.free % self.width;
        UnhashedState {
            pieces,
            free: col * self.width + self.width - row - 1,
        }
    }

    /// Rotates entire board 180 degrees
    fn board_180(&self, s: &UnhashedState) -> UnhashedState {
        let num_pieces: u64 = self.get_pieces();
        let pieces = s
            .pieces
            .iter()
            .rev()
            .map(|o| {
                let mut new_piece = o.clone();
                new_piece.front_180();
                new_piece
            })
            .collect();
        let f: u64 = num_pieces - s.free;
        UnhashedState { pieces, free: f }
    }

    /// Flips board over from left to right
    fn flip_board(&self, s: &UnhashedState) -> UnhashedState {
        let mut rep: Vec<Orientation> = Vec::new();
        for row in 0..self.length {
            for col in (0..self.width).rev() {
                let mut pos: u64 = row * self.width + col;
                if pos != s.free {
                    if pos > s.free {
                        pos -= 1;
                    }
                    rep.push(mov::top_180(&s.pieces[pos as usize]));
                }
            }
        }
        let new_free: u64 = self.width - (s.free % self.width) - 1
            + (s.free / self.width) * self.width;
        return UnhashedState {
            pieces: rep,
            free: new_free,
        };
    }

    /// Combines flipping board plus 90 degree rotation, this is a
    /// valid symmetry due to the perpedicular slot orentation on front and back
    fn flip_90(&self, s: &UnhashedState) -> UnhashedState {
        if self.width == self.length {
            return self.board_cw(&self.flip_board(s));
        } else {
            return s.clone();
        }
    }

    /// Reduces the state to one with last piece orientation of 0
    /// and adjusting the rest of the pieces equivalently
    /// Note that this does not mean applying the exact same transformation
    /// to every piece.
    /// See apply_transformation() for additional details.
    fn reduce(&self, s: &mut UnhashedState) {
        let pos: u64 = hash_orientation(&s.pieces.last().unwrap());
        if pos != 0 {
            let transform: u64 = TRANSFORM_MAP[pos as usize];
            s.pieces
                .iter_mut()
                .for_each(|piece| piece.apply_transforms(transform));
        }
    }

    /// Applies 4 physical board symmetries and finds the canonical of the set
    /// -Rotate 180
    /// -Flip board & rotate 90
    /// -Flip board & rotate 270
    /// NOTE: for boards with unequal dimensions, cannot apply
    /// Flip board & rotate 90 or flip board & rotate 270 symmetries.
    /// Not Currently in use.
    fn board_sym(&self, s: &UnhashedState) -> UnhashedState {
        let mut sym_list: Vec<UnhashedState> = Vec::new();
        sym_list.push(s.clone());
        sym_list.push(self.board_180(s));
        if self.width == self.length {
            let flipped: UnhashedState = self.flip_90(s);
            sym_list.push(flipped.clone());
            sym_list.push(self.board_180(&flipped));
        }
        let mut min: u64 = self.hash(sym_list[0]);
        let mut min_i: usize = 0;
        for i in 0..sym_list.len() {
            let curr: u64 = self.hash(sym_list[i]);
            if curr < min {
                min_i = i;
                min = curr;
            }
        }
        return sym_list.remove(min_i);
    }

    /// Applies 4 board symmetries and then reduces state to canonical
    fn canonical(&self, s: &mut UnhashedState) {
        self.reduce(s);
    }
}

impl Clone for Orientation {
    fn clone(&self) -> Orientation {
        Orientation {
            front: self.front,
            top: self.top,
            right: self.right,
        }
    }
}

impl Orientation {
    /// Transforms an individual piece orientation as if it was shifted right
    fn right(&mut self) {
        let right = self.right;
        self.right = 5 - self.front;
        self.front = right;
    }

    /// Transforms an individual piece orientation as if it was shifted left
    fn left(&mut self) {
        let front = self.front;
        self.front = 5 - self.right;
        self.right = front;
    }

    /// Transforms an individual piece orientation as if it was shifted up
    fn up(&mut self) {
        let front = self.front;
        self.front = 5 - self.top;
        self.top = front;
    }

    /// Transforms an individual piece orientation as if it was shifted down
    fn down(&mut self) {
        let top = self.top;
        self.top = 5 - self.front;
        self.front = top;
    }

    /// Transforms an individual piece orientation as if it was rotated
    /// clockwise along the front-bottom axis. Used for symmetries
    fn cw_front(&mut self) {
        let top = self.top;
        self.top = 5 - self.right;
        self.right = top;
    }

    /// Transforms an individual piece orientation as if it was rotated
    /// counter-clockwise along the front-back axis. Used for symmetries
    fn ccw_front(&mut self) {
        let right = self.right;
        self.right = 5 - self.top;
        self.top = right;
    }

    /// Transforms an individual piece orientation as if it was rotated
    /// clockwise along the top-bottom axis. Used for symmetries
    fn cw_top(&mut self) {
        let right = self.right;
        self.right = 5 - self.front;
        self.front = right;
    }

    /// Transforms an individual piece orientation as if it was rotated
    /// counter-clockwise along the top-bottom axis. Used for symmetries
    fn ccw_top(&mut self) {
        let front = self.front;
        self.front = 5 - self.right;
        self.right = front;
    }

    /// Transforms an individual piece orientation as if it was rotated
    /// clockwise along the right-left. Used for symmetries
    fn cw_right(&mut self) {
        let front = self.front;
        self.front = 5 - self.top;
        self.top = front;
    }

    /// Transforms an individual piece orientation as if it was rotated
    /// counter-clockwise along the right-left axis. Used for symmetries
    fn ccw_right(&mut self) {
        let top = self.top;
        self.top = 5 - self.front;
        self.front = top;
    }

    /// Transforms an individual piece orientation as if it was rotated
    /// 180 degrees along the front-back axis. Used for symmetries.
    fn front_180(&mut self) {
        self.top = 5 - self.top;
        self.right = 5 - self.right;
    }

    /// Transforms an individual piece orientation as if it was rotated
    /// 180 degrees along the top-bottom axis. Used for symmetries.
    fn top_180(&mut self) {
        self.front = 5 - self.front;
        self.right = 5 - self.right;
    }

    /// Transforms an individual piece orientation as if it was rotated
    /// 180 degrees along the right-left axis. Used for symmetries.
    fn right_180(&mut self) {
        self.front = 5 - self.front;
        self.top = 5 - self.top;
    }

    /// Performs a clowckwise rotation on an orientation
    /// along the specified axis.
    /// I am trying to figure out a way to make this faster.
    fn cw_on_axis(&mut self, axis: u64) {
        if axis == self.front {
            self.cw_front();
        } else if axis == 5 - self.front {
            self.ccw_front();
        } else if axis == self.top {
            self.cw_top();
        } else if axis == 5 - self.top {
            self.ccw_top();
        } else if axis == self.right {
            self.cw_right();
        } else if axis == 5 - self.right {
            self.ccw_right();
        } else {
            panic!("Invalid Orientation");
        }
    }

    /// Performs a 180 degree rotation on an orientation
    /// along the specified axis.
    pub fn axis_180(&mut self, axis: u64) {
        if axis == self.front || axis == 5 - self.front {
            self.front_180();
        } else if axis == self.top || axis == 5 - self.top {
            self.top_180();
        } else if axis == self.right || axis == 5 - self.right {
            self.right_180();
        } else {
            panic!("Invalid Orientation");
        }
    }

    /// Applies a sequence of transformations on an orientation
    /// See TRANSFORM_MAP for details on these sequences
    /// Uses cw_on_axis() to apply transformations equivalently to any piece
    /// This means it will always apply the transformations on the axis
    /// speicified by TRANSFORM_MAP, no matter the piece orientation.
    /// This allows us to reduce to an equivalent state
    /// Where equivalent means the same sequence of moves will reach a
    /// terminal state.
    fn apply_transforms(&mut self, t: u64) {
        let mut t_list: u64 = t;
        let mut transform: u64;
        let mut axis: u64;
        while t_list & 0b11 != 0 {
            transform = t_list & 0b11;
            t_list >>= 2;
            axis = t_list & 0b111;
            t_list >>= 3;
            if transform == 0b01 {
                self.cw_on_axis(axis);
            } else if transform == 0b10 {
                self.axis_180(axis);
            } else if transform == 0b11 {
                self.cw_on_axis(5 - axis);
            }
        }
    }
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
        let s: UnhashedState = self.unhash(state);
        let mut states: Vec<State> = Vec::new();
        if s.free / self.width != self.length - 1 {
            let mut s_up = self.board_up(&s);
            self.canonical(&mut s_up);
            states.push(self.hash(s_up));
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
        todo!()
    }
}

/* STATE RESOLUTION IMPLEMENTATIONS */

impl Bounded for Session {
    fn start(&self) -> State {
        // Orientations for each piece in the final (solved) position
        let pieces = vec![
            Orientation {
                front: 0,
                top: 1,
                right: 2,
            }, // Initial state with all pieces aligned
            Orientation {
                front: 0,
                top: 1,
                right: 2,
            },
            Orientation {
                front: 0,
                top: 1,
                right: 2,
            },
            Orientation {
                front: 0,
                top: 1,
                right: 2,
            },
            Orientation {
                front: 0,
                top: 1,
                right: 2,
            },
            Orientation {
                front: 0,
                top: 1,
                right: 2,
            },
            Orientation {
                front: 0,
                top: 1,
                right: 2,
            },
            Orientation {
                front: 0,
                top: 1,
                right: 2,
            },
        ];
        let unhashed_state = UnhashedState { pieces, free: 4 };
        let moved_state = self.board_up(&unhashed_state);

        self.hash(&self.canonical(&moved_state))
    }

    fn end(&self, state: State) -> bool {
        let current_state = self.unhash(state);

        // Check if the free space is in the middle
        if current_state.free != 4 {
            return false;
        }
        // Check if all pieces have the same front, top, and right orientation
        if let Some(first_piece) = current_state.pieces.first() {
            let front = first_piece.front;
            let top = first_piece.top;
            let right = first_piece.right;

            current_state
                .pieces
                .iter()
                .all(|p| p.front == front && p.top == top && p.right == right)
        } else {
            false // Return false if there are no pieces, or handle differently if needed
        }
    }
}

impl Codec for Session {
    fn decode(&self, string: String) -> Result<State> {
        parse_state(string, self).context("Malformed state")
    }

    fn encode(&self, state: State) -> String {
        let s: UnhashedState = self.unhash(state);
        let mut v: Vec<Vec<String>> = Vec::new();
        for _i in 0..self.length {
            v.push(Vec::new());
        }
        let board_size: usize = (self.width * self.length) as usize;
        let mut out: String = String::new();
        let mut row: usize;
        let mut i: usize = 0;
        let mut found_empty: usize = 0;
        while i < board_size {
            row = i / self.width as usize;
            if i as u64 == s.free {
                v[row].push("X".to_string());
                found_empty = 1;
            } else {
                v[row].push(
                    hash_orientation(&s.pieces[i - found_empty]).to_string(),
                );
            }
            i += 1;
        }
        for i in 0..self.length {
            out.push('|');
            out.push_str(&v[i as usize].join("-"));
        }
        out.push('|');
        return out;
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
