//! # Sled Database Utilities
//!
//! TODO

use anyhow::Context;
use anyhow::Result;

use std::env;

use crate::game::Variable;

/* API */

/// Parses environment variables and obtains a handle to the sled game solution
/// global database.
pub fn open_tree<G>(game: &G) -> Result<sled::Tree>
where
    G: Variable,
{
    let path = env::var("SLED_DATABASE")
        .context("SLED_DATABASE environment variable not set.")?;

    let db = sled::open(path)?;
    let tree = db.open_tree(game.name())?;
    Ok(tree)
}

/// TODO
pub fn drop_tree<G>(game: &G) -> Result<()>
where
    G: Variable,
{
    let path = env::var("SLED_DATABASE")
        .context("SLED_DATABASE environment variable not set.")?;

    let db = sled::open(path)?;
    db.drop_tree(game.name())?;
    Ok(())
}
