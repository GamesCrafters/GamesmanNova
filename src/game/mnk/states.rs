//! # MNK State Handling Module
//!
//! This module helps parse the a string encoding of an m,n,k game state into
//! a more efficient binary representation, performing a series of checks which
//! partially ensure compatibility with a game variant.

use anyhow::Result;
use anyhow::bail;
use regex::Regex;

use crate::game::State;
use crate::game::error::GameError;
use crate::game::mnk::Board;
use crate::game::mnk::MAX_BOARD_SIDE;
use crate::game::mnk::NAME;
use crate::game::mnk::Session;
use crate::game::mnk::Symbol;

/* MNK STATE ENCODING */

pub const STATE_DEFAULT: &str = "[[_, _, _], [_, _, _], [_, _, _]]";
pub const STATE_PATTERN: &str = r"^\[\s*(?:\[\s*(?:[XO_])(?:\s*,\s*[XO_])*\s*\])(?:\s*,\s*\[\s*(?:[XO_])(?:\s*,\s*[XO_])*\s*\])*\s*\]$";
pub const STATE_PROTOCOL: &str = "List of shape (m, n); that is, a list of \
length m, where each item is a list of length n. Here, m and n are variant \
parameters. Lists are comma-separated and bracket-enclosed, with whitespace \
after each comma. The elements of the inner list should be one of the \
following characters: 'X', 'O', or '_'. In all variants, the player with \
symbol 'X' goes first.";

/* API */

/// Returns an m,n,k game state encoding using the parameters specified by a
/// pre-verified game variant combined with the state update provided by the
/// state encoded in `from`. This does not verify that the provided `from` is
/// reachable in `session`'s game variant.
pub fn decode_state_string(session: &Session, from: String) -> Result<State> {
    check_state_pattern(&from)?;
    let symbols: Vec<Symbol> = from
        .chars()
        .filter_map(|c| match c {
            'X' => Some(Symbol::X),
            'O' => Some(Symbol::O),
            '_' => Some(Symbol::B),
            _ => None,
        })
        .collect();

    let m = session.m;
    let n = session.n;
    if symbols.len() != m * n {
        bail!(
            "Expected {} symbols for a {}×{} board, but found {}.",
            m * n,
            m,
            n,
            symbols.len()
        );
    }

    let mut board: Board = [[Symbol::B; MAX_BOARD_SIDE]; MAX_BOARD_SIDE];
    for (idx, &sym) in symbols.iter().enumerate() {
        let row = idx / m;
        let col = idx % n;
        board[row][col] = sym;
    }

    let xs = symbols
        .iter()
        .filter(|&s| *s == Symbol::X)
        .count();

    let os = symbols
        .iter()
        .filter(|&s| *s == Symbol::O)
        .count();

    // X's are turn 1, O's are turn 0.
    let turn = if xs <= os { 1 } else { 0 };
    let state = Session::encode_state(session, turn, &board);
    Ok(state)
}

/// Returns an m,n,k game state string encoding using the parameters specified
/// by a pre-verified game variant, corresponding to the input `board`. The turn
/// is inferred from the board.
pub fn encode_state_string(session: &Session, board: &Board) -> Result<String> {
    let m = session.m;
    let n = session.n;

    let mut row_strs = Vec::with_capacity(m);
    (0..m).for_each(|i| {
        let mut elems = Vec::with_capacity(n);
        for j in 0..n {
            let ch = match board[i][j] {
                Symbol::X => 'X',
                Symbol::O => 'O',
                Symbol::B => '_',
            };
            elems.push(ch.to_string());
        }
        row_strs.push(format!("[{}]", elems.join(", ")));
    });

    let full = format!("[{}]", row_strs.join(", "));
    Ok(full)
}

/* HELPERS */

fn check_state_pattern(from: &String) -> Result<(), GameError> {
    let re = Regex::new(STATE_PATTERN).unwrap();
    if !re.is_match(from) {
        Err(GameError::StateMalformed {
            game: NAME,
            hint: format!(
                "Input string '{from}' does not match the pattern \
                '{STATE_PATTERN}'.",
            ),
        })
    } else {
        Ok(())
    }
}

/* TESTS */

#[cfg(test)]
mod test {

    use super::*;
    use crate::game::*;

    /* STATE STRING PARSING */

    #[test]
    fn state_pattern_is_valid_regex() {
        assert!(Regex::new(STATE_PATTERN).is_ok());
    }

    #[test]
    fn default_state_matches_state_pattern() {
        let re = Regex::new(STATE_PATTERN).unwrap();
        assert!(re.is_match(STATE_DEFAULT));
    }

    #[test]
    fn no_state_equals_default_state() {
        let with_none = Session::default();
        let with_default = Session::default();

        assert_eq!(
            with_none.start,
            decode_state_string(&with_default, STATE_DEFAULT.to_string())
                .unwrap()
        );
    }

    #[test]
    fn malformed_states_fail_checks() {
        let s1 = "[[,,],\n[,,],\n[,,]]".to_owned();
        let s2 = "[[X, O, X], [_, _, _], [A, _, O]]".to_owned();
        let s3 = "[[X, O, X], [_, _, _]]".to_owned();
        let s4 = "[[X, O, X], [_, _, _], [X, _, O, _]]".to_owned();
        let s5 = "[[X, _, X],\t [_, _, _],\t [X, O, _], [_, _, _]]".to_owned();

        fn f() -> Session {
            Session::default()
        }

        assert!(decode_state_string(&f(), s1).is_err());
        assert!(decode_state_string(&f(), s2).is_err());
        assert!(decode_state_string(&f(), s3).is_err());
        assert!(decode_state_string(&f(), s4).is_err());
        assert!(decode_state_string(&f(), s5).is_err());
    }

    #[test]
    fn well_formed_states_pass_checks() {
        let s1 = "[[_, _,\t _],\n\n [_, _, _], [_, _, O]]".to_owned();
        let s2 = "[[_, _, _], [_, O, _],\n [_, _, X]]".to_owned();
        let s3 = "[[X, _, \t X], [_, O, _], [O, _, X]]".to_owned();
        let s4 = "[\n[X, O, X],\n [_, O, _],\n [O, X, X]\n]".to_owned();

        fn f() -> Session {
            Session::default()
        }

        assert!(decode_state_string(&f(), s1).is_ok());
        assert!(decode_state_string(&f(), s2).is_ok());
        assert!(decode_state_string(&f(), s3).is_ok());
        assert!(decode_state_string(&f(), s4).is_ok());
    }

    #[test]
    fn compatible_variants_and_states_pass_checks() -> Result<()> {
        let v1 = "4-4-4";
        let v2 = "3-4-3";
        let v3 = "2-2-2";

        let s3 = "[[X, _], [_, O]]".to_owned();
        let s2 = "[[X, _, _, _], [_, O, _, _], [_, _, X, _]]".to_owned();
        let s1 = "[[X, _, _, _], [_, O, _, _], [_, _, X, _], [_, _, _, O]]"
            .to_owned();

        assert!(decode_state_string(&variant(v1)?, s1.clone()).is_ok());
        assert!(decode_state_string(&variant(v1)?, s2.clone()).is_err());
        assert!(decode_state_string(&variant(v1)?, s3.clone()).is_err());

        assert!(decode_state_string(&variant(v2)?, s1.clone()).is_err());
        assert!(decode_state_string(&variant(v2)?, s2.clone()).is_ok());
        assert!(decode_state_string(&variant(v2)?, s3.clone()).is_err());

        assert!(decode_state_string(&variant(v3)?, s1.clone()).is_err());
        assert!(decode_state_string(&variant(v3)?, s2.clone()).is_err());
        assert!(decode_state_string(&variant(v3)?, s3.clone()).is_ok());
        Ok(())
    }

    /* UTILITIES */

    fn variant(v: &str) -> Result<Session> {
        Session::variant(v.to_string())
    }
}
