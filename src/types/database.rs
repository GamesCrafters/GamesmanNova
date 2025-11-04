//! # Database Models
//!
//! TODO

use crate::types::game::PlayerCount;

/* TYPE ALIASES */

/// SQL string for an INSERT query.
pub type InsertQuery = String;

/// SQL string for a SELECT query.
pub type SelectQuery = String;

/* STRUCTURES */

/// A database column within a table schema, corresponding to one attribute.
#[derive(Default, Clone)]
pub struct Column {
    pub name: String,
    pub data: String,
}

/// A database table schema containing a collection of columns (with a set
/// amount of utility entries), a table name, and a primary key specification.
pub struct Schema {
    pub columns: Vec<Column>,
    pub players: PlayerCount,
    pub table: String,
    pub key: Column,
}

/// Builder pattern for a database table schema, specifying and guaranteeing a
/// collection of different columns, a primary key, table name, and the correct
/// number of utility attributes.
pub struct SchemaBuilder {
    pub columns: Vec<Column>,
    pub players: Option<PlayerCount>,
    pub key: Option<Column>,
    pub table: String,
}
