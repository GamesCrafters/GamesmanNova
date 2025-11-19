//! # Database Implementations
//!
//! TODO

use anyhow::Result;
use anyhow::bail;

use std::collections::HashSet;
use std::hash::Hash;

use crate::database::traits::ClassicUtilityRecord;
use crate::database::traits::IntegerUtilityRecord;
use crate::database::traits::PlayerRecord;
use crate::database::traits::PuzzleUtilityRecord;
use crate::database::traits::SimpleUtilityRecord;
use crate::game::IUtility;
use crate::game::PlayerCount;
use crate::game::SUtility;

/* SUBMODULES */

pub mod traits;
pub mod sqlite;
pub mod sled;
pub mod util;

/* API STRUCTURES */

/// A database table schema containing a collection of columns (with a set
/// amount of utility entries), a table name, and a primary key specification.
#[derive(Clone)]
pub struct Schema {
    columns: Vec<Column>,
    players: PlayerCount,
    table: String,
    key: Column,
}

/// Builder pattern for a database table schema, specifying and guaranteeing a
/// collection of different columns, a primary key, table name, and the correct
/// number of utility attributes.
pub struct SchemaBuilder {
    columns: Vec<Column>,
    players: Option<PlayerCount>,
    key: Option<Column>,
    table: String,
}

/// A database column within a table schema, corresponding to one attribute.
#[derive(Default, Clone)]
pub struct Column {
    name: String,
    data: String,
}

/* IMPLEMENTATIONS */

impl Column {
    pub fn new(name: &str, data: &str) -> Self {
        Self {
            name: sqlize(name),
            data: sqlize(data),
        }
    }

    fn datatype(&self) -> &str {
        &self.data
    }

    fn name(&self) -> &str {
        &self.name
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

    /// Returns an SQL 'DROP TABLE' query that deletes the table that belongs
    /// to this schema.
    pub fn drop_table_query(&self) -> String {
        format!("DROP TABLE IF EXISTS {};", self.table())
    }

    /// Returns the table row index where utility entries start in the schema.
    pub fn utility_index(&self) -> usize {
        self.len() - self.players
    }

    /* UTILITY */

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

impl SchemaBuilder {
    /// Initialize a schema builder for a table with the provided `name`.
    pub fn new(table: &str) -> Self {
        Self {
            table: sqlize(table),
            columns: Vec::new(),
            players: None,
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

        if let Some(name) = first_duplicate(&names) {
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
            table: self.table,
            players,
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

/* IMPL TRAIT FOR TYPE */

impl<R, const N: PlayerCount> IntegerUtilityRecord<N> for R
where
    R: SimpleUtilityRecord<N>,
{
    fn set_utility(&mut self, iutility: [IUtility; N]) -> Result<&mut Self> {
        let mut sutility = [SUtility::Lose; N];
        for (i, u) in sutility.iter_mut().enumerate() {
            *u = iutility[i].try_into()?;
        }

        self.set_utility(sutility)
    }

    fn get_utility(&self) -> [IUtility; N] {
        let sutility = self.get_utility();
        let mut iutility = [0; N];
        iutility
            .iter_mut()
            .enumerate()
            .for_each(|(i, u)| *u = IUtility::from(sutility[i]) - 1);

        iutility
    }
}

impl<R> SimpleUtilityRecord<2> for R
where
    R: ClassicUtilityRecord,
    R: PlayerRecord,
{
    fn set_utility(&mut self, value: [SUtility; 2]) -> Result<&mut Self> {
        let turn = self.get_player();
        self.set_utility(value[turn])
    }

    fn get_utility(&self) -> [SUtility; 2] {
        let mut sutility = [SUtility::Tie; 2];
        let utility = self.get_utility();
        let turn = self.get_player();
        let them = (turn + 1) % 2;
        sutility[them] = !utility;
        sutility[turn] = utility;
        sutility
    }
}

impl<R> SimpleUtilityRecord<1> for R
where
    R: PuzzleUtilityRecord,
{
    fn set_utility(&mut self, value: [SUtility; 1]) -> Result<&mut Self> {
        self.set_utility(value[0])
    }

    fn get_utility(&self) -> [SUtility; 1] {
        [self.get_utility()]
    }
}

/* HELPER FUNCTIONS */

/// Transform input string into a valid SQL identifier.
pub fn sqlize(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i == 0 {
            if ch.is_ascii_alphabetic() || ch == '_' {
                out.push(ch);
            } else {
                out.push('_');
            }
        } else if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }

    out
}

/// Returns the first duplicate found in `vec`.
pub fn first_duplicate<T: Eq + Hash + Clone>(vec: &[T]) -> Option<T> {
    let mut seen = HashSet::new();
    for item in vec {
        if !seen.insert(item) {
            return Some(item.clone());
        }
    }
    None
}

/* TESTS */

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn builder_fails_if_columns_repeat() -> Result<()> {
        let b1 = SchemaBuilder::new("example")
            .column("field", "INTEGER")
            .column("field", "FLOAT")
            .key("key", "INTEGER")
            .build();

        assert!(b1.is_err());
        let b2 = SchemaBuilder::new("example")
            .column("field", "INTEGER")
            .key("field", "INTEGER")
            .build();

        assert!(b2.is_err());
        Ok(())
    }

    #[test]
    fn builder_fails_if_elements_missing() -> Result<()> {
        let key_missing = SchemaBuilder::new("example")
            .column("field", "INTEGER")
            .build();

        assert!(key_missing.is_err());
        Ok(())
    }
}
