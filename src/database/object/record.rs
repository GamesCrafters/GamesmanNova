//! # Database Record Module
//!
//! This module implements a static-size record storing the attributes
//! associated with a game position. Since it is usually unfeasible to store
//! game state hashes as primary keys, it is assumed that the DBMS which uses
//! this definition of a record handles fetching and storing through some method
//! other than storing primary keys in records.
//!
//! #### Authorship
//!
//! - Max Fierro, 11/4/2023 (maxfierro@berkeley.edu)

use std::fmt::Display;

use crate::database::object::schema::Schema;

/* DEFINITION */

/// Represents a fixed-length contiguous list of bytes with meaning. The meaning
/// of each byte is defined by the enclosed schema, while the data is referenced
/// as a pointer without a predetermined length.
pub struct Record<'a> {
    schema: Schema,
    data: &'a [u8],
}

/* IMPLEMENTATION */

impl Record<'_> {}

/* STANDARD IMPLEMENTATIONS */

impl Default for Record<'_> {
    fn default() -> Self {
        todo!()
    }
}

impl Display for Record<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
