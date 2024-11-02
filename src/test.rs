//! # Test Utilities Module
//!
//! This module provides utility functions which tests or other utilities depend
//! on throughout the project, and defines the structure of the development
//! resources that are generated through tests.

use anyhow::{anyhow, bail, Context, Result};
use strum_macros::Display;

use std::{
    env, fs,
    path::{self, PathBuf},
    sync::RwLock,
};

/* CONSTANTS */

/// Global lock for creating development data directories. Since `cargo test`
/// executes tests in parallel, this helps prevent test flakiness by avoiding
/// race conditions when creating directories (specific files are still
/// susceptible, but they have no specific structure).
pub static DIRECTORY_LOCK: RwLock<()> = RwLock::new(());

/// The name of the global directory at the project root used for generated
/// development data. This directory is not shipped with release builds.
const DEV_DIRECTORY: &str = "dev";

/* DEFINITIONS */

/// Specifies directories for different kinds of data generated for development
/// purposes, which should not be distributed.
#[derive(Display)]
#[strum(serialize_all = "kebab-case")]
pub enum DevelopmentData {
    Visuals,
}

/// Specifies the level of side effects to generate during testing. This
/// corresponds to the `TEST_SETTING` environment variable.
pub enum TestSetting {
    Correctness,
    Development,
}

/* UTILITY FUNCTIONS */

/// Returns the testing side effects setting as obtained from the `TEST_SETTING`
/// environment variable.
pub fn test_setting() -> Result<TestSetting> {
    if let Ok(setting) = env::var("TEST_SETTING") {
        match &setting[..] {
            "0" => Ok(TestSetting::Correctness),
            "1" => Ok(TestSetting::Development),
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
) -> Result<path::PathBuf> {
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
fn find_cargo_lock_directory() -> Result<path::PathBuf> {
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
