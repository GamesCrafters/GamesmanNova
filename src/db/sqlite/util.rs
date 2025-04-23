//! # SQLite Database Utilities
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use rusqlite::Connection;

use std::collections::HashSet;
use std::env;
use std::hash::Hash;

/* API */

/// Parses environment variables and establishes an SQLite connection to the
/// global game solution database.
pub fn database() -> Result<Connection> {
    let path = env::var("SQLITE_DATABASE")
        .context("SQLITE_DATABASE environment variable not set.")?;

    let db = Connection::open(&path).context(format!(
        "Failed to initialize SQLite connection to {}",
        path
    ))?;

    db.execute(
        "PRAGMA cache_size = 10000; \
            PRAGMA synchronous = OFF; \
            PRAGMA journal_mode = MEMORY; \
            PRAGMA temp_store = MEMORY;",
        [],
    )
    .context("Failed to tune SQLite database options.")?;

    Ok(db)
}

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
