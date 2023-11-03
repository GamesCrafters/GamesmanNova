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
use crate::models::Variant;
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
    let params = parse_parameters(variant)?;
    check_param_count(&params)?;
    check_params_are_positive(&params)?;
    Ok(Session {
        variant: Some(variant),
        players: params[0],
        from: params[1],
        by: Vec::from(&params[2..]),
    })
}

/* VARIANT STRING VERIFICATION */

fn parse_parameters(variant: Variant) -> Result<Vec<u64>, VariantError>
{
    let params = variant
        .split('-')
        .map(|int_string| {
            let param = int_string.parse::<u64>();
            match param {
                Ok(integer) => integer,
                Err(e) => VariantError::Malformed {
                    game_name: NAME.to_owned(),
                    message: format!("{}", e.to_string()),
                },
            }
        })
        .collect::<Vec<u64>>();
    Ok(params)
}

fn check_variant_pattern(variant: &Variant) -> Result<(), VariantError>
{
    let re = Regex::new(VARIANT_PATTERN).unwrap();
    if !re.is_match(&variant) {
        return VariantError::Malformed {
            game_name: NAME.to_owned(),
            message: format!(
                "String does not match the pattern '{}'.",
                VARIANT_PATTERN
            ),
        }
    }
    Ok(())
}

fn check_param_count(params: &Vec<u64>) -> Result<(), VariantError>
{
    if params.len() < 3 {
        return VariantError::Malformed {
            game_name: NAME.to_owned(),
            message: "String needs to have at least 3 dash-separated integers."
                .to_owned(),
        }
    }
    Ok(())
}

fn check_params_are_positive(params: &Vec<u64>) -> Result<(), VariantError>
{
    if params.iter().any(|&x| x <= 0) {
        return VariantError::Malformed {
            game_name: NAME.to_owned(),
            message: "All integers in the string must be positive.".to_owned(),
        }
    }
    Ok(())
}
