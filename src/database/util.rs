//! # Database Utilities Module
//!
//! This module makes room for common utility routines used throughout the
//! `crate::database` module. This does not include any implementation-specific
//! behavior; in particular, each database is to own a `util` module as needed,
//! leaving this for cases where their functionality intersects.
//!
//! #### Authorship
//! - Max Fierro, 2/24/2024 (maxfierro@berkeley.edu)

use std::fmt::Display;

use anyhow::Result;

use crate::database::error::DatabaseError;
use crate::database::Attribute;
use crate::database::Datatype;
use crate::database::Schema;
use crate::solver::RecordType;

/* DEFINITIONS */

/// Builder pattern intermediary for constructing a schema declaratively out of
/// provided attributes. This is here to help ensure schemas are not changed
/// accidentally after being instantiated.
pub struct SchemaBuilder {
    attributes: Vec<Attribute>,
    record: Option<RecordType>,
    size: usize,
}

/// Iterator over borrows of the attributes that form a database table schema.
pub struct SchemaIterator<'a> {
    schema: &'a Schema,
    index: usize,
}

/* BUILDER IMPLEMENTATION */

impl SchemaBuilder {
    /// Returns a new instance of a `SchemaBuilder`, which can be used to
    /// declaratively construct a new record `Schema`.
    pub fn new() -> Self {
        SchemaBuilder {
            attributes: Vec::new(),
            record: None,
            size: 0,
        }
    }

    /// Associates `attr` to the schema under construction. Returns an error
    /// if adding `attr` to the schema would result in an invalid state.
    pub fn add(mut self, attr: Attribute) -> Result<Self> {
        self.check_attribute_validity(&attr)?;
        self.size += attr.size();
        Ok(self)
    }

    /// Associates a known `record` type to the schema under construction.
    pub fn of(mut self, record: RecordType) -> Self {
        self.record = Some(record);
        self
    }

    /// Constructs the schema using the current state of the `SchemaBuilder`.
    pub fn build(self) -> Schema {
        Schema {
            attributes: self.attributes,
            record: self.record,
            size: self.size,
        }
    }

    /* VERIFICATION METHODS */

    /// Verifies that adding a `new` attribute to tje existing set of attributes
    /// would not result in an invalid state for the schema under construction,
    /// and that the added attribute does not break any datatype sizing rules.
    fn check_attribute_validity(
        &self,
        new: &Attribute,
    ) -> Result<(), DatabaseError> {
        if new.name().is_empty() {
            Err(DatabaseError::UnnamedAttribute { table: None })
        } else if new.size() == 0 {
            Err(DatabaseError::EmptyAttribute { table: None })
        } else if self
            .attributes
            .iter()
            .any(|a| a.name() == new.name())
        {
            Err(DatabaseError::RepeatedAttribute {
                name: new.name().to_string(),
                table: None,
            })
        } else {
            Self::check_datatype_validity(new)?;
            Ok(())
        }
    }

    /// Verifies that the datatype in an attribute is coherent with its
    /// indicated size, which is specific to each valid datatype.
    fn check_datatype_validity(new: &Attribute) -> Result<(), DatabaseError> {
        let s = new.size();
        if match new.datatype() {
            Datatype::SINT => s < 2,
            Datatype::BOOL => s != 1,
            Datatype::SPFP => s != 32,
            Datatype::DPFP => s != 64,
            Datatype::CSTR => s % 8 != 0,
            Datatype::UINT | Datatype::ENUM => s == 0,
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
}

/* UTILITY IMPLEMENTATIONS */

impl Display for Datatype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            Datatype::DPFP => "Double-Precision Floating Point",
            Datatype::SPFP => "Single-Precision Floating Point",
            Datatype::CSTR => "C-Style ASCII String",
            Datatype::UINT => "Unsigned Integer",
            Datatype::SINT => "Signed Integer",
            Datatype::ENUM => "Enumeration",
            Datatype::BOOL => "Boolean",
        };
        write!(f, "{}", content)
    }
}

impl<'a> IntoIterator for &'a Schema {
    type IntoIter = SchemaIterator<'a>;

    type Item = &'a Attribute;

    fn into_iter(self) -> Self::IntoIter {
        SchemaIterator {
            schema: self,
            index: 0,
        }
    }
}

impl<'a> Iterator for SchemaIterator<'a> {
    type Item = &'a Attribute;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.schema
            .attributes
            .get(self.index - 1)
    }
}
