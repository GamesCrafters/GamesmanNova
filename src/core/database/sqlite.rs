//! # SQLite Database Applications
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use rusqlite::Connection;

use std::env;

/* HELPER FUNCTIONS */

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
