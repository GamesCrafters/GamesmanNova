//! # Sled Database Applications
//!
//! TODO

use anyhow::Context;
use anyhow::Result;

use std::env;

use crate::traits::game::Variable;

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
