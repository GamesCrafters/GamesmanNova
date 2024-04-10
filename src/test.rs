//! # Test Utilities Module
//!
//! This module provides utility functions which tests or other utilities depend
//! on throughout the project, and defines the structure of the development
//! resources that are generated through tests.
//!
//! #### Authorship
//! - Max Fierro, 4/09/2024 (maxfierro@berkeley.edu)

use anyhow::{anyhow, Context, Result};
use strum_macros::Display;

use std::{env, fs, path};

/* DATA RESOURCES DIRECTORY */

const DATA_DIRECTORY: &str = "data";

/// Specifies directories for different kinds of data generated for development
/// purposes, which should not be distributed.
#[derive(Display)]
#[strum(serialize_all = "kebab-case")]
pub enum TestData {
    Visuals,
}

/* UTILITY FUNCTIONS */

/// Returns a PathBuf corresponding to the correct subdirectory for storing
/// development `data`, creating it in the process if it does not exist.
pub fn get_directory(data: TestData) -> Result<path::PathBuf> {
    let root = find_cargo_lock_directory()
        .context("Failed to find project root directory.")?;

    let data_dir = root.join(DATA_DIRECTORY);
    if !data_dir.exists() {
        fs::create_dir(&data_dir)
            .context("Failed to create data directory at project root.")?;
    }
    let name = format!("{}", data);
    let dir = data_dir.join(name);
    if !dir.exists() {
        fs::create_dir(&dir)
            .context("Failed to create subdirectory inside data directory.")?;
    }
    Ok(dir)
}

/* HELPER FUNCTIONS */

/// Searches for a parent directory containing a `Cargo.lock` file.
fn find_cargo_lock_directory() -> Result<path::PathBuf> {
    let mut cwd = env::current_dir()?;
    loop {
        let lock_file = cwd.join("Cargo.lock");
        if lock_file.exists() && lock_file.is_file() {
            return Ok(cwd);
        }
        if let Some(parent_dir) = cwd.parent() {
            cwd = parent_dir.to_owned();
        } else {
            break;
        }
    }
    Err(anyhow!(
        "Could not find any parent directory with a Cargo.lock file."
    ))
}
