//! # Example Game Module - Simple Utility General Sum
//!
//! This module contains games of type tree, acyclic, and cyclic that
//! adhere to the following definitions:
//!     Simple Utility - Player utilities are defined as WIN, LOSE, TIE, or DRAW.
//!     General Sum - Sum of all utilities is not necessarily zero.

/* IMPORTS */

use crate::game::mock::example::{
    AcyclicExampleGame, CyclicExampleGame, TreeExampleGame, Visualizer,
};
use crate::game::mock::{builder, Node};
use crate::game::{SimpleSum, SimpleUtility};
use crate::model::State;
use crate::node;
use anyhow::Result;

/* CONSTANTS */

const TREE_GAME_NAME: &str = "simple-utility-general-sum-tree-structure";
const ACYCLIC_GAME_NAME: &str = "simple-utility-general-sum-acyclic-structure";
const CYCLIC_GAME_NAME: &str = "simple-utility-general-sum-cyclic-structure";

/* DEFINITIONS */

trait ExampleGame<'a>: Sized {
    fn new(nodes: &'a mut Vec<Node>) -> Result<Self>;
}

/* IMPLEMENTATIONS */

impl<'a> ExampleGame<'a> for TreeExampleGame<'a> {
    fn new(store: &'a mut Vec<Node>) -> Result<TreeExampleGame<'a>> {
        let mut nodes = vec![
            node!(0),
            node!(1),
            node!(1),
            node!(1),
            node!(0),
            node!(0),
            node!(0),
            node!(0),
            node!(0),
            node!(0),
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::WIN.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::LOSE.into(),
            ],
            node![
                SimpleUtility::TIE.into(),
                SimpleUtility::WIN.into(),
            ],
            node![
                SimpleUtility::TIE.into(),
                SimpleUtility::TIE.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::WIN.into(),
            ],
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::LOSE.into(),
            ],
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::WIN.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::LOSE.into(),
            ],
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::TIE.into(),
            ],
        ];

        let length = store.len();
        store.append(&mut nodes);
        let store = &store[length..];

        let game = builder::SessionBuilder::new(&TREE_GAME_NAME)
            .edge(&store[0], &store[1])?
            .edge(&store[0], &store[2])?
            .edge(&store[0], &store[3])?
            .edge(&store[1], &store[4])?
            .edge(&store[1], &store[5])?
            .edge(&store[1], &store[6])?
            .edge(&store[2], &store[7])?
            .edge(&store[2], &store[8])?
            .edge(&store[2], &store[9])?
            .edge(&store[3], &store[10])?
            .edge(&store[3], &store[11])?
            .edge(&store[3], &store[12])?
            .edge(&store[4], &store[13])?
            .edge(&store[5], &store[14])?
            .edge(&store[6], &store[15])?
            .edge(&store[7], &store[16])?
            .edge(&store[8], &store[17])?
            .edge(&store[9], &store[18])?
            .start(&store[0])?
            .build()?;

        Ok(TreeExampleGame { game })
    }
}

impl<'a> ExampleGame<'a> for AcyclicExampleGame<'a> {
    fn new(store: &'a mut Vec<Node>) -> Result<AcyclicExampleGame<'a>> {
        let mut nodes = vec![
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::WIN.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::LOSE.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::TIE.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::TIE.into(),
            ],
            node![
                SimpleUtility::TIE.into(),
                SimpleUtility::TIE.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::WIN.into(),
            ],
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::LOSE.into(),
            ],
        ];

        let length = store.len();
        store.append(&mut nodes);
        let store = &store[length..];

        let game = builder::SessionBuilder::new(&ACYCLIC_GAME_NAME)
            .edge(&store[0], &store[1])?
            .edge(&store[0], &store[3])?
            .edge(&store[1], &store[2])?
            .edge(&store[1], &store[4])?
            .edge(&store[2], &store[5])?
            .edge(&store[2], &store[7])?
            .edge(&store[3], &store[4])?
            .edge(&store[3], &store[6])?
            .edge(&store[4], &store[7])?
            .edge(&store[4], &store[9])?
            .edge(&store[5], &store[6])?
            .edge(&store[5], &store[8])?
            .edge(&store[6], &store[9])?
            .edge(&store[6], &store[11])?
            .edge(&store[7], &store[8])?
            .edge(&store[7], &store[10])?
            .edge(&store[8], &store[11])?
            .edge(&store[8], &store[13])?
            .edge(&store[9], &store[10])?
            .edge(&store[9], &store[12])?
            .edge(&store[10], &store[13])?
            .edge(&store[10], &store[15])?
            .edge(&store[11], &store[12])?
            .edge(&store[11], &store[14])?
            .edge(&store[12], &store[15])?
            .edge(&store[12], &store[17])?
            .edge(&store[13], &store[14])?
            .edge(&store[13], &store[16])?
            .edge(&store[14], &store[17])?
            .edge(&store[14], &store[19])?
            .edge(&store[15], &store[16])?
            .edge(&store[15], &store[18])?
            .edge(&store[16], &store[19])?
            .edge(&store[16], &store[20])?
            .edge(&store[17], &store[21])?
            .edge(&store[17], &store[22])?
            .start(&store[0])?
            .build()?;

        Ok(AcyclicExampleGame { game })
    }
}

impl<'a> ExampleGame<'a> for CyclicExampleGame<'a> {
    fn new(store: &'a mut Vec<Node>) -> Result<CyclicExampleGame<'a>> {
        let mut nodes = vec![
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node!(0),
            node!(1),
            node!(0),
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::LOSE.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::LOSE.into(),
            ],
            node![
                SimpleUtility::TIE.into(),
                SimpleUtility::WIN.into(),
            ],
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::WIN.into(),
            ],
            node![
                SimpleUtility::LOSE.into(),
                SimpleUtility::LOSE.into(),
            ],
            node![
                SimpleUtility::WIN.into(),
                SimpleUtility::LOSE.into(),
            ],
            node![
                SimpleUtility::TIE.into(),
                SimpleUtility::WIN.into(),
            ],
        ];

        let length = store.len();
        store.append(&mut nodes);
        let store = &store[length..];

        let game = builder::SessionBuilder::new(&CYCLIC_GAME_NAME)
            .edge(&store[0], &store[1])?
            .edge(&store[0], &store[3])?
            .edge(&store[0], &store[5])?
            .edge(&store[1], &store[8])?
            .edge(&store[1], &store[10])?
            .edge(&store[2], &store[1])?
            .edge(&store[2], &store[3])?
            .edge(&store[2], &store[5])?
            .edge(&store[3], &store[2])?
            .edge(&store[3], &store[4])?
            .edge(&store[4], &store[5])?
            .edge(&store[4], &store[7])?
            .edge(&store[5], &store[6])?
            .edge(&store[5], &store[8])?
            .edge(&store[6], &store[7])?
            .edge(&store[6], &store[9])?
            .edge(&store[6], &store[16])?
            .edge(&store[7], &store[11])?
            .edge(&store[7], &store[12])?
            .edge(&store[7], &store[4])?
            .edge(&store[8], &store[9])?
            .edge(&store[8], &store[14])?
            .edge(&store[9], &store[13])?
            .edge(&store[10], &store[15])?
            .edge(&store[10], &store[13])?
            .start(&store[0])?
            .build()?;

        Ok(CyclicExampleGame { game })
    }
}

/* TREE GAME UTILITY IMPLEMENTATIONS */

impl SimpleSum<2> for TreeExampleGame<'_> {
    fn utility(&self, state: State) -> [SimpleUtility; 2] {
        match self.game.node(state) {
            Node::Terminal(vector) => [
                vector[0].try_into().unwrap(),
                vector[1].try_into().unwrap(),
            ],
            Node::Medial(_) => {
                panic!("Attempted to fetch utility of medial state.")
            },
        }
    }
}

/* TESTING */

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn initialize_example_tree_game() -> Result<()> {
        let _ = TreeExampleGame::new(&mut vec![])?;
        Ok(())
    }

    #[test]
    fn initialize_example_acyclic_game() -> Result<()> {
        let _ = AcyclicExampleGame::new(&mut vec![])?;
        Ok(())
    }

    #[test]
    fn initialize_example_cyclic_game() -> Result<()> {
        let _ = CyclicExampleGame::new(&mut vec![])?;
        Ok(())
    }

    #[test]
    fn visualize_example_tree_game() -> Result<()> {
        let _ = TreeExampleGame::new(&mut vec![])?.visualize();
        Ok(())
    }

    #[test]
    fn visualize_example_acyclic_game() -> Result<()> {
        let _ = AcyclicExampleGame::new(&mut vec![])?.visualize();
        Ok(())
    }

    #[test]
    fn visualize_example_cyclic_game() -> Result<()> {
        let _ = CyclicExampleGame::new(&mut vec![])?.visualize();
        Ok(())
    }
}
