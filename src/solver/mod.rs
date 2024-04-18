//! # Solvers Module
//!
//! This module provides behavior for the systematic traversal of game trees
//! via their implementation of different interfaces defining deterministic or
//! probabilistic behavior, with the objective of computing their strong or weak
//! solutions, or finding "solutions" under different game-theoretic definitions
//! of that word.
//!
//! #### Authorship
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)
//! - Ishir Garg, 4/3/2024 (ishirgarg@berkeley.edu)

use crate::model::PlayerCount;

/* CONSTANTS */

/// Describes the maximum number of states that are one move away from any state
/// within a game. Used to allocate statically-sized arrays on the stack for
/// faster execution of solving algorithms. If this limit is violated by a game
/// implementation, this program should panic.
pub const MAX_TRANSITIONS: usize = 128;

/* MODULES */

/// Implementations of records that can be used by solving algorithms to store
/// or persist the information they compute about a game, and communicate it to
/// a database system.
pub mod record {
    pub mod mur;
    pub mod sur;
    pub mod rem;
}

/// Implementations of algorithms that can consume game implementations and
/// compute different features of interest associated with groups of states or
/// particular states.
pub mod algorithm {
    /// Solving algorithms for games that are either of incomplete information
    /// or non-deterministic. The strategies used here diverge somewhat from the
    /// other solving procedures, as bringing in probability is a fundamental
    /// change.
    pub mod stochastic {
        pub mod acyclic;
        pub mod cyclic;
    }

    /// Solving algorithms for deterministic complete-information games that are
    /// able to generate complete solution sets (from which an equilibrium
    /// strategy can be distilled for any possible state in the game).
    pub mod strong {
        pub mod acyclic;
        pub mod cyclic;
        pub mod puzzle;
    }

    /// Solving algorithms for deterministic complete-information games that
    /// only guarantee to provide an equilibrium strategy for the underlying
    /// game's starting position, but which do not necessarily explore entire
    /// game trees.
    pub mod weak {
        pub mod acyclic;
        pub mod cyclic;
    }

    /// These are custom records for certain solving algorithms that may need to
    /// store additional data; these should not be accessible outside the
    /// solving algorithms
    mod record {
        pub mod surcc;
    }
}

#[cfg(test)]
mod test;
mod error;
mod util;

/* RECORD MODULES */

/// A record layout that can be used to encode and decode the attributes stored
/// in serialized records. This is stored in database table schemas so that it
/// can be retrieved later for deserialization.
#[derive(Clone, Copy)]
pub enum RecordType {
    /// Real Utility Remoteness record for a specific number of players.
    RUR(PlayerCount),
    /// Simple Utility Remoteness record for a specific number of players.
    SUR(PlayerCount),
    /// Remoteness record (no utilities).
    REM,
}
