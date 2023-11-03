//! # Data Models Module
//!
//! This module contains centralized definitions for custom datatypes used
//! throughout the project.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

use crate::interfaces::terminal::cli::IOMode;
use nalgebra::SVector;

/* PRIMARY TYPES */

/// Encodes the configuration of a game in a string, which allows game
/// implementations to set themselves up differently depending on its contents.
/// The protocol used to map a variant string to a specific game setup is
/// decided by the implementation of a game, so reading game-specific
/// documentation will be necessary to properly form a variant string.
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
pub type Player = u16;

/// The signature of a function which can solve a game implementation, with side
/// effects specified by an `IOMode` optional argument. Returns the record
/// associated with the starting position of the game.
pub type Solver<G, const N: usize> = fn(&G, Option<IOMode>) -> Record<N>;

/* ATTRIBUTE TYPES */

/// A measure of how "good" an outcome is for a given player in a game. Positive
/// values indicate an overall gain from having played the game, and negative
/// values are net losses. The metric over abstract utility is subjective.
pub type Utility = i64;

/// Indicates the "depth of draw" which a drawing position corresponds to. For
/// more information, see [this whitepaper](TODO). This value should be 0 for
/// non-drawing positions.
pub type DrawDepth = u64;

/// Indicates the number of choices that players have to make to reach a
/// terminal state in a game under perfect play. For drawing positions,
/// indicates the number of choices players can make to bring the game to a
/// state which can transition to a non-drawing state.
pub type Remoteness = u64;

/// Please refer to [this](https://en.wikipedia.org/wiki/Mex_(mathematics)).
pub type MinimumExcludedValue = u64;

/* CONSTRUCTS */

/// The set of attributes related to a game position in an arbitrary `N` player
/// game.
#[derive(Eq, Hash, PartialEq)]
pub struct Record<const N: usize>
{
    pub util: SVector<Utility, N>,
    pub draw: DrawDepth,
    pub rem: Remoteness,
    pub mex: MinimumExcludedValue,
}

impl<const N: usize> Default for Record<N>
{
    fn default() -> Self
    {
        Record {
            util: SVector::<Utility, N>::zeros(),
            draw: u64::default(),
            rem: u64::default(),
            mex: u64::default(),
        }
    }
}

impl<const N: usize> Record<N>
{
    /* RECORD BUILDER LITE (TM) */

    pub fn with_util(mut self, util: SVector<Utility, N>) -> Self
    {
        self.util = util;
        self
    }

    pub fn with_draw(mut self, draw: u64) -> Self
    {
        self.draw = draw;
        self
    }

    pub fn with_rem(mut self, rem: u64) -> Self
    {
        self.rem = rem;
        self
    }

    pub fn with_mex(mut self, mex: u64) -> Self
    {
        self.mex = mex;
        self
    }

    /* RECORD UTILS */

    fn get_utility(&self, player: Player) -> Utility
    {
        if let Some(utility) = self.util.get(player as usize) {
            *utility
        } else {
            panic!(
                "Out-of-bounds vector access: Attempted to fetch utility for \
                player {} in a game of {} players.",
                player,
                self.util.nrows()
            )
        }
    }
}
