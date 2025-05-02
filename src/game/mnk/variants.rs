//! # MNK Variant Handling Module
//!
//! TODO

use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use bitvec::array::BitArray;
use bitvec::field::BitField;
use bitvec::order::Msb0;
use regex::Regex;

use crate::game::mnk::NAME;
use crate::game::mnk::Session;
use crate::solver::db::SchemaBuilder;

/* MNK VARIANT ENCODING */

pub const VARIANT_DEFAULT: &str = "3-3-3";
pub const VARIANT_PATTERN: &str =
    r"^([1-9][0-9]*)-([1-9][0-9]*)-([1-9][0-9]*)$";

pub const VARIANT_PROTOCOL: &str = "Three nonzero positive integers separated \
by dashes, in the form M-N-K. Here, M and N are the dimensions of the board, \
and K is the number of symbols which, when placed in a row, are needed to win.";

/* API */

/// TODO
pub fn parse_variant(variant: String) -> Result<Session> {
    let re = Regex::new(VARIANT_PATTERN)
        .expect("`VARIANT_PATTERN` is a valid regex");

    if !re.is_match(&variant) {
        bail!(
            "Variant {:?} does not match pattern `{}`",
            variant,
            VARIANT_PATTERN
        );
    }

    let parts: Vec<usize> = variant
        .split('-')
        .map(|s| {
            s.parse::<usize>()
                .map_err(|_| anyhow!("Failed to parse integer."))
        })
        .collect::<Result<_>>()?;

    let (m, n, k) = match *parts.as_slice() {
        [m, n, k] => (m, n, k),
        _ => bail!(
            "Variant {:?} must have exactly three dash-separated integers (got {})",
            variant,
            parts.len()
        ),
    };

    let table = format!("{}_{}", NAME, variant);
    let schema = SchemaBuilder::new(&table)
        .players(2)
        .key("state", "INTEGER")
        .column("remoteness", "INTEGER")
        .column("player", "INTEGER")
        .build()?;

    let mut state = BitArray::<_, Msb0>::ZERO;
    state[..1].store_be(1);

    Ok(Session {
        schema,
        start: state.data,
        m,
        n,
        k,
    })
}
