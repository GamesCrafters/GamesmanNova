//! # Example Mock Test Game Module [WIP]
//!
//! This module provides sub modules with concrete examples of small games that adhere
//! to useful interface definitions that can be used for testing purposes. The games here
//! are built over the `mock` game graph implementation.

/* IMPORTS */

use crate::game::mock::{MockGame, Session};
use anyhow::Result;

/* CONSTANTS */

const MODULE_STORAGE: &str = "mock-game-examples";

/* SUBMODULES */

pub mod general_utility;
pub mod simple_utility;

/* DEFINITIONS */

pub struct TreeExampleGame<'a> {
    game: Session<'a>,
}

pub struct AcyclicExampleGame<'a> {
    game: Session<'a>,
}

pub struct CyclicExampleGame<'a> {
    game: Session<'a>,
}

pub trait Visualizer {
    fn visualize(&self) -> Result<()>;
}

/* TRAVERSAL IMPLEMENTATIONS */

impl MockGame for TreeExampleGame<'_> {
    fn game(&self) -> &Session<'_> {
        &self.game
    }
}

impl MockGame for AcyclicExampleGame<'_> {
    fn game(&self) -> &Session<'_> {
        &self.game
    }
}

impl MockGame for CyclicExampleGame<'_> {
    fn game(&self) -> &Session<'_> {
        &self.game
    }
}

/* VISUALIZER IMPLEMENTATIONS */

impl Visualizer for TreeExampleGame<'_> {
    fn visualize(&self) -> Result<()> {
        self.game.visualize(MODULE_STORAGE)
    }
}

impl Visualizer for AcyclicExampleGame<'_> {
    fn visualize(&self) -> Result<()> {
        self.game.visualize(MODULE_STORAGE)
    }
}

impl Visualizer for CyclicExampleGame<'_> {
    fn visualize(&self) -> Result<()> {
        self.game.visualize(MODULE_STORAGE)
    }
}
