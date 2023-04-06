//! # Acyclic Solver Module
//! 
//! This module implements an acyclic solver for all applicable types of games
//! through a blanket implementation of the `AcyclicallySolvable` trait.
//! 
//! #### Authorship
//! 
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

use super::{choose_value, AcyclicallySolvable};
use crate::archetypes::{Game, AcyclicGame, TreeGame};
use crate::{State, Value};
use std::collections::HashMap;

/* IMPLEMENTATIONS */

impl<G> AcyclicallySolvable for G
where
    G: Game + AcyclicGame,
    G: Game + TreeGame,
{
    fn acyclic_solve(&self) -> Value {
        let initial_state = self.state();
        let mut seen: HashMap<State, Value> = HashMap::new();
        traverse(initial_state, self, &mut seen)
    }
}

/* HELPER FUNCTIONS */

fn traverse<G>(state: State, game: &G, seen: &mut HashMap<State, Value>) -> Value 
where
    G: Game + AcyclicGame,
    G: Game + TreeGame,
{
    if let Some(out) = game.value(state) {
        return out;
    }
    let mut child_values: Vec<Value> = Vec::new();
    for mv in game.generate_moves(state) {
        let child = game.play(state, mv);
        if let Some(out) = seen.get(&child).copied() {
            child_values.push(out);
        } else {
            let out = traverse(child, game, seen);
            child_values.push(out);
            seen.insert(child, out);
        }
    }
    choose_value(child_values)
}
