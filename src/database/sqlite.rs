//! # SQLite Database Applications
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use rusqlite::Connection;

use crate::database::traits::SQLiteManager;
use crate::frontend::IOMode;
use crate::game::PlayerCount;

/* CONSTANTS */

/// Environment variable containing SQLite DB path.
const SQLITE_DATABASE: &str = "SQLITE_DATABASE";

/// SQL query to (hopefully) optimize SQLite.
const OPTIMIZATION_SQL: &str = "\
    PRAGMA cache_size = 10000; \
    PRAGMA synchronous = OFF; \
    PRAGMA journal_mode = MEMORY; \
    PRAGMA temp_store = MEMORY;";

/* HELPER FUNCTIONS */

pub fn init_sqlite<G, const N: PlayerCount, const B: usize>(
    mode: IOMode,
    game: &G,
) -> Result<Connection>
where
    G: SQLiteManager<N, B>,
{
    let conn = match mode {
        IOMode::Forgetful => Connection::open(":memory:")
            .context("Failed to open in-memory SQLite database")?,
        _ => {
            let path = std::env::var(SQLITE_DATABASE).with_context(|| {
                format!("{SQLITE_DATABASE} environment variable must be set")
            })?;

            Connection::open(&path).context(format!(
                "Failed to open SQLite database at {}",
                path
            ))?
        },
    };

    conn.execute(OPTIMIZATION_SQL, [])
        .context("Failed to tune SQLite database options")?;

    let schema = game.schema();
    match mode {
        IOMode::Overwrite => {
            conn.execute(&schema.drop_table_query(), [])
                .ok();
            conn.execute(&schema.create_table_query(), [])
                .context("Failed to create SQLite table")?;
        },
        IOMode::Constructive | IOMode::Forgetful => {
            conn.execute(&schema.create_table_query(), [])
                .context("Failed to create SQLite table")?;
        },
    }

    Ok(conn)
}
