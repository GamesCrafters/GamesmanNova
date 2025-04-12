//! # Interface Utilities Module
//!
//! This module makes room for common utility routines used throughout the
//! `crate::interface` module. The contents here are unstructured.

use anyhow::{Context, Result};
use serde_json::{Map, Value};

use crate::target::TargetData;

use super::{InfoFormat, TargetAttribute};

/* CONSTANTS */

/// All available game attributes uniquely listed.
const GAME_ATTRIBUTES: [TargetAttribute; 9] = [
    TargetAttribute::VariantProtocol,
    TargetAttribute::VariantDefault,
    TargetAttribute::VariantPattern,
    TargetAttribute::StateProtocol,
    TargetAttribute::StateDefault,
    TargetAttribute::StatePattern,
    TargetAttribute::Authors,
    TargetAttribute::About,
    TargetAttribute::Name,
];

/* OUTPUT UTILITIES */

/// Collects the attributes specified in `attr` from the provided game `data`
/// to a single string in a specific `format`.
pub fn aggregate_and_format_attributes(
    data: TargetData,
    attrs: Vec<TargetAttribute>,
    format: InfoFormat,
) -> Result<String> {
    match format {
        InfoFormat::Legible => {
            let mut output = String::new();
            attrs.iter().for_each(|&a| {
                output += &format!("\t{a}:\n{}\n\n", data.find(a))
            });
            Ok(output)
        },
        InfoFormat::Json => {
            let mut map = Map::new();
            attrs.iter().for_each(|&a| {
                map.insert(a.to_string(), Value::String(data.find(a).into()));
            });
            serde_json::to_string(&map)
                .context("Failed to generate JSON object from game data.")
        },
    }
}

/// Collects all possible game attributes from the provided game `data` to a
/// single string in a specific `format`.
pub fn aggregate_and_format_all_attributes(
    data: TargetData,
    format: InfoFormat,
) -> Result<String> {
    aggregate_and_format_attributes(data, GAME_ATTRIBUTES.to_vec(), format)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn no_duplicates_in_game_attrs_list() {
        let mut attrs = GAME_ATTRIBUTES.to_vec();
        let s1 = attrs.len();
        attrs.sort();
        attrs.dedup();
        let s2 = attrs.len();
        assert_eq!(s1, s2);
    }
}
