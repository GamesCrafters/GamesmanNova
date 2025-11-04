//! # Game Implementations
//!
//! TODO

use std::fmt::Display;

use crate::types::frontend::GameAttribute;
use crate::types::game::GameData;

/* SUBMODULES */

pub mod util;

/* GAME IMPLEMENTATIONS */

#[cfg(test)]
pub mod mock;
pub mod zero_by;

/* METADATA FORMAATTING */

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
