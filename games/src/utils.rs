//! # Game Utilities Module
//!
//! This module includes useful functionality common in the process of writing
//! game implementations.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

/* MACROS */

/// Implements multiple traits for a single concrete type. The traits
/// implemented must be marker traits; in other words, they must have no
/// behavior (no functions). You will usually want to use this for implementing
/// all the solvers for a game ergonomically through their marker traits.
///
/// Example usage:
///
/// ```rust
/// implement! { for Game =>
///     AcyclicallySolvable,
///     TreeSolvable,
///     TierSolvable
/// }
/// ```
///
/// ...which expands to the following:
///
/// ```rust
/// impl AcyclicallySolvable for Game { }
///
/// impl TreeSolvable for Game { }
///
/// impl TierSolvable for Game { }
/// ```
#[macro_export]
macro_rules! implement {
    (for $b:ty => $($t:ty),+) => {
        $(impl $t for $b { })*
    }
}
