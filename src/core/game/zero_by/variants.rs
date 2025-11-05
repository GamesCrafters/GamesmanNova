//! # Zero-By Variant Implementation Utilities
//!
//! TODO

use anyhow::Result;
use bitvec::array::BitArray;
use bitvec::field::BitField;
use bitvec::order::Msb0;
use regex::Regex;

use crate::core::database::SchemaBuilder;
use crate::core::error::GameError;
use crate::core::game::Player;
use crate::core::game::util::min_ubits;
use crate::core::game::zero_by::NAME;
use crate::core::game::zero_by::Session;
use crate::core::game::zero_by::VARIANT_PATTERN;

/* API */

/// Returns a zero-by game session set up using the parameters specified by
/// `variant`. Returns a `GameError::VariantMalformed` if the variant string
/// does not conform to the variant protocol.
pub fn parse_variant(variant: String) -> Result<Session> {
    check_variant_pattern(&variant)?;
    let params = parse_parameters(&variant)?;
    check_param_count(&params)?;
    check_params_are_positive(&params)?;
    let players = parse_player_count(&params)?;

    let start_elems = params[1];
    let mut start_state: BitArray<_, Msb0> = BitArray::ZERO;
    let player_bits = min_ubits(players as u128);
    start_state[..player_bits].store_be(Player::default());
    start_state[player_bits..].store_be(start_elems);

    let table = format!("{}_{}", NAME, variant);
    let schema = SchemaBuilder::new(&table)
        .players(players)
        .key("state", "INTEGER")
        .column("remoteness", "INTEGER")
        .column("player", "INTEGER")
        .build()?;

    Ok(Session {
        start_state: start_state.data,
        start_elems,
        player_bits,
        players,
        schema,
        name: variant,
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
                    game: NAME,
                    hint: e.to_string(),
                })
        })
        .collect();
    params
}

fn check_variant_pattern(variant: &str) -> Result<(), GameError> {
    let re = Regex::new(VARIANT_PATTERN).unwrap();
    if !re.is_match(variant) {
        Err(GameError::VariantMalformed {
            game: NAME,
            hint: format!(
                "String does not match the pattern '{VARIANT_PATTERN}'.",
            ),
        })
    } else {
        Ok(())
    }
}

fn check_param_count(params: &[u64]) -> Result<(), GameError> {
    if params.len() < 3 {
        Err(GameError::VariantMalformed {
            game: NAME,
            hint: "String needs to have at least 3 dash-separated integers."
                .to_string(),
        })
    } else {
        Ok(())
    }
}

fn check_params_are_positive(params: &[u64]) -> Result<(), GameError> {
    if params.contains(&0) {
        Err(GameError::VariantMalformed {
            game: NAME,
            hint: "All integers in the string must be positive.".to_string(),
        })
    } else {
        Ok(())
    }
}

fn parse_player_count(params: &[u64]) -> Result<Player, GameError> {
    if params[0] > (Player::MAX as u64) {
        Err(GameError::VariantMalformed {
            game: NAME,
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

    use crate::core::game::zero_by::VARIANT_DEFAULT;
    use crate::traits::game::Variable;

    use super::*;

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
        let _ = Session::default();
        let with_default = Session::variant(VARIANT_DEFAULT.to_owned());
        assert!(with_default.is_ok());
    }

    #[test]
    fn no_variant_equals_default_variant() -> Result<()> {
        let with_none = Session::default();
        let with_default = Session::variant(VARIANT_DEFAULT.to_owned())?;
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

        fn wrapper(v: &'static str) -> Result<Session> {
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

        fn wrapper(v: &'static str) -> Result<Session> {
            parse_variant(v.to_owned())
        }

        assert!(wrapper(v1).is_ok());
        assert!(wrapper(v2).is_ok());
        assert!(wrapper(v3).is_ok());
        assert!(wrapper(v4).is_ok());
        assert!(wrapper(v5).is_ok());
    }
}
