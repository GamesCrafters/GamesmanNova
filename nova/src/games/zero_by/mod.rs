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

use super::{
    Game, GameData, Solvable, Automaton,
};
use nalgebra::{Matrix2, SMatrix, SVector, Vector2};
use crate::{
    models::{Solver, State, Variant, Player},
};
use regex::Regex;
use std::process;

/* GAME DATA */

pub const NAME: &str = "Zero-By";
pub const AUTHOR: &str = "Max Fierro";
pub const CATEGORY: &str = "Two-player game";
pub const ABOUT: &str = 
"Two players take turns removing a number of elements from a set of arbitrary \
size. They can make a choice of how many elements to remove (and of how many \
elements to start out with) based on the game variant. The player who is left \
with 0 elements in their turn loses. A player cannot remove more elements than \
currently available in the set.";

pub const VARIANT_DEFAULT: &str = "10-1-2";
pub const VARIANT_PATTERN: &str = r"^[1-9]\d*(?:-[1-9]\d*)+$";
pub const VARIANT_PROTOCOL: &str =
"The variant string should be a dash-separated group of two or more positive \
integers. For example, '239-232-23-6-3-6' is valid but '598', '-23-1-5', and \
'fifteen-2-5' are not. The first integer represents the beginning number of \
elements in the set, and the rest are choices that the players have when they \
need to remove a number of pieces on their turn. Note that the numbers can be \
repeated, but if you repeat the first number it will be a win for the player \
with the first turn in 1 move. If you repeat any of the rest of the numbers, \
the only consequence will be a slight decrease in performance.";

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
    fn initialize(variant: Option<Variant>) -> Self
    {
        if let Some(variant) = variant {
            decode_variant(variant)
        } else {
            decode_variant(VARIANT_DEFAULT.to_owned())
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
        Matrix2::new(1, 1, 1, 1);
    }

    fn utility(&self, state: State) -> Option<SVector<i32, 1>>
    {
        if !self.accepts(state) {
            None
        } else {
            Some(Vector2::new(state % 2, (state + 1) % 2))
        }
    }

    fn solvers(&self) -> Vec<(String, Solver<Self>)>
    {

    }
}

/* HELPER FUNCTIONS */

impl Session {

}

fn decode_variant(v: Variant) -> Session
{
    let re = Regex::new(VARIANT_PATTERN).unwrap();

    if !re.is_match(&v) {
        println!("Variant string malformed.");
        process::exit(exitcode::USAGE);
    }

    let mut from_by = v
        .split('-')
        .map(|int_string| {
            int_string.parse::<u64>().expect("Could not parse variant.")
        })
        .collect::<Vec<u64>>();

    Session {
        variant: Some(v),
        from: *from_by.first().unwrap(),
        by: {
            from_by.remove(0);
            from_by
        },
    }
}

fn encode_turn(state: State, player_count: Turn, turn: Turn) -> State 
{
    let turn_bits = 
        (0 as Turn).leading_zeros() - player_count.leading_zeros();
    let shifted_state = state << turn_bits;
    shifted_state + turn
}

fn decode_turn(state: State, ) -> Turn {

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
    fn init_with_no_variant_is_the_same_as_with_default_variant()
    {
        let with_none = Session::initialize(None);
        let with_default =
            Session::initialize(Some(VARIANT_DEFAULT.to_owned()));
        assert_eq!(with_none.variant, with_default.variant);
        assert_eq!(with_none.from, with_default.from);
        assert_eq!(with_none.by, with_default.by);
    }
}
