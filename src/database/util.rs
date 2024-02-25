//! # Database Utilities Module
//!
//! This module makes room for common utility routines used throughout the
//! `crate::database` module. This does not include any implementation-specific
//! behavior; in particular, each database is to own a `util` module as needed,
//! leaving this for cases where their functionality intersects.
//!
//! #### Authorship
//!
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use anyhow::Result;

use crate::database::error::DatabaseError;
use crate::database::object::schema::Attribute;

/// Verifies that adding a `new` attribute to an `existing` set of attributes
/// would not result in an invalid state for the schema who owns `existing`.
pub fn check_attribute_validity(
    existing: &Vec<Attribute>,
    new: &Attribute,
) -> Result<(), DatabaseError> {
    if new.name().is_empty() {
        Err(DatabaseError::UnnamedAttribute { table: None })
    } else if new.size() == 0 {
        Err(DatabaseError::EmptyAttribute { table: None })
    } else if existing
        .iter()
        .any(|a| a.name() == new.name())
    {
        Err(DatabaseError::RepeatedAttribute {
            name: new.name().to_string(),
            table: None,
        })
    } else {
        Ok(())
    }
}
