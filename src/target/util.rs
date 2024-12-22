//! # Extraction Target Utility Module
//!
//! This module defines utilities used across extraction target implementations.

use anyhow::bail;
use anyhow::{Context, Result};

use std::fmt::Display;
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::database::model::SequenceKey;
use crate::interface::TargetAttribute;
use crate::target::model::State;
use crate::target::Information;
use crate::target::TargetData;
use crate::target::Variable;
use crate::target::{error::TargetError, Bounded, Codec, Transition};
use crate::target::{Extractor, Target};
use crate::util::Identify;

/* DEFINITIONS */

/// TODO
pub struct ExtractorBuilder<'a, T>
where
    T: Target,
{
    name: &'a str,
    function: Option<Box<dyn Fn(&'a T) -> Result<()> + 'a>>,
    features: Vec<&'a str>,
    requires: Vec<&'a str>,
}

/* EXTRACTOR BUILDER PATTERN */

impl<'a, T> ExtractorBuilder<'a, T>
where
    T: Target,
{
    /// TODO
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            function: None,
            features: Vec::new(),
            requires: Vec::new(),
        }
    }

    /// TODO
    pub fn runs(mut self, function: impl Fn(&'a T) -> Result<()> + 'a) -> Self {
        self.function = Some(Box::new(function));
        self
    }

    /// TODO
    pub fn extracts(mut self, name: &'a str) -> Self {
        self.features.push(name);
        self
    }

    /// TODO
    pub fn requires(mut self, name: &'a str) -> Self {
        self.requires.push(name);
        self
    }

    /// TODO
    pub fn build(self) -> Result<Extractor<'a, T>> {
        if self.features.len() == 0 {
            todo!()
        }

        if let Some(function) = self.function {
            Ok(Extractor {
                name: self.name,
                function,
                provides: self.features,
                requires: self.requires,
            })
        } else {
            todo!()
        }
    }
}

/* STATE HISTORY VERIFICATION */

/// Verifies that the elements of `history` are a valid sequence of states under
/// the rules of `target`, failing if this is not true.
pub fn verify_state_history<const B: usize, T>(
    target: &T,
    history: Vec<String>,
) -> Result<State<B>>
where
    T: Information + Bounded<B> + Codec<B> + Transition<B>,
{
    let history = sanitize_input(history);
    if let Some((l, s)) = history.first() {
        let mut prev = target
            .decode(s.clone())
            .context(format!("Failed to parse line #{l}."))?;
        if prev == target.start() {
            for h in history.iter().skip(1) {
                let (l, s) = h.clone();
                let next = target
                    .decode(s)
                    .context(format!("Failed to parse line #{l}."))?;
                if target.end(prev) {
                    bail!(
                        terminal_history_error(target, prev, next)?.context(
                            format!(
                                "Invalid state transition found at line #{l}.",
                            ),
                        )
                    )
                }
                let transitions = target.prograde(prev);
                if !transitions.contains(&next) {
                    bail!(transition_history_error(target, prev, next)?
                        .context(format!(
                            "Invalid state transition found at line #{l}."
                        ),))
                }
                prev = next;
            }
            Ok(prev)
        } else {
            bail!(TargetError::InvalidHistory {
                target_name: T::info().name,
                hint: format!(
                    "The state history must begin with the starting state for \
                    the provided game variant, which is {}.",
                    target.encode(target.start())?
                ),
            })
        }
    } else {
        bail!(TargetError::InvalidHistory {
            target_name: T::info().name,
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

fn transition_history_error<const B: usize, T>(
    target: &T,
    prev: State<B>,
    next: State<B>,
) -> Result<anyhow::Error>
where
    T: Information + Codec<B> + Bounded<B>,
{
    bail!(TargetError::InvalidHistory {
        target_name: T::info().name,
        hint: format!(
            "Transitioning from the state '{}' to the sate '{}' is illegal in \
            the provided target variant.",
            target.encode(prev)?,
            target.encode(next)?,
        ),
    })
}

fn terminal_history_error<const B: usize, T>(
    target: &T,
    prev: State<B>,
    next: State<B>,
) -> Result<anyhow::Error>
where
    T: Information + Codec<B> + Bounded<B>,
{
    bail!(TargetError::InvalidHistory {
        target_name: T::info().name,
        hint: format!(
            "Transitioning from the state '{}' to the sate '{}' is illegal in \
            the provided target variant, because '{}' is a terminal state.",
            target.encode(prev)?,
            target.encode(next)?,
            target.encode(prev)?,
        ),
    })
}

/* TARGET DATA UTILITIES */

impl Display for TargetAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            TargetAttribute::VariantProtocol => "variant-protocol",
            TargetAttribute::VariantPattern => "variant-pattern",
            TargetAttribute::VariantDefault => "variant-default",
            TargetAttribute::StateProtocol => "state-protocol",
            TargetAttribute::StateDefault => "state-default",
            TargetAttribute::StatePattern => "state-pattern",
            TargetAttribute::Authors => "authors",
            TargetAttribute::About => "about",
            TargetAttribute::Name => "name",
        };
        write!(f, "{content}")
    }
}

impl TargetData {
    pub fn find(&self, attribute: TargetAttribute) -> &str {
        match attribute {
            TargetAttribute::VariantProtocol => self.variant_protocol,
            TargetAttribute::VariantPattern => self.variant_pattern,
            TargetAttribute::VariantDefault => self.variant_default,
            TargetAttribute::StateProtocol => self.state_protocol,
            TargetAttribute::StateDefault => self.state_default,
            TargetAttribute::StatePattern => self.state_pattern,
            TargetAttribute::Authors => self.authors,
            TargetAttribute::About => self.about,
            TargetAttribute::Name => self.name,
        }
    }
}

/* IDENTIFICATION */

impl<T> Identify for T
where
    T: Variable,
{
    fn id(&self) -> SequenceKey {
        let mut hasher = DefaultHasher::new();
        self.variant_string()
            .hash(&mut hasher);
        hasher.finish()
    }
}
