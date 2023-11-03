//! # Zero-By Game Module
//!
//! Zero-By is a small acyclic game, where two players take turns removing
//! one of certain amounts of elements from a set of N elements. For example,
//! players could take turns removing either one or two coins from a stack
//! of ten, which would be an instance of Ten to Zero by One or Two (coins).
//!
//! This module encapsulates the commonalities for all Zero-By games, allowing
//! users to specify which abstract instance of the Zero-By game they wish to
//! emulate.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use crate::games::{
    Game, GameData, Solvable, Automaton,
};
use crate::{models::{Solver, State, Variant, Player}, errors::VariantError};
use nalgebra::{Matrix2, SMatrix, SVector, Vector2};
use variants::*;

/* SUBMODULES */

mod variants;
mod utils;

/* GAME DATA */

const NAME: &str = "Zero-By";
const AUTHOR: &str = "Max Fierro";
const CATEGORY: &str = "Combinatorial n-player zero-sum game";
const ABOUT: &str = 
"Many players take turns removing a number of elements from a set of arbitrary \
size. They can make a choice of how many elements to remove (and of how many \
elements to start out with) based on the game variant. The player who is left \
with 0 elements in their turn loses. A player cannot remove more elements than \
currently available in the set.";

/* GAME IMPLEMENTATION */

pub struct Session
{
    variant: Option<String>,
    players: Player,
    from: State,
    by: Vec<u64>,
}

impl Game for Session
{
    fn initialize(variant: Option<Variant>) -> Result<Self, VariantError>
    {
        if let Some(v) = variant {
            parse_variant(v)
        } else {
            parse_variant(VARIANT_DEFAULT.to_owned())
        }
    }

    fn id(&self) -> String
    {
        if let Some(variant) = self.variant.clone() {
            format!("{}.{}", NAME, variant)
        } else {
            NAME.to_owned()
        }
    }

    fn info(&self) -> GameData
    {
        GameData {
            name: NAME.to_owned(),
            author: AUTHOR.to_owned(),
            about: ABOUT.to_owned(),
            category: CATEGORY.to_owned(),
            variant_protocol: VARIANT_PROTOCOL.to_owned(),
            variant_pattern: VARIANT_PATTERN.to_owned(),
            variant_default: VARIANT_DEFAULT.to_owned(),
        }
    }
}

impl Automaton<State> for Session
{
    fn start(&self) -> State
    {
        self.from
    }

    fn transition(&self, state: State) -> Vec<State>
    {
        self.by
            .iter()
            .cloned()
            .filter(|&mv| state >= mv)
            .map(|mv| state - mv)
            .collect::<Vec<State>>()
    }

    fn accepts(&self, state: State) -> bool
    {
        state == 0
    }
}

impl Solvable<2> for Session
{
    fn weights(&self) -> SMatrix<i32, 2, 2>
    {
        Matrix2::new(1, 0, 0, 1)
    }

    fn utility(&self, state: State) -> Option<SVector<i32, 1>>
    {
        if !self.accepts(state) {
            None
        } else {
            Some(Vector2::new(state % 2, (state + 1) % 2))
        }
    }

    fn solvers(&self) -> Vec<(String, Solver<Self, 2>)>
    {

    }
}

/* TESTS */

#[cfg(test)]
mod test
{
    use crate::games::{
        zero_by::{Session, VARIANT_DEFAULT, VARIANT_PATTERN},
        Game,
    };
    use regex::Regex;

    #[test]
    fn variant_pattern_is_valid_regex()
    {
        assert!(Regex::new(VARIANT_PATTERN).is_ok());
    }

    #[test]
    fn default_variant_matches_variant_pattern()
    {
        let re = Regex::new(VARIANT_PATTERN).unwrap();
        assert!(re.is_match(VARIANT_DEFAULT));
    }

    #[test]
    fn no_variant_equals_default_variant()
    {
        let with_none = Session::initialize(None);
        let with_default =
            Session::initialize(Some(VARIANT_DEFAULT.to_owned()));
        assert_eq!(with_none.variant, with_default.variant);
        assert_eq!(with_none.from, with_default.from);
        assert_eq!(with_none.by, with_default.by);
    }
}
