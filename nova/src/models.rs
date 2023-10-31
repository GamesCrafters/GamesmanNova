//! # Data Models Module
//!
//! This module contains centralized definitions for custom datatypes used
//! throughout the project.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

use nalgebra::DVector;

use crate::interfaces::terminal::cli::IOMode;

/* TYPES */

/// Encodes the configuration of a game in a string, which allows game
/// implementations to set themselves up differently depending on its contents.
/// The protocol used to map a variant string to a specific game setup is
/// decided by the implementation of a game, so reading game-specific
/// documentation will be necessary to porperly form a variant string.
pub type Variant = String;

/// Encodes the state of a game in a 64-bit unsigned integer. This also
/// sets a limiting upper bound on the amount of possible non-equivalent states
/// that can be achieved in a game.
pub type State = u64;

/// Expresses whose turn it is in a game, where every player is assigned to a
/// different integer. The fact that this only reaches `u16::MAX == 255` does
/// mean that we should only be prepared to consider games of up to 255 players.
/// This is a reasonable limitation, because considering games of any larger
/// player count is computationally unfeasible in transferrable utility
/// settings.
pub type Player = u8;

/// The signature of a function which can solve a game implementation, with side
/// effects specified by an `IOMode` optional argument. Returns the record
/// associated with the starting position of the game.
pub type Solver<G> = fn(&G, Option<IOMode>) -> Record;

/* CONSTRUCTS */

/// The set of attributes related to a game position in an arbitrary `N` player
/// game.
///
/// TODO: Explanations of the meaning of each attribute.
pub struct Record
{
    pub utility: DVector<f64>,
    pub draw_depth: u64,
    pub remoteness: u64,
    pub mex: u64,
}
