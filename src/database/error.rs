//! # Database Error Module
//!
//! This module defines possible errors that could happen while a database is
//! being executed. These errors should regard only the top-level module, not
//! any specific database implementation (in a sense, providing an abstraction
//! under which all database implementations' errors can be grouped into).
//!
//! #### Authorship
//!
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use std::{error::Error, fmt};

/* ERROR WRAPPER */

/// Wrapper for all database-interface-related errors that could happen during
/// runtime. This pertains specifically to the elements of the `crate::database`
/// module, and the interfaces it provides (not specific implementations). Many
/// of the variants of this wrapper include a field for a schema; this allows
/// consumers to provide specific errors when deserializing persisted schemas.
#[derive(Debug)]
pub enum DatabaseError {
    /// An error to indicate that there was an attempt to construct a schema
    /// containing two attributes with the same name.
    RepeatedAttribute { name: String, table: Option<String> },

    /// An error to indicate that there was an attempt to construct a schema
    /// containing an attribute without a name.
    UnnamedAttribute { table: Option<String> },

    /// An error to indicate that there was an attempt to construct a schema
    /// containing an attribute of zero size.
    EmptyAttribute { table: Option<String> },
}

impl Error for DatabaseError {}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RepeatedAttribute { name, table } => {
                if let Some(t) = table {
                    write!(
                        f,
                        "The attribute name '{}' was observed more than once \
                        while deserializing the schema of table '{}'.",
                        name, t,
                    )
                } else {
                    write!(
                        f,
                        "Attempted to build a schema with more than one \
                        attribute named '{}'.",
                        name,
                    )
                }
            },
            Self::UnnamedAttribute { table } => {
                if let Some(t) = table {
                    write!(
                        f,
                        "Encountered empty attribute name while deserializing \
                        the schema of table '{}'.",
                        t,
                    )
                } else {
                    write!(
                        f,
                        "Attempted to build a schema containing an attribute \
                        with an empty name.",
                    )
                }
            },
            Self::EmptyAttribute { table } => {
                if let Some(t) = table {
                    write!(
                        f,
                        "Encountered zero-sized attribute while deserializing \
                        the schema of table '{}'.",
                        t,
                    )
                } else {
                    write!(
                        f,
                        "Attempted to build a schema containing an attribute \
                        of size zero.",
                    )
                }
            },
        }
    }
}
