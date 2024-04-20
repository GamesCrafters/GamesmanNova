//! # Crossteaser State Handling Module
//!
//! This module helps parse the a string encoding of a crossteaser game state
//! into a more efficient binary representation, performing a series of checks
//! which partially ensure compatibility with a game variant.
//!
//! #### Authorship
//! - Max Fierro, 3/7/2023 (maxfierro@berkeley.edu)
//! - Michael Setchko Palmerlee, 4/18/2024 (michaelsp@berkeley.edu)

pub const STATE_DEFAULT: &'static str = "|0-0-0|0-X-0|0-0-0|";
pub const STATE_PATTERN: &'static str = r"^([|]([\dX]-)+[\dX])+[|]$";
pub const STATE_PROTOCOL: &'static str =
    "Rows are separated by |, columns are separated by -, empty space is X. \
Integers 0-24 are a piece orientation as defined by ORIENTATION_MAP";

#[cfg(test)]
mod test {

    use super::*;
    use crate::game::crossteaser::*;
    use crate::game::{util::verify_history_dynamic, Game};
    use std::collections::HashSet;

    /* STATE STRING PARSING */

    #[test]
    fn test_transition() {
        let session: Session = Session {
            variant: None,
            length: 3,
            width: 3,
            free: 1,
        };
        let mut s: UnhashedState = UnhashedState {
            pieces: Vec::new(),
            free: 4,
        };
        for _i in 0..8 {
            s.pieces
                .push(unhash_orientation(0));
        }
        let mut found: HashSet<State> = HashSet::new();
        let mut unsolved: Vec<State> = Vec::new();
        unsolved.push(session.hash(&s));
        for i in 0..1000 {
            let s: State = unsolved.pop().unwrap();
            found.insert(s);
            let f: Vec<State> = session.prograde(s);
            for state in f {
                if !found.contains(&state) {
                    unsolved.push(state);
                }
            }
        }
    }
}
