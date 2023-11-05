//! # Zero-By Variant Handling Module
//!
//! This module helps parse the `Variant` string provided to the Zero-By game
//! into parameters that can help build a game session.
//!
//! #### Authorship
//!
//! - Max Fierro, 11/2/2023 (maxfierro@berkeley.edu)

use super::{Session, NAME};
use crate::errors::VariantError;
use crate::models::{Player, Variant};
use regex::Regex;

/* ZERO-BY VARIANT DEFINITION */

pub const VARIANT_DEFAULT: &str = "2-10-1-2";
pub const VARIANT_PATTERN: &str = r"^[1-9]\d*(?:-[1-9]\d*)+$";
pub const VARIANT_PROTOCOL: &str =
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
/// `variant`. Returns a `VariantError::Malformed` if the variant string does
/// not conform to the variant protocol.
pub fn parse_variant(variant: Variant) -> Result<Session, VariantError>
{
    check_variant_pattern(&variant)?;
    let params = parse_parameters(&variant)?;
    check_param_count(&params)?;
    check_params_are_positive(&params)?;
    Ok(Session {
        variant: Some(variant),
        players: parse_player_count(&params)?,
        from: params[1],
        by: Vec::from(&params[2..]),
    })
}

/* VARIANT STRING VERIFICATION */

fn parse_parameters(variant: &str) -> Result<Vec<u64>, VariantError>
{
    let params: Result<Vec<u64>, _> = variant
        .split('-')
        .map(|int_string| {
            int_string
                .parse::<u64>()
                .map_err(|e| VariantError::Malformed {
                    game_name: NAME.to_owned(),
                    message: format!("{}", e.to_string()),
                })
        })
        .collect();
    params
}

fn check_variant_pattern(variant: &Variant) -> Result<(), VariantError>
{
    let re = Regex::new(VARIANT_PATTERN).unwrap();
    if !re.is_match(&variant) {
        Err(VariantError::Malformed {
            game_name: NAME.to_owned(),
            message: format!(
                "String does not match the pattern '{}'.",
                VARIANT_PATTERN
            ),
        })
    } else {
        Ok(())
    }
}

fn check_param_count(params: &Vec<u64>) -> Result<(), VariantError>
{
    if params.len() < 3 {
        Err(VariantError::Malformed {
            game_name: NAME.to_owned(),
            message: "String needs to have at least 3 dash-separated integers."
                .to_owned(),
        })
    } else {
        Ok(())
    }
}

fn check_params_are_positive(params: &Vec<u64>) -> Result<(), VariantError>
{
    if params.iter().any(|&x| x <= 0) {
        Err(VariantError::Malformed {
            game_name: NAME.to_owned(),
            message: "All integers in the string must be positive.".to_owned(),
        })
    } else {
        Ok(())
    }
}

fn parse_player_count(params: &Vec<u64>) -> Result<Player, VariantError>
{
    if params[0] > Player::MAX.into() {
        Err(VariantError::Malformed {
            game_name: NAME.to_owned(),
            message: format!(
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
mod test
{
    use super::*;
    use crate::games::Game;

    #[test]
    fn variant_pattern_is_valid_regex()
    {
        assert!(Regex::new(VARIANT_PATTERN).is_ok());
    }

    #[test]
    fn default_variant_matches_variant_pattern()
    {
        let re = Regex::new(VARIANT_PATTERN).unwrap();
        assert!(re.is_match(VARIANT_DEFAULT));
    }

    #[test]
    fn initialization_success_with_no_variant()
    {
        let with_none = Session::initialize(None);
        let with_default =
            Session::initialize(Some(VARIANT_DEFAULT.to_owned()));
        assert!(with_none.is_ok());
        assert!(with_default.is_ok());
    }

    #[test]
    fn no_variant_equals_default_variant()
    {
        let with_none = Session::initialize(None).unwrap();
        let with_default =
            Session::initialize(Some(VARIANT_DEFAULT.to_owned())).unwrap();
        assert_eq!(with_none.variant, with_default.variant);
        assert_eq!(with_none.from, with_default.from);
        assert_eq!(with_none.by, with_default.by);
    }

    #[test]
    fn invalid_variants_fail_checks()
    {
        assert!(parse_variant("23-34-0-23".to_owned()).is_err());
        assert!(parse_variant("two-three-five".to_owned()).is_err());
        assert!(parse_variant("234572342-2345".to_owned()).is_err());
        assert!(parse_variant("34-236--8-6-3".to_owned()).is_err());
        assert!(parse_variant("0-12-234-364".to_owned()).is_err());
        assert!(parse_variant("-234-256".to_owned()).is_err());
    }

    #[test]
    fn valid_variants_pass_checks()
    {
        assert!(parse_variant("5-1000-8-23-63-7".to_owned()).is_ok());
        assert!(parse_variant("1-1-1".to_owned()).is_ok());
        assert!(parse_variant("34-23623-8-6-3".to_owned()).is_ok());
        assert!(parse_variant("5-2-8-23".to_owned()).is_ok());
        assert!(parse_variant("1-619-496-1150".to_owned()).is_ok());
    }
}
