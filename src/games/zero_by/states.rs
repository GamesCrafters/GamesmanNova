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

use super::{variants::VARIANT_DEFAULT, Session, NAME};
use crate::{
    errors::NovaError,
    games::utils::{pack_turn, unpack_turn},
    models::{PlayerCount, State, Turn},
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

/// Returns a zero-by game session set up using the parameters specified by a
/// pre-processed game variant combined with the state update provided by the
/// state encoded in `from`. The updated state must be reachable from the start
/// state specified by the game variant.
pub fn parse_state(
    mut session: Session,
    from: String,
) -> Result<Session, NovaError> {
    check_state_pattern(&from)?;
    let params = parse_parameters(&from)?;
    let (state, turn) = check_param_count(&params)?;
    verify_state_coherence(state, turn, &session)?;
    session.start = pack_turn(state, turn, session.players);
    Ok(session)
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
                "String contains {} integers, but needs 2.",
                params.len()
            ),
        })
    } else {
        Ok((params[0], params[1] as usize))
    }
}

fn verify_state_coherence(
    state: State,
    turn: Turn,
    session: &Session,
) -> Result<(), NovaError> {
    if state > unpack_turn(session.start, session.players).0 {
        Err(NovaError::StateMalformed {
            game_name: NAME.to_owned(),
            hint: format!(
                "{} is greater than the number of initial elements in the \
                game variant {}.",
                state,
                session
                    .variant
                    .clone()
                    .unwrap_or(VARIANT_DEFAULT.to_owned())
            ),
        })
    } else if turn >= session.players {
        Err(NovaError::StateMalformed {
            game_name: NAME.to_owned(),
            hint: format!(
                "A player with an identifier of {} would exceed the player \
                count of {} indicated by the game variant {}.",
                turn,
                session.players,
                session
                    .variant
                    .clone()
                    .unwrap_or(VARIANT_DEFAULT.to_owned())
            ),
        })
    } else if !can_reach_target(
        session.start,
        state,
        &session.by,
        session.players,
        turn,
    ) {
        Err(NovaError::StateMalformed {
            game_name: NAME.to_owned(),
            hint: format!(
                "It is impossible to reach {} elements from {} with the \
                options specified by the variant {}.",
                state,
                session.start,
                session
                    .variant
                    .clone()
                    .unwrap_or(VARIANT_DEFAULT.to_owned())
            ),
        })
    } else {
        Ok(())
    }
}

/* HELPER FUNCTIONS */

fn can_reach_target(
    from: u64,
    to: u64,
    by: &Vec<u64>,
    players: PlayerCount,
    turn: Turn,
) -> bool {
    let target = if let Some(t) = from.checked_sub(to) {
        t as usize
    } else {
        return false;
    };
    let mut dp = vec![(usize::MAX, false); target + 1];
    dp[0] = (0, true);
    for i in 1..=target {
        for j in by {
            let j = *j as usize;
            if i >= j && dp[(i - j) as usize].1 {
                let found = dp[(i - j) as usize].0 + 1;
                if dp[i as usize].0 > found {
                    dp[i as usize] = (found, true);
                }
            }
        }
    }
    dp[target].1 && dp[target].0 >= turn
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
    fn state_default_compatible_with_variant_default() {
        let with_none = Session::initialize(None, None);
        let with_default =
            Session::initialize(None, Some(STATE_DEFAULT.to_owned()));
        assert!(with_none.is_ok());
        assert!(with_default.is_ok());
    }

    #[test]
    fn no_state_equals_default_state() {
        let with_none = Session::initialize(None, None).unwrap();
        let with_default =
            Session::initialize(None, Some(STATE_DEFAULT.to_owned())).unwrap();
        assert_eq!(with_none.variant, with_default.variant);
        assert_eq!(with_none.start, with_default.start);
        assert_eq!(with_none.by, with_default.by);
    }

    #[test]
    fn obvious_invalid_states_fail_checks() {
        let s1 = "-8-1".to_owned();
        let s2 = "10-1".to_owned();
        let s3 = "5-2".to_owned();
        let s4 = "2000-1".to_owned();
        let s5 = "45-32".to_owned();
        let s6 = "7-".to_owned();
        let s7 = "11-0".to_owned();

        fn f() -> Session {
            Session::initialize(None, None).unwrap()
        }

        assert!(parse_state(f(), s1).is_err());
        assert!(parse_state(f(), s2).is_err());
        assert!(parse_state(f(), s3).is_err());
        assert!(parse_state(f(), s4).is_err());
        assert!(parse_state(f(), s5).is_err());
        assert!(parse_state(f(), s6).is_err());
    }

    #[test]
    fn incoherent_states_fail_checks() {
        let s1 = "100-5".to_owned();
        let s2 = "101-0".to_owned();
        let s3 = "95-2".to_owned();
        let s4 = "87-0".to_owned();
        let s5 = "49-4".to_owned();
        let s6 = "3-1".to_owned();

        fn f() -> Session {
            let variant = "5-100-6-14-20".to_owned();
            Session::initialize(Some(variant), None).unwrap()
        }

        assert!(parse_state(f(), s1).is_err());
        assert!(parse_state(f(), s2).is_err());
        assert!(parse_state(f(), s3).is_err());
        assert!(parse_state(f(), s4).is_err());
        assert!(parse_state(f(), s5).is_err());
        assert!(parse_state(f(), s6).is_err());
    }

    #[test]
    fn coherent_states_pass_checks() {
        let s1 = "100-5".to_owned();
        let s2 = "150-0".to_owned();
        let s3 = "95-2".to_owned();
        let s4 = "88-0".to_owned();
        let s5 = "49-4".to_owned();
        let s6 = "3-1".to_owned();

        fn f() -> Session {
            let variant = "10-150-6-15-20-1".to_owned();
            Session::initialize(Some(variant), None).unwrap()
        }

        assert!(parse_state(f(), s1).is_ok());
        assert!(parse_state(f(), s2).is_ok());
        assert!(parse_state(f(), s3).is_ok());
        assert!(parse_state(f(), s4).is_ok());
        assert!(parse_state(f(), s5).is_ok());
        assert!(parse_state(f(), s6).is_ok());
    }
}
