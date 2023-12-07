//! # Zero-By State Handling Module
//!
//! This module helps parse the state string provided to the Zero-By game
//! into parameters that can help build a game session, in addition to providing
//! a way to translate any string encoding of a Zero-By state into the game's
//! internal representation.
//!
//! #### Authorship
//!
//! - Max Fierro, 11/2/2023 (maxfierro@berkeley.edu)

use super::{Session, NAME};
use crate::{
    errors::NovaError,
    games::utils::pack_turn,
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
elements specified by the game variant, and it must also be reachable from \
that state according to the rules of the variant. Likewise, the second integer \
must be strictly less than the number of players in the game.";

/* API */

/// Returns a zero-by game state encoding using the parameters specified by a
/// pre-verified game variant combined with the state update provided by the
/// state encoded in `from`. This does not verify that the provided `from` is
/// reachable in `session`'s game variant.
pub fn parse_state(
    session: &mut Session,
    from: String,
) -> Result<State, NovaError> {
    check_state_pattern(&from)?;
    let params = parse_parameters(&from)?;
    let (state, turn) = check_param_count(&params)?;
    check_coherence(state, turn, &session)?;
    let state = pack_turn(state, turn, session.players);
    Ok(state)
}

/* STATE STRING VERIFICATION */

fn check_state_pattern(from: &String) -> Result<(), NovaError> {
    let re = Regex::new(STATE_PATTERN).unwrap();
    if !re.is_match(&from) {
        Err(NovaError::VariantMalformed {
            game_name: NAME.to_owned(),
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
                    game_name: NAME.to_owned(),
                    hint: format!("{}", e.to_string()),
                })
        })
        .collect()
}

fn check_param_count(params: &Vec<u64>) -> Result<(State, Turn), NovaError> {
    if params.len() != 2 {
        Err(NovaError::StateMalformed {
            game_name: NAME.to_owned(),
            hint: format!(
                "String contains {} integers, but needs to have exactly 2.",
                params.len()
            ),
        })
    } else {
        Ok((params[0], params[1] as usize))
    }
}

fn check_coherence(
    state: State,
    turn: Turn,
    session: &Session,
) -> Result<(), NovaError> {
    if state > session.start {
        Err(NovaError::StateMalformed {
            game_name: NAME.to_owned(),
            hint: format!(
                "Specified more starting elements ({}) than variant allows \
                ({}).",
                state, session.start,
            ),
        })
    } else if turn >= session.players {
        Err(NovaError::StateMalformed {
            game_name: NAME.to_owned(),
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
    use crate::games::Game;

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

        let mut with_default = Session::initialize(None).unwrap();
        with_default.forward(vec![STATE_DEFAULT.to_owned()]);

        assert_eq!(with_none.variant, with_default.variant);
        assert_eq!(with_none.start, with_default.start);
        assert_eq!(with_none.by, with_default.by);
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

        assert!(parse_state(&mut f(), s1).is_err());
        assert!(parse_state(&mut f(), s2).is_err());
        assert!(parse_state(&mut f(), s3).is_err());
        assert!(parse_state(&mut f(), s4).is_err());
        assert!(parse_state(&mut f(), s5).is_err());
        assert!(parse_state(&mut f(), s6).is_err());
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

        assert!(parse_state(&mut f(), s1).is_ok());
        assert!(parse_state(&mut f(), s2).is_ok());
        assert!(parse_state(&mut f(), s3).is_ok());
        assert!(parse_state(&mut f(), s4).is_ok());
        assert!(parse_state(&mut f(), s5).is_ok());
        assert!(parse_state(&mut f(), s6).is_ok());
    }
}
