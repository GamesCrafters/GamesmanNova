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
use crate::database::object::schema::Datatype;

/// Verifies that adding a `new` attribute to an `existing` set of attributes
/// would not result in an invalid state for the schema who owns `existing`,
/// and that the added attribute does not break any datatype sizing rules.
pub fn check_attribute_validity<'a>(
    existing: &Vec<Attribute>,
    new: &Attribute,
) -> Result<(), DatabaseError<'a>> {
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
        check_datatype_validity(new)?;
        Ok(())
    }
}

fn check_datatype_validity<'a>(
    new: &Attribute,
) -> Result<(), DatabaseError<'a>> {
    let s = new.size();
    if match new.datatype() {
        Datatype::CSTR => s % 8 != 0,
        Datatype::DPFP => s != 64,
        Datatype::SPFP => s != 32,
        Datatype::SINT => s < 2,
        Datatype::ENUM { map } => s > 8,
        Datatype::UINT => unreachable!("UINTs can have any size."),
    } {
        Err(DatabaseError::InvalidSize {
            size: new.size(),
            name: new.name().to_string(),
            data: new.datatype(),
            table: None,
        })
    } else {
        Ok(())
    }
}

/* UTILITY IMPLEMENTATIONS */

impl ToString for Datatype<'_> {
    fn to_string(&self) -> String {
        match self {
            Datatype::DPFP => "Double-Precision Floating Point".to_string(),
            Datatype::SPFP => "Single-Precision Floating Point".to_string(),
            Datatype::CSTR => "C-Style ASCII String".to_string(),
            Datatype::ENUM { map } => "Enumeration".to_string(),
            Datatype::UINT => "Unsigned Integer".to_string(),
            Datatype::SINT => "Signed Integer".to_string(),
        }
    }
}
