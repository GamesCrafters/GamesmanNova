//! # Scheduler Phases
//!
//! Five-phase tick execution with explicit contracts.

use anyhow::Result;

/* SUBMODULES */

pub mod collection;
pub mod execution;
pub mod preemption;
pub mod resolution;
pub mod retry;

/* STRUCTURES */

/// Result of phase execution.
pub struct PhaseResult {
    pub changed: bool,
    pub affected: usize,
}

impl PhaseResult {
    pub fn none() -> Self {
        Self {
            affected: 0,
            changed: false,
        }
    }
}

/* TRAITS */

/// Phase execution trait.
pub trait Phase {
    fn execute(&mut self) -> Result<PhaseResult>;
    fn name(&self) -> &'static str;
}
