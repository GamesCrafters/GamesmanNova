//! # Crossteaser Variant Handling Module
//!
//! This module helps parse the `Variant` string provided to the Crossteaser
//! game into parameters that can help build a game session.
//!
//! #### Authorship
//!
//! - Max Fierro, 11/2/2023 (maxfierro@berkeley.edu)

use super::Session;
use crate::{errors::NovaError, models::Variant};

/* CROSSTEASER VARIANT DEFINITION */

pub const VARIANT_DEFAULT: &str = "PLACEHOLDER";
pub const VARIANT_PATTERN: &str = r"PLACEHOLDER";
pub const VARIANT_PROTOCOL: &str = "PLACEHOLDER";

/* API */

/// Returns a crossteaser session set up using the parameters specified by
/// `variant`. Returns a `NovaError::VariantMalformed` if the variant string
/// does not conform to the variant protocol specified.
pub fn parse_variant(variant: Variant) -> Result<Session, NovaError> {
    todo!()
}
