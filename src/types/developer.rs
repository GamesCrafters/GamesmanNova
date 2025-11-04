//! # Developer Models  
//!
//! TODO

use strum_macros::Display;

use std::sync::RwLock;

/* CONSTANTS */

/// Global lock for creating development data directories. Since `cargo test`
/// executes tests in parallel, this helps prevent test flakiness by avoiding
/// race conditions when creating directories (specific files are still
/// susceptible, but they have no specific structure).
pub static DIRECTORY_LOCK: RwLock<()> = RwLock::new(());

/// The name of the global directory at the project root used for generated
/// development data. This directory is not shipped with release builds.
pub const DEV_DIRECTORY: &str = "dev";

/* ENUMERATIONS */

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
