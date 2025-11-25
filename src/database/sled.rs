//! # Sled Database Applications
//!
//! TODO

use anyhow::Context;
use anyhow::Result;

use std::env;

use crate::frontend::IOMode;

/* CONSTANTS */

/// Environment variable with Sled DB path.
const SLED_DATABASE: &str = "SLED_DATABASE";

/// In bytes. Recall 1GB ~= 1_000_000_000B.
const CACHE_CAPACITY: u64 = 10_000_000_000;

/// In milliseconds.
const FLUSH_INTERVAL: u64 = 500;

/* HELPER FUNCTIONS */

pub fn init_sled(mode: IOMode, name: &str) -> Result<sled::Db> {
    let path = env::var(SLED_DATABASE).with_context(|| {
        format!("{SLED_DATABASE} environment variable must be set")
    })?;

    let cfg = match mode {
        IOMode::Forgetful => sled::Config::new().temporary(true),
        IOMode::Overwrite => sled::Config::new().path(&path),
        IOMode::Constructive => sled::Config::new()
            .create_new(false)
            .path(&path),
    };

    let db = cfg
        .flush_every_ms(Some(FLUSH_INTERVAL))
        .mode(sled::Mode::HighThroughput)
        .cache_capacity(CACHE_CAPACITY)
        .print_profile_on_drop(true)
        .open()
        .context("Failed to open Sled database")?;

    if matches!(mode, IOMode::Overwrite) {
        db.drop_tree(name)
            .context("Failed to drop Sled tree for Overwrite mode")?;
    }

    Ok(db)
}
