//! # Crossteaser Variant Handling Module
//!
//! This module helps parse the `Variant` string provided to the Crossteaser
//! game into parameters that can help build a game session.
//!
//! #### Authorship
//! - Max Fierro, 11/5/2023 (maxfierro@berkeley.edu)
//! - Atharva Gupta, 11/28/2023
//! - Cindy Xu, 11/28/2023

use regex::Regex;

use crate::game::crossteaser::{Session, NAME};
use crate::game::error::GameError;

/* CROSSTEASER VARIANT DEFINITION */

pub const VARIANT_DEFAULT: &str = "3x3-1";
pub const VARIANT_PATTERN: &str = r"^\d+x\d+\-\d+$";
pub const VARIANT_PROTOCOL: &str = "The variant string allows users to define \
any size of the puzzle and the number of free slots. The string should follow \
the format LxW-F, with L representing the length and W representing the width \
of the puzzle, and F representing the number of free slots, all positive \
integers. Note that L and W must be greater than 1, or it would be possible \
for the resulting variant to not be solvable.";

/* API */

/// Returns a crossteaser session set up using the parameters specified by
/// `variant`. Returns a `GameError::VariantMalformed` if the variant string
/// does not conform to the variant protocol specified, which should contain
/// useful information about why it was not parsed/accepted.
pub fn parse_variant(variant: String) -> Result<Session, GameError> {
    check_variant_pattern(&variant)?;
    let params = parse_parameters(&variant)?;
    check_param_count(&params)?;
    check_params_are_positive(&params)?;
    Ok(Session {
        variant: Some(variant),
        length: params[0],
        width: params[1],
        free: params[2],
    })
}

/* VARIANT STRING VERIFICATION */

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

fn parse_parameters(variant: &str) -> Result<Vec<u64>, GameError> {
    variant
        .split(['x', '-'])
        .map(|int_string| {
            int_string
                .parse::<u64>()
                .map_err(|e| GameError::VariantMalformed {
                    game_name: NAME,
                    hint: format!("{}", e.to_string()),
                })
        })
        .collect()
}

fn check_param_count(params: &Vec<u64>) -> Result<(), GameError> {
    if params.len() != 3 {
        Err(GameError::VariantMalformed {
            game_name: NAME,
            hint: "String needs to have exactly 3 dash-separated integers."
                .to_owned(),
        })
    } else {
        Ok(())
    }
}

fn check_params_are_positive(params: &Vec<u64>) -> Result<(), GameError> {
    if params.iter().any(|&x| x <= 0) {
        Err(GameError::VariantMalformed {
            game_name: NAME,
            hint: "All integers in the string must be positive.".to_owned(),
        })
    } else if params
        .iter()
        .take(2)
        .any(|&x| x <= 1)
    {
        Err(GameError::VariantMalformed {
            game_name: NAME,
            hint: "L and W must both be strictly greater than 1.".to_owned(),
        })
    } else {
        Ok(())
    }
}

/* TESTS */

#[cfg(test)]
mod test {

    use super::*;
    use crate::game::Game;

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
        let with_none = Session::new(None);
        let with_default = Session::new(Some(VARIANT_DEFAULT.to_owned()));

        assert!(with_none.is_ok());
        assert!(with_default.is_ok());
    }

    #[test]
    fn invalid_variants_fail_checks() {
        let some_variant_1 = Session::new(Some("None".to_owned()));
        let some_variant_2 = Session::new(Some("x4-".to_owned()));
        let some_variant_3 = Session::new(Some("-".to_owned()));
        let some_variant_4 = Session::new(Some("1x2-5".to_owned()));
        let some_variant_5 = Session::new(Some("0x2-5".to_owned()));
        let some_variant_6 = Session::new(Some("1x1-1".to_owned()));
        let some_variant_7 = Session::new(Some("8x2.6-5".to_owned()));
        let some_variant_8 = Session::new(Some("3x4-0".to_owned()));

        assert!(some_variant_1.is_err());
        assert!(some_variant_2.is_err());
        assert!(some_variant_3.is_err());
        assert!(some_variant_4.is_err());
        assert!(some_variant_5.is_err());
        assert!(some_variant_6.is_err());
        assert!(some_variant_7.is_err());
        assert!(some_variant_8.is_err());
    }

    #[test]
    fn valid_variants_pass_checks() {
        let some_variant_1 = Session::new(Some("4x3-2".to_owned()));
        let some_variant_2 = Session::new(Some("5x4-2".to_owned()));
        let some_variant_3 = Session::new(Some("2x4-1".to_owned()));
        let some_variant_4 = Session::new(Some("4x2-1".to_owned()));

        assert!(some_variant_1.is_ok());
        assert!(some_variant_2.is_ok());
        assert!(some_variant_3.is_ok());
        assert!(some_variant_4.is_ok());
    }
}
