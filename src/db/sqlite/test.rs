//! SQLite Testing Utilities Module
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use rusqlite::Connection;

use std::env;

use crate::test::TestSetting;
use crate::test::test_setting;

/* API */

/// Parses environment variables and establishes an SQLite connection to the
/// appropriate solution database.
pub fn database() -> Result<Connection> {
    let db = match test_setting()? {
        TestSetting::Correctness => Connection::open_in_memory()
            .context("Failed to establish connection to in-memory database.")?,
        TestSetting::Development => {
            let path = env::var("TEST_DATABASE")
                .context("DATABASE environment variable not set.")?;

            Connection::open(&path).context(format!(
                "Failed to initialize SQLite connection to {}",
                path
            ))?
        },
    };

    db.execute(
        "PRAGMA synchronous = OFF; \
            PRAGMA journal_mode = MEMORY; \
            PRAGMA temp_store = MEMORY;",
        [],
    )
    .context("Failed to tune SQLite database options.")?;
    Ok(db)
}
