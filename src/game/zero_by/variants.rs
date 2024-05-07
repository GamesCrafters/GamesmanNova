//! # Zero-By Variant Handling Module
//!
//! This module helps parse the variant string provided to the Zero-By game
//! into parameters that can help build a game session.
//!
//! #### Authorship
//! - Max Fierro, 11/2/2023 (maxfierro@berkeley.edu)

use bitvec::field::BitField;
use regex::Regex;

use crate::game::error::GameError;
use crate::game::zero_by::{Session, NAME};
use crate::model::game::{Player, State};
use crate::util::min_ubits;

/* ZERO-BY VARIANT ENCODING */

pub const VARIANT_DEFAULT: &'static str = "2-10-1-2";
pub const VARIANT_PATTERN: &'static str = r"^[1-9]\d*(?:-[1-9]\d*)+$";
pub const VARIANT_PROTOCOL: &'static str =
"The variant string should be a dash-separated group of three or more positive \
integers. For example, '4-232-23-6-3-6' is valid but '598', '-23-1-5', and \
'fifteen-2-5' are not. The first integer represents the number of players in \
the game. The second integer represents the number of elements in the set. The \
rest are choices that the players have when they need to remove a number of \
pieces on their turn. Note that the numbers can be repeated, but if you repeat \
the first number it will be a win for the player with the first turn in 1 \
move. If you repeat any of the rest of the numbers, the only consequence will \
be a slight decrease in performance.";

/* API */

/// Returns a zero-by game session set up using the parameters specified by
/// `variant`. Returns a `GameError::VariantMalformed` if the variant string
/// does not conform to the variant protocol.
pub fn parse_variant(variant: String) -> Result<Session, GameError> {
    check_variant_pattern(&variant)?;
    let params = parse_parameters(&variant)?;
    check_param_count(&params)?;
    check_params_are_positive(&params)?;
    let players = parse_player_count(&params)?;

    let start_elems = params[1];
    let mut start_state = State::ZERO;
    let player_bits = min_ubits(players as u64);
    start_state[..player_bits].store_be(Player::default());
    start_state[player_bits..].store_be(start_elems);

    Ok(Session {
        start_state,
        start_elems,
        player_bits,
        players,
        variant,
        by: Vec::from(&params[2..]),
    })
}

/* VARIANT STRING VERIFICATION */

fn parse_parameters(variant: &str) -> Result<Vec<u64>, GameError> {
    let params: Result<Vec<u64>, _> = variant
        .split('-')
        .map(|int_string| {
            int_string
                .parse::<u64>()
                .map_err(|e| GameError::VariantMalformed {
                    game_name: NAME,
                    hint: format!("{}", e.to_string()),
                })
        })
        .collect();
    params
}

fn check_variant_pattern(variant: &String) -> Result<(), GameError> {
    let re = Regex::new(VARIANT_PATTERN).unwrap();
    if !re.is_match(&variant) {
        Err(GameError::VariantMalformed {
            game_name: NAME,
            hint: format!(
                "String does not match the pattern '{}'.",
                VARIANT_PATTERN
            ),
        })
    } else {
        Ok(())
    }
}

fn check_param_count(params: &Vec<u64>) -> Result<(), GameError> {
    if params.len() < 3 {
        Err(GameError::VariantMalformed {
            game_name: NAME,
            hint: format!(
                "String needs to have at least 3 dash-separated integers."
            ),
        })
    } else {
        Ok(())
    }
}

fn check_params_are_positive(params: &Vec<u64>) -> Result<(), GameError> {
    if params.iter().any(|&x| x <= 0) {
        Err(GameError::VariantMalformed {
            game_name: NAME,
            hint: format!("All integers in the string must be positive."),
        })
    } else {
        Ok(())
    }
}

fn parse_player_count(params: &Vec<u64>) -> Result<Player, GameError> {
    if params[0] > (Player::MAX as u64) {
        Err(GameError::VariantMalformed {
            game_name: NAME,
            hint: format!(
                "The number of players in the game must be lower than {}.",
                Player::MAX
            ),
        })
    } else {
        Ok(Player::try_from(params[0]).unwrap())
    }
}

/* TESTS */

#[cfg(test)]
mod test {

    use super::*;
    use crate::game::*;

    #[test]
    fn variant_pattern_is_valid_regex() {
        assert!(Regex::new(VARIANT_PATTERN).is_ok());
    }

    #[test]
    fn default_variant_matches_variant_pattern() {
        let re = Regex::new(VARIANT_PATTERN).unwrap();
        assert!(re.is_match(VARIANT_DEFAULT));
    }

    #[test]
    fn initialization_success_with_no_variant() {
        let _ = Session::new();
        let with_default =
            Session::new().into_variant(Some(VARIANT_DEFAULT.to_owned()));
        assert!(with_default.is_ok());
    }

    #[test]
    fn no_variant_equals_default_variant() -> Result<()> {
        let with_none = Session::new();
        let with_default =
            Session::new().into_variant(Some(VARIANT_DEFAULT.to_owned()))?;
        assert_eq!(with_none.variant, with_default.variant);
        assert_eq!(with_none.start_state, with_default.start_state);
        assert_eq!(with_none.by, with_default.by);
        Ok(())
    }

    #[test]
    fn invalid_variants_fail_checks() {
        let v1 = "23-34-0-23";
        let v2 = "two-three-five";
        let v3 = "234572342-2345";
        let v4 = "34-236--8-6-3";
        let v5 = "0-12-234-364";
        let v6 = "-234-256";

        fn wrapper(v: &'static str) -> Result<Session, GameError> {
            parse_variant(v.to_owned())
        }

        assert!(wrapper(v1).is_err());
        assert!(wrapper(v2).is_err());
        assert!(wrapper(v3).is_err());
        assert!(wrapper(v4).is_err());
        assert!(wrapper(v5).is_err());
        assert!(wrapper(v6).is_err());
    }

    #[test]
    fn valid_variants_pass_checks() {
        let v1 = "5-1000-8-23-63-7";
        let v2 = "1-1-1";
        let v3 = "34-23623-8-6-3";
        let v4 = "5-2-8-23";
        let v5 = "1-619-496-1150";

        fn wrapper(v: &'static str) -> Result<Session, GameError> {
            parse_variant(v.to_owned())
        }

        assert!(wrapper(v1).is_ok());
        assert!(wrapper(v2).is_ok());
        assert!(wrapper(v3).is_ok());
        assert!(wrapper(v4).is_ok());
        assert!(wrapper(v5).is_ok());
    }
}
