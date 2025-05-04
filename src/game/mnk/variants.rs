//! # MNK Variant Handling Module
//!
//! This module helps parse the variant string provided to the m,n,k game
//! into parameters that can help build a game session.

use anyhow::Result;
use bitvec::array::BitArray;
use bitvec::field::BitField;
use bitvec::order::Msb0;
use regex::Regex;

use crate::game::error::GameError;
use crate::game::mnk::MAX_BOARD_SIDE;
use crate::game::mnk::NAME;
use crate::game::mnk::Session;
use crate::solver::db::SchemaBuilder;

/* MNK VARIANT ENCODING */

pub const VARIANT_DEFAULT: &str = "3-3-3";
pub const VARIANT_PATTERN: &str =
    r"^([1-9][0-9]*)-([1-9][0-9]*)-([1-9][0-9]*)$";

pub const VARIANT_PROTOCOL: &str = "Three nonzero positive integers separated \
by dashes, in the form M-N-K. Here, M and N are the dimensions of the board, \
and K is the number of symbols which, when placed in a row, result in a win.";

/* API */

/// Returns an m,n,k-game session set up using the parameters specified by
/// `variant`.
pub fn parse_variant(variant: String) -> Result<Session> {
    check_variant_pattern(&variant)?;
    let params = parse_parameters(&variant)?;
    check_param_count(&params)?;
    check_params_are_positive(&params)?;
    check_dimensionality(&params)?;

    let table = format!("{}_{}", NAME, variant);
    let schema = SchemaBuilder::new(&table)
        .players(2)
        .key("state", "INTEGER")
        .column("remoteness", "INTEGER")
        .column("player", "INTEGER")
        .column("orbit_rep", "INTEGER")
        .build()?;

    let mut state = BitArray::<_, Msb0>::ZERO;
    state[..1].store_be(1);

    Ok(Session {
        schema,
        start: state.data,
        m: params[0],
        n: params[1],
        k: params[2],
    })
}

/* HELPERS */

fn parse_parameters(variant: &str) -> Result<Vec<usize>, GameError> {
    let params: Result<Vec<usize>, _> = variant
        .split('-')
        .map(|int_string| {
            int_string
                .parse::<usize>()
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

fn check_param_count(params: &[usize]) -> Result<(), GameError> {
    if params.len() != 3 {
        Err(GameError::VariantMalformed {
            game: NAME,
            hint: "String needs to have exactly 3 dash-separated integers."
                .to_string(),
        })
    } else {
        Ok(())
    }
}

fn check_params_are_positive(params: &[usize]) -> Result<(), GameError> {
    if params.iter().any(|&x| x == 0) {
        Err(GameError::VariantMalformed {
            game: NAME,
            hint: "All integers in the string must be positive.".to_string(),
        })
    } else {
        Ok(())
    }
}

fn check_dimensionality(params: &[usize]) -> Result<(), GameError> {
    if params[0] > MAX_BOARD_SIDE {
        return Err(GameError::VariantMalformed {
            game: NAME,
            hint: format!(
                "Dimension 'm = {}' is too large. Maximum is {}.",
                params[0], MAX_BOARD_SIDE,
            ),
        });
    }

    if params[1] > MAX_BOARD_SIDE {
        return Err(GameError::VariantMalformed {
            game: NAME,
            hint: format!(
                "Dimension 'n = {}' is too large. Maximum is {}.",
                params[1], MAX_BOARD_SIDE,
            ),
        });
    }

    // State encodings are 64 bits. One bit is used to store turns efficiently.
    // The remaining 63 bits are used to store board slots. Each slot needs 2
    // bits. So, 2 * m * n cannot be  greater than 63.
    if 2 * params[0] * params[1] > 63 {
        return Err(GameError::VariantMalformed {
            game: NAME,
            hint: format!(
                "Dimensions are too large for state encoding scheme. Ensure \
                that (m * n * 2) < 63. Currently, it is {}.",
                params[0] * params[1],
            ),
        });
    }

    Ok(())
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
        let _ = Session::default();
        let with_default = Session::variant(VARIANT_DEFAULT.to_owned());
        assert!(with_default.is_ok());
    }

    #[test]
    fn no_variant_equals_default_variant() -> Result<()> {
        let with_none = Session::default();
        let with_default = Session::variant(VARIANT_DEFAULT.to_owned())?;
        assert_eq!(with_none.start, with_default.start);
        assert_eq!(with_none.m, with_default.m);
        assert_eq!(with_none.n, with_default.n);
        assert_eq!(with_none.k, with_default.k);
        Ok(())
    }

    #[test]
    fn invalid_variants_fail_checks() {
        let v1 = "23-34-0-";
        let v2 = "two-three-five";
        let v3 = "234572342-2345";
        let v4 = "34--236-3";
        let v5 = "364";
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
        let v1 = "4-4-4";
        let v2 = "3-3-3";
        let v3 = "2-4-2";
        let v4 = "3-2-3";

        fn wrapper(v: &'static str) -> Result<Session> {
            parse_variant(v.to_owned())
        }

        assert!(wrapper(v1).is_ok());
        assert!(wrapper(v2).is_ok());
        assert!(wrapper(v3).is_ok());
        assert!(wrapper(v4).is_ok());
    }

    #[test]
    fn too_high_dimensional_variant_fails_checks() {
        let v1 = "8-4-4";
        let v2 = "7-9-3";
        let v3 = "7-5-5";
        let v4 = "4-10-2";

        fn wrapper(v: &'static str) -> Result<Session> {
            parse_variant(v.to_owned())
        }

        assert!(wrapper(v1).is_err());
        assert!(wrapper(v2).is_err());
        assert!(wrapper(v3).is_err());
        assert!(wrapper(v4).is_err());
    }
}
