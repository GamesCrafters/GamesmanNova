//! # Utilities Module
//!
//! This module factors out common behavior across this project.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

use crate::errors::NovaError;

/// Checks if the game exists among the offerings and returns an error if
/// it does not. Includes a suggestion in the error information.
pub fn check_game_exists(name: &String) -> Result<(), NovaError>
{
    if !crate::games::LIST.contains(&&name[0..]) {
        Err(NovaError::GameNotFoundError(name.to_owned()))
    } else {
        Ok(())
    }
}

/// Returns the most similar string to `model` in the collection.
pub fn most_similar(model: &str, all: Vec<&str>) -> String
{
    let mut best = usize::MAX;
    let mut closest = "";
    let mut curr;
    for s in all {
        curr = strsim::damerau_levenshtein(model, s);
        if curr <= best {
            closest = s;
            best = curr;
        }
    }
    closest.to_owned()
}

/// Implements multiple traits for a single concrete type. The traits
/// implemented must be marker traits; in other words, they must have no
/// behavior (no functions). You will usually want to use this for implementing
/// all the solvers for a game ergonomically through their marker traits.
///
/// Example usage:
///
/// ```ignore
/// implement! { for Game =>
///     AcyclicGame,
///     AcyclicallySolvable,
///     TreeSolvable,
///     TierSolvable
/// }
/// ```
///
/// ...which expands to the following:
///
/// ```ignore
/// impl AcyclicallySolvable for Game {}
///
/// impl TreeSolvable for Game {}
///
/// impl TierSolvable for Game {}
/// ```
#[macro_export]
macro_rules! implement {
    (for $b:ty => $($t:ty),+) => {
        $(impl $t for $b { })*
    }
}
