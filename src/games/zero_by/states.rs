//! # Zero-By State Handling Module
//!
//! This module helps parse the a string encoding of a zero-by game state into
//! a more efficient binary representation, performing a series of checks which
//! partially ensure compatibility with a game variant.
//!
//! #### Authorship
//!
//! - Max Fierro, 11/2/2023 (maxfierro@berkeley.edu)

use super::{Session, NAME};
use crate::{
    errors::NovaError,
    games::utils::{pack_turn, unpack_turn},
    models::{State, Turn},
};
use regex::Regex;

/* ZERO-BY STATE ENCODING */

pub const STATE_DEFAULT: &str = "10-0";
pub const STATE_PATTERN: &str = r"^\d+-\d+$";
pub const STATE_PROTOCOL: &str =
"The state string should be two dash-separated positive integers without any \
decimal points. The first integer will indicate the amount of elements left to \
remove from the set, and the second indicates whose turn it is to remove an \
element. The first integer must be less than or equal to the number of initial \
elements specified by the game variant. Likewise, the second integer must be \
strictly less than the number of players in the game.";

/* API */

/// Returns a zero-by game state encoding using the parameters specified by a
/// pre-verified game variant combined with the state update provided by the
/// state encoded in `from`. This does not verify that the provided `from` is
/// reachable in `session`'s game variant.
pub fn parse_state(
    session: &Session,
    from: String,
) -> Result<State, NovaError> {
    check_state_pattern(&from)?;
    let params = parse_parameters(&from)?;
    let (from, turn) = check_param_count(&params)?;
    check_variant_coherence(from, turn, &session)?;
    let state = pack_turn(from, turn, session.players);
    Ok(state)
}

/* STATE STRING VERIFICATION */

fn check_state_pattern(from: &String) -> Result<(), NovaError> {
    let re = Regex::new(STATE_PATTERN).unwrap();
    if !re.is_match(&from) {
        Err(NovaError::VariantMalformed {
            game_name: NAME,
            hint: format!(
                "String does not match the pattern '{}'.",
                STATE_PATTERN
            ),
        })
    } else {
        Ok(())
    }
}

fn parse_parameters(from: &String) -> Result<Vec<u64>, NovaError> {
    from.split('-')
        .map(|int_string| {
            int_string
                .parse::<u64>()
                .map_err(|e| NovaError::StateMalformed {
                    game_name: NAME,
                    hint: format!("{}", e.to_string()),
                })
        })
        .collect()
}

fn check_param_count(params: &Vec<u64>) -> Result<(State, Turn), NovaError> {
    if params.len() != 2 {
        Err(NovaError::StateMalformed {
            game_name: NAME,
            hint: format!(
                "String contains {} integers, but needs to have exactly 2.",
                params.len()
            ),
        })
    } else {
        Ok((params[0], params[1] as usize))
    }
}

fn check_variant_coherence(
    from: State,
    turn: Turn,
    session: &Session,
) -> Result<(), NovaError> {
    let (session_from, _) = unpack_turn(session.start, session.players);
    if from > session_from {
        Err(NovaError::StateMalformed {
            game_name: NAME,
            hint: format!(
                "Specified more starting elements ({}) than variant allows \
                ({}).",
                from, session.start,
            ),
        })
    } else if turn >= session.players {
        Err(NovaError::StateMalformed {
            game_name: NAME,
            hint: format!(
                "Specified a turn ({}) too high for this ({}-player) game \
                variant.",
                turn, session.players,
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
    use crate::games::{utils::verify_history_dynamic, Game};

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
        let with_none = Session::initialize(None).unwrap();
        let with_default = Session::initialize(None).unwrap();

        assert_eq!(
            with_none.start,
            parse_state(&with_default, STATE_DEFAULT.to_string()).unwrap()
        );
    }

    #[test]
    fn malformed_states_fail_checks() {
        let s1 = "-8-1".to_owned();
        let s2 = "10-2".to_owned();
        let s3 = "5-2".to_owned();
        let s4 = "2000-1".to_owned();
        let s5 = "45-32".to_owned();
        let s6 = "7-".to_owned();
        let s7 = "11-0".to_owned();

        fn f() -> Session {
            // 2-player 10-to-zero by 1 or 2
            Session::initialize(None).unwrap()
        }

        assert!(parse_state(&f(), s1).is_err());
        assert!(parse_state(&f(), s2).is_err());
        assert!(parse_state(&f(), s3).is_err());
        assert!(parse_state(&f(), s4).is_err());
        assert!(parse_state(&f(), s5).is_err());
        assert!(parse_state(&f(), s6).is_err());
        assert!(parse_state(&f(), s7).is_err());
    }

    #[test]
    fn well_formed_states_pass_checks() {
        let s1 = "10-0".to_owned();
        let s2 = "9-1".to_owned();
        let s3 = "0-0".to_owned();
        let s4 = "3-1".to_owned();
        let s5 = "5-1".to_owned();
        let s6 = "10-1".to_owned(); // <-- Impossible but well formed
        let s7 = "1-0".to_owned();

        fn f() -> Session {
            Session::initialize(None).unwrap()
        }

        assert!(parse_state(&f(), s1).is_ok());
        assert!(parse_state(&f(), s2).is_ok());
        assert!(parse_state(&f(), s3).is_ok());
        assert!(parse_state(&f(), s4).is_ok());
        assert!(parse_state(&f(), s5).is_ok());
        assert!(parse_state(&f(), s6).is_ok());
        assert!(parse_state(&f(), s7).is_ok());
    }

    #[test]
    fn compatible_variants_and_states_pass_checks() {
        let v1 = "50-10-12-1-4";
        let v2 = "5-100-6-2-7";
        let v3 = "10-200-1-5";

        let s1 = "1-4".to_owned();
        let s2 = "150-9".to_owned();
        let s3 = "200-0".to_owned();

        fn f(v: &str) -> Session {
            Session::initialize(Some(v.to_owned())).unwrap()
        }

        assert!(parse_state(&f(v1), s1.clone()).is_ok());
        assert!(parse_state(&f(v1), s2.clone()).is_err());
        assert!(parse_state(&f(v1), s3.clone()).is_err());

        assert!(parse_state(&f(v2), s1.clone()).is_ok());
        assert!(parse_state(&f(v2), s2.clone()).is_err());
        assert!(parse_state(&f(v2), s3.clone()).is_err());

        assert!(parse_state(&f(v3), s1.clone()).is_ok());
        assert!(parse_state(&f(v3), s2.clone()).is_ok());
        assert!(parse_state(&f(v3), s3.clone()).is_ok());
    }

    /* GAME HISTORY VERIFICATION */

    #[test]
    fn verify_incorrect_default_zero_by_history_fails() {
        let i1 = vec!["10-0", "9-1", "8-0", "5-1"]; // Illegal move
        let i2 = vec!["10-0", "8-0", "7-0", "5-1"]; // Turns don't switch
        let i3 = vec!["10-1", "8-0", "7-1", "5-0"]; // Starting turn wrong
        let i4 = vec!["1-10", "0-9", "1-7", "0-5"]; // Turn and state switched
        let i5 = vec!["10-0", "", "9-1", "", "8-0"]; // Empty states
        let i6 = vec!["ten-zero", "nine-one"]; // Malformed
        let i7: Vec<&str> = vec![]; // No history
        let i8 = vec![""]; // Empty string

        assert!(verify_history_dynamic(&session(None), owned(i1)).is_err());
        assert!(verify_history_dynamic(&session(None), owned(i2)).is_err());
        assert!(verify_history_dynamic(&session(None), owned(i3)).is_err());
        assert!(verify_history_dynamic(&session(None), owned(i4)).is_err());
        assert!(verify_history_dynamic(&session(None), owned(i5)).is_err());
        assert!(verify_history_dynamic(&session(None), owned(i6)).is_err());
        assert!(verify_history_dynamic(&session(None), owned(i7)).is_err());
        assert!(verify_history_dynamic(&session(None), owned(i8)).is_err());
    }

    #[test]
    fn verify_correct_default_zero_by_history_passes() {
        let c1 = vec![
            "10-0", "8-1", "6-0", "4-1", "2-0", "0-1",
        ];
        let c2 = vec!["10-0", "8-1", "6-0", "4-1", "2-0"];
        let c3 = vec!["10-0", "9-1", "7-0", "6-1"];
        let c4 = vec!["10-0", "8-1", "6-0"];
        let c5 = vec!["10-0", "9-1"];
        let c6 = vec!["10-0"];

        assert!(verify_history_dynamic(&session(None), owned(c1)).is_ok());
        assert!(verify_history_dynamic(&session(None), owned(c2)).is_ok());
        assert!(verify_history_dynamic(&session(None), owned(c3)).is_ok());
        assert!(verify_history_dynamic(&session(None), owned(c4)).is_ok());
        assert!(verify_history_dynamic(&session(None), owned(c5)).is_ok());
        assert!(verify_history_dynamic(&session(None), owned(c6)).is_ok());
    }

    #[test]
    fn verify_zero_by_history_compatibility() {
        fn v() -> Option<String> {
            Some(format!("8-200-30-70-15-1"))
        }

        let c1 = vec![
            "200-0", "185-1", "115-2", "114-3", "113-4", "83-5", "82-6",
            "81-7", "11-0", "10-1", "9-2", "0-3",
        ];
        let c2 = vec![
            "200-0", "199-1", "198-2", "197-3", "196-4", "195-5", "180-6",
            "110-7", "80-0", "79-1",
        ];

        assert!(verify_history_dynamic(&session(v()), owned(c1)).is_ok());
        assert!(verify_history_dynamic(&session(v()), owned(c2)).is_ok());

        let i1 = vec!["200-0", "184-1", "115-2", "114-3"]; // Illegal move
        let i2 = vec!["200-0", "185-1", "115-1", "114-2"]; // Turns don't switch
        let i3 = vec!["200-2", "185-3", "115-4", "114-5"]; // Bad initial turn
        let i4 = vec!["201-0", "186-1", "116-2", "115-3"]; // Bad initial state

        assert!(verify_history_dynamic(&session(v()), owned(i1)).is_err());
        assert!(verify_history_dynamic(&session(v()), owned(i2)).is_err());
        assert!(verify_history_dynamic(&session(v()), owned(i3)).is_err());
        assert!(verify_history_dynamic(&session(v()), owned(i4)).is_err());
    }

    /* UTILITIES */

    fn session(v: Option<String>) -> Session {
        Session::initialize(v).unwrap()
    }

    fn owned(v: Vec<&str>) -> Vec<String> {
        v.iter()
            .map(|&s| s.to_owned())
            .collect()
    }
}
