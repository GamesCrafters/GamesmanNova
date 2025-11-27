//! # Game Implementation Utilities
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;

use crate::error::GameError;
use crate::game::State;
use crate::game::traits::Codec;
use crate::game::traits::Implicit;
use crate::game::traits::Information;

/* FORWARD VERIFICATION */

/// Verifies that the elements of `history` are a valid sequence of states under
/// the rules of `target`, failing if this is not true.
pub fn verify_state_history<G>(
    target: &G,
    history: Vec<String>,
) -> Result<State>
where
    G: Information + Implicit + Codec,
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

                if target.sink(&prev) {
                    bail!(
                        terminal_history_error(target, prev, next)?.context(
                            format!(
                                "Invalid state transition found at line #{l}.",
                            ),
                        )
                    )
                }

                let transitions = target.outgoing(&prev);
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
                    target.encode(&target.source())?
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

fn transition_history_error<G>(
    target: &G,
    prev: State,
    next: State,
) -> Result<anyhow::Error>
where
    G: Information + Codec,
{
    bail!(GameError::InvalidHistory {
        game: G::info().name,
        hint: format!(
            "Transitioning from the state '{}' to the sate '{}' is illegal in \
            the provided target variant.",
            target.encode(&prev)?,
            target.encode(&next)?,
        ),
    })
}

fn terminal_history_error<G>(
    target: &G,
    prev: State,
    next: State,
) -> Result<anyhow::Error>
where
    G: Information + Codec,
{
    bail!(GameError::InvalidHistory {
        game: G::info().name,
        hint: format!(
            "Transitioning from the state '{}' to the sate '{}' is illegal in \
            the provided target variant, because '{}' is a terminal state.",
            target.encode(&prev)?,
            target.encode(&next)?,
            target.encode(&prev)?,
        ),
    })
}

/* STATE BIT MANGLING */

pub const fn min_ubits(n: u128) -> usize {
    assert!(n > 0);
    (u128::BITS - n.leading_zeros()) as usize
}

pub const fn bits_required_signed(n: i128) -> usize {
    if n >= 0 {
        (u128::BITS - (n as u128).leading_zeros() + 1) as usize
    } else {
        (u128::BITS - ((!n as u128).leading_zeros())) as usize
    }
}
