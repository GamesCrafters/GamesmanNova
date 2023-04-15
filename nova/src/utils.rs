//! # Nova Utilities Module
//!
//! This module factors out common behavior of the command execution handling
//! modules in this crate.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/9/2023 (maxfierro@berkeley.edu)

use strsim::damerau_levenshtein;

/// Returns the most similar string to `model` in the collection.
pub fn most_similar(model: &str, all: Vec<&str>) -> String {
    let mut best = usize::MAX;
    let mut closest = "";
    let mut curr;
    for s in all {
        curr = damerau_levenshtein(model, s);
        if curr <= best {
            closest = s;
            best = curr;
        }
    }
    closest.to_owned()
}
