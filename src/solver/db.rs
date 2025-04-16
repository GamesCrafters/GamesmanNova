//! # Solver Database Module
//!
//! Contains abstractions for handling database logic, including schema
//! definitions and constructors.

use anyhow::Result;
use anyhow::bail;

use crate::game::PlayerCount;
use crate::util;

/* DEFINITIONS */

/// A database column within a table schema, corresponding to one attribute.
#[derive(Default, Clone)]
pub struct Column {
    name: String,
    data: String,
}

/// Builder pattern for a database table schema, specifying and guaranteeing a
/// collection of different columns, a primary key, table name, and the correct
/// number of utility attributes.
pub struct SchemaBuilder {
    columns: Vec<Column>,
    players: Option<PlayerCount>,
    table: Option<String>,
    key: Option<Column>,
}

/// A database table schema containing a collection of columns (with a set
/// amount of utility entries), a table name, and a primary key specification.
pub struct Schema {
    columns: Vec<Column>,
    players: PlayerCount,
    table: String,
    key: Column,
}

/* QUERY UTILITIES */

impl SchemaBuilder {
    /// Initialize a builder.
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            players: None,
            table: None,
            key: None,
        }
    }

    /// Specifies the number of players this schema is built to contain.
    pub fn players(mut self, count: PlayerCount) -> Self {
        self.players = Some(count);
        self
    }

    /// Inserts a new column into the table schema.
    pub fn column(mut self, name: &str, data: &str) -> Self {
        self.columns
            .push(Column::new(name, data));

        self
    }

    /// Specifies a table name for this schema.
    pub fn table(mut self, name: &str) -> Self {
        self.table = Some(name.to_string());
        self
    }

    /// Adds a column that will be marked as primary key.
    pub fn key(mut self, name: &str, data: &str) -> Self {
        self.key = Some(Column::new(name, data));
        self
    }

    /// Checks for correctness and builds the complete schema.
    pub fn build(mut self) -> Result<Schema> {
        let players = if let Some(players) = self.players {
            players
        } else {
            bail!("Attempted to initialize schema without player count.")
        };

        let utility_cols = self.utility_columns(players);
        self.columns
            .extend_from_slice(&utility_cols);

        let table = if let Some(table) = self.table {
            table
        } else {
            bail!("Attempted to initialize schema without table.")
        };

        let key = if let Some(key) = self.key {
            key
        } else {
            bail!("Attempted to initialize schema without primary key.")
        };

        let names = self
            .columns
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<String>>();

        if let Some(name) = util::first_duplicate(&names) {
            bail!(
                "Attempted to initialize schema with duplicate name: {}",
                name
            )
        }

        if let Some(name) = self
            .columns
            .iter()
            .map(|x| x.name())
            .find(|&x| x == key.name())
        {
            bail!(
                "Attempted to initialize schema with primary key duplicate: {}",
                name,
            )
        }

        Ok(Schema {
            columns: self.columns,
            players,
            table,
            key,
        })
    }

    /* UTILITY */

    fn utility_columns(&mut self, players: PlayerCount) -> Vec<Column> {
        (0..players)
            .map(|i| Column::new(&format!("utility_{}", i), "INTEGER"))
            .collect::<Vec<Column>>()
    }
}

impl Schema {
    /// Returns an SQL 'INSERT' query string with placeholders for the values
    /// to be inserted into the schema's table.
    pub fn insert_query(&self) -> String {
        format!(
            "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT({}) DO UPDATE SET {}",
            self.table(),
            self.colnames(),
            self.placeholders(),
            self.key(),
            self.update(),
        )
    }

    /// Returns an SQL 'INSERT' query string with one placeholder for the key
    /// to be queried from the schema's table.
    pub fn select_query(&self) -> String {
        format!(
            "SELECT {} FROM {} WHERE state = ?",
            self.colnames(),
            self.table(),
        )
    }

    /// Returns an SQL 'CREATE TABLE' query that materializes a table with the
    /// columns specified in this schema.
    pub fn create_table_query(&self) -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} ({});",
            self.table(),
            self.annotations(),
        )
    }

    /// Returns the table row index where utility entries start in the schema.
    pub fn utility_index(&self) -> usize {
        self.len() - self.players
    }

    /* UTILS */

    fn len(&self) -> usize {
        self.columns.len() + 1
    }

    fn table(&self) -> &str {
        &self.table
    }

    fn key(&self) -> &str {
        self.key.name()
    }

    fn colnames(&self) -> String {
        let mut cols = Vec::new();
        cols.push(format!("\"{}\"", self.key.name()));
        cols.extend(
            self.columns
                .iter()
                .map(|s| format!("\"{}\"", s.name())),
        );

        cols.join(", ")
    }

    fn update(&self) -> String {
        self.columns
            .iter()
            .map(|x| x.name())
            .map(|col| format!("\"{}\" = excluded.\"{}\"", col, col))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn placeholders(&self) -> String {
        (0..self.len())
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn annotations(&self) -> String {
        let mut fields = Vec::new();
        fields.push(format!(
            "{} {} PRIMARY KEY",
            self.key.name(),
            self.key.datatype()
        ));

        self.columns.iter().for_each(|c| {
            fields.push(format!("{} {} NOT NULL", c.name(), c.datatype()))
        });

        fields.join(", ")
    }
}

impl Column {
    fn new(name: &str, data: &str) -> Self {
        Self {
            name: name.to_string(),
            data: data.to_string(),
        }
    }

    fn datatype(&self) -> &str {
        &self.data
    }

    fn name(&self) -> &str {
        &self.name
    }
}
