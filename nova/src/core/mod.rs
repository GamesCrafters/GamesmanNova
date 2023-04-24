//! # Core Behavior Module
//!
//! `core` is a collection of analyzers, databases, and solvers which can be
//! applied to any deterministic finite-state abstract strategy game through
//! common abstract interfaces.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

/* MODULES */

pub mod analyzers;
pub mod databases;
pub mod solvers;

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

/// Represents how far away a state is from its corresponding terminal state.
pub type Remoteness = u8;

/// Gives a metric for determining how much of an advantage one player has.
pub type WinBy = u8;

/// Stands for 'minimum excluded value' -- for any given state, it is the
/// minimum amount of moves one has to make before getting to any end state.
pub type MinExclusion = u8;

/// The signature of a function which can solve a game, taking in the game,
/// and parameters read and write.
pub type Solver<G> = fn(&G, bool, bool) -> Value;

/* GAME STATE CHARACTERISTICS */

/// Indicates the value of a game state according to the game's rules. Contains
/// remoteness information (how far away a state is from its corresponding
/// terminal state).
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Value {
    /// Indicates that a player has won.
    Win(StateDescription),
    /// Indicates that a player has lost.
    Lose(StateDescription),
    /// Indicates that the game is a tie.
    Tie(StateDescription),
}

macro_rules! gamestate_features {
    ($($name:ident; $type:ty),+) => {
        #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
        pub struct StateDescription {
            $(pub $name: Option<$type>,)+
        }

        #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
        pub struct StateDescriptionMask {
            $(pub $name: bool,)+
        }
    }
}

gamestate_features!(
    rem; Remoteness,
    mex; MinExclusion,
    wby; WinBy
);
