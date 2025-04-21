//! # Game Utility Module
//!
//! This module defines utilities used game implementations.

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use rusqlite::Connection;

use std::env;
use std::fmt::Display;

use crate::game::GameData;
use crate::game::Information;
use crate::game::State;
use crate::game::{Codec, Implicit, error::GameError};
use crate::interface::GameAttribute;

/* DATABASE */

/// Parses environment variables and establishes an SQLite connection to the
/// global game solution database.
pub fn database() -> Result<Connection> {
    let path = env::var("DATABASE")
        .context("DATABASE environment variable not set.")?;

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

/* STATE HISTORY VERIFICATION */

/// Verifies that the elements of `history` are a valid sequence of states under
/// the rules of `target`, failing if this is not true.
pub fn verify_state_history<const B: usize, G>(
    target: &G,
    history: Vec<String>,
) -> Result<State<B>>
where
    G: Information + Implicit<B> + Codec<B>,
{
    let history = sanitize_input(history);
    if let Some((l, s)) = history.first() {
        let mut prev = target
            .decode(s.clone())
            .context(format!("Failed to parse line #{l}."))?;

        if prev == target.source() {
            for h in history.iter().skip(1) {
                let (l, s) = h.clone();
                let next = target
                    .decode(s)
                    .context(format!("Failed to parse line #{l}."))?;

                if target.sink(prev) {
                    bail!(
                        terminal_history_error(target, prev, next)?.context(
                            format!(
                                "Invalid state transition found at line #{l}.",
                            ),
                        )
                    )
                }

                let transitions = target.adjacent(prev);
                if !transitions.contains(&next) {
                    bail!(
                        transition_history_error(target, prev, next)?.context(
                            format!(
                                "Invalid state transition found at line #{l}."
                            ),
                        )
                    )
                }
                prev = next;
            }
            Ok(prev)
        } else {
            bail!(GameError::InvalidHistory {
                game: G::info().name,
                hint: format!(
                    "The state history must begin with the starting state for \
                    the provided game variant, which is {}.",
                    target.encode(target.source())?
                ),
            })
        }
    } else {
        bail!(GameError::InvalidHistory {
            game: G::info().name,
            hint: "State history must contain at least one state.".into(),
        })
    }
}

/// Enumerates lines and trims whitespace from input.
fn sanitize_input(mut input: Vec<String>) -> Vec<(usize, String)> {
    input
        .iter_mut()
        .enumerate()
        .map(|(i, s)| (i, s.trim().to_owned()))
        .filter(|(_, s)| !s.is_empty())
        .collect()
}

/* HISTORY VERIFICATION ERRORS */

fn transition_history_error<const B: usize, G>(
    target: &G,
    prev: State<B>,
    next: State<B>,
) -> Result<anyhow::Error>
where
    G: Information + Codec<B>,
{
    bail!(GameError::InvalidHistory {
        game: G::info().name,
        hint: format!(
            "Transitioning from the state '{}' to the sate '{}' is illegal in \
            the provided target variant.",
            target.encode(prev)?,
            target.encode(next)?,
        ),
    })
}

fn terminal_history_error<const B: usize, G>(
    target: &G,
    prev: State<B>,
    next: State<B>,
) -> Result<anyhow::Error>
where
    G: Information + Codec<B>,
{
    bail!(GameError::InvalidHistory {
        game: G::info().name,
        hint: format!(
            "Transitioning from the state '{}' to the sate '{}' is illegal in \
            the provided target variant, because '{}' is a terminal state.",
            target.encode(prev)?,
            target.encode(next)?,
            target.encode(prev)?,
        ),
    })
}

/* GAME DATA UTILITIES */

impl Display for GameAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            GameAttribute::VariantProtocol => "variant-protocol",
            GameAttribute::VariantPattern => "variant-pattern",
            GameAttribute::VariantDefault => "variant-default",
            GameAttribute::StateProtocol => "state-protocol",
            GameAttribute::StateDefault => "state-default",
            GameAttribute::StatePattern => "state-pattern",
            GameAttribute::Authors => "authors",
            GameAttribute::About => "about",
            GameAttribute::Name => "name",
        };
        write!(f, "{content}")
    }
}

impl GameData {
    pub fn find(&self, attribute: GameAttribute) -> &str {
        match attribute {
            GameAttribute::VariantProtocol => self.variant_protocol,
            GameAttribute::VariantPattern => self.variant_pattern,
            GameAttribute::VariantDefault => self.variant_default,
            GameAttribute::StateProtocol => self.state_protocol,
            GameAttribute::StateDefault => self.state_default,
            GameAttribute::StatePattern => self.state_pattern,
            GameAttribute::Authors => self.authors,
            GameAttribute::About => self.about,
            GameAttribute::Name => self.name,
        }
    }
}
