//! # Crossteaser Variant Handling Module
//!
//! This module helps parse the `Variant` string provided to the Crossteaser
//! game into parameters that can help build a game session.
//!
//! #### Authorship
//!
//! - Max Fierro, 11/5/2023 (maxfierro@berkeley.edu)
//! - YOUR NAME HERE

use super::Session;
use crate::{errors::NovaError, models::Variant};
use regex::Regex;

/* CROSSTEASER VARIANT DEFINITION */

pub const VARIANT_DEFAULT: &str = "3x3-1";
pub const VARIANT_PATTERN: &str = r"^\d+x\d+\-\d+$";
pub const VARIANT_PROTOCOL: &str = "The variant string allows users to define any size of the puzzle and the number of free slots. The string should follow the format LxW-F, with L representing the length and W representing the width of the puzzle, and F representing the number of free slots, all positive integers ";

/* API */

/// Returns a crossteaser session set up using the parameters specified by
/// `variant`. Returns a `NovaError::VariantMalformed` if the variant string
/// does not conform to the variant protocol specified, which should contain
/// useful information about why it was not parsed/accepted.
pub fn parse_variant(variant: Variant) -> Result<Session, NovaError> {
    todo!()
}

/* TESTS */

#[cfg(test)]
mod test {

    use super::*;
    use crate::games::Game;

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
        let with_none = Session::initialize(None);
        let with_default =
            Session::initialize(Some(VARIANT_DEFAULT.to_owned()));
        assert!(with_none.is_ok());
        assert!(with_default.is_ok());
    }

    #[test]
    fn no_variant_equals_default_variant() {
        let with_none = Session::initialize(None).unwrap();
        let with_default =
            Session::initialize(Some(VARIANT_DEFAULT.to_owned())).unwrap();

        // Check these two things generated identical `Session`s.
        todo!()
    }

    #[test]
    fn invalid_variants_fail_checks() {
        todo!()
    }

    #[test]
    fn valid_variants_pass_checks() {
        todo!()
    }
}
