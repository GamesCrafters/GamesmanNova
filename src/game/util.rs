//! # Game Utilities Module
//!
//! This module provides some common utilities used in the implementation of
//! more than a single game.

use std::fmt::Display;

use anyhow::bail;
use anyhow::{Context, Result};

use crate::game::GameData;
use crate::game::Information;
use crate::interface::GameAttribute;
use crate::{
    game::{error::GameError, Bounded, Codec, Transition},
    model::game::State,
};

/* STATE HISTORY VERIFICATION */

/// Verifies that the elements of `history` are a valid sequence of states under
/// the rules of `game`, failing if this is not true.
pub fn verify_state_history<const B: usize, G>(
    game: &G,
    history: Vec<String>,
) -> Result<State<B>>
where
    G: Information + Bounded<B> + Codec<B> + Transition<B>,
{
    let history = sanitize_input(history);
    if let Some((l, s)) = history.first() {
        let mut prev = game
            .decode(s.clone())
            .context(format!("Failed to parse line #{l}."))?;
        if prev == game.start() {
            for h in history.iter().skip(1) {
                let (l, s) = h.clone();
                let next = game
                    .decode(s)
                    .context(format!("Failed to parse line #{l}."))?;
                if game.end(prev) {
                    bail!(
                        terminal_history_error(game, prev, next)?.context(
                            format!(
                                "Invalid state transition found at line #{l}.",
                            ),
                        )
                    )
                }
                let transitions = game.prograde(prev);
                if !transitions.contains(&next) {
                    bail!(
                        transition_history_error(game, prev, next)?.context(
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
                game_name: G::info().name,
                hint: format!(
                    "The state history must begin with the starting state for \
                    the provided game variant, which is {}.",
                    game.encode(game.start())?
                ),
            })
        }
    } else {
        bail!(GameError::InvalidHistory {
            game_name: G::info().name,
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
    game: &G,
    prev: State<B>,
    next: State<B>,
) -> Result<anyhow::Error>
where
    G: Information + Codec<B> + Bounded<B>,
{
    bail!(GameError::InvalidHistory {
        game_name: G::info().name,
        hint: format!(
            "Transitioning from the state '{}' to the sate '{}' is illegal in \
            the provided game variant.",
            game.encode(prev)?,
            game.encode(next)?,
        ),
    })
}

fn terminal_history_error<const B: usize, G>(
    game: &G,
    prev: State<B>,
    next: State<B>,
) -> Result<anyhow::Error>
where
    G: Information + Codec<B> + Bounded<B>,
{
    bail!(GameError::InvalidHistory {
        game_name: G::info().name,
        hint: format!(
            "Transitioning from the state '{}' to the sate '{}' is illegal in \
            the provided game variant, because '{}' is a terminal state.",
            game.encode(prev)?,
            game.encode(next)?,
            game.encode(prev)?,
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
