//! # Developer Applications
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use rusqlite::Connection;

use std::env;
use std::fs;
use std::path::PathBuf;

use crate::model::developer::DEV_DIRECTORY;
use crate::model::developer::DIRECTORY_LOCK;
use crate::model::developer::DevelopmentData;
use crate::model::developer::TestSetting;

/// Parses environment variables and establishes an SQLite connection to the
/// global game solution testing database.
pub fn test_database() -> Result<Connection> {
    use crate::application::developer::test_setting;
    use crate::model::developer::TestSetting;
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

/// Returns the testing side effects setting as obtained from the `TEST_SETTING`
/// environment variable.
pub fn test_setting() -> Result<TestSetting> {
    if let Ok(setting) = env::var("TEST_SETTING") {
        match &setting[..] {
            "correctness" => Ok(TestSetting::Correctness),
            "development" => Ok(TestSetting::Development),
            _ => bail!("TEST_SETTING assignment '{setting}' not recognized."),
        }
    } else {
        Ok(TestSetting::Development)
    }
}

/// Returns a PathBuf corresponding to the correct subdirectory for storing
/// development `data` at a `module`-specific subdirectory, creating it in the
/// process if it does not exist.
pub fn get_directory(
    data: DevelopmentData,
    module: PathBuf,
) -> Result<PathBuf> {
    let root = find_cargo_lock_directory()
        .context("Failed to find project root directory.")?;

    let directory = root
        .join(DEV_DIRECTORY)
        .join(format!("{data}"))
        .join(module);

    let guard = {
        let _lock = DIRECTORY_LOCK.read().unwrap();
        directory.try_exists()?
    };

    if !guard {
        // Does not completely prevent multiple threads from attempting to
        // create the same directory path, but `create_dir_all` is resilient
        // to this regardless. This is only necessary for preventing race
        // conditions within `find_cargo_lock_directory`.
        let _lock = DIRECTORY_LOCK.write().unwrap();
        fs::create_dir_all(&directory)
            .context("Failed to create module subdirectory.")?;
    }

    Ok(directory)
}

/* HELPER FUNCTIONS */

/// Searches for a parent directory containing a `Cargo.lock` file.
fn find_cargo_lock_directory() -> Result<PathBuf> {
    let _lock = DIRECTORY_LOCK.read().unwrap();
    let mut cwd = env::current_dir()?;
    loop {
        let cargo_lock = cwd.join("Cargo.lock");
        if cargo_lock.try_exists()? && cargo_lock.is_file() {
            return Ok(cwd);
        }
        if let Some(parent_dir) = cwd.parent() {
            cwd = parent_dir.to_owned();
        } else {
            break;
        }
    }
    bail!("Could not find any parent directory with a Cargo.lock file.")
}
