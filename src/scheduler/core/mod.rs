//! # Scheduler Core
//!
//! Core scheduler implementation with type-state safety and phase
//! encapsulation.

pub mod context;
pub mod decision;
pub mod phase;
pub mod registry;
pub mod state;

pub use context::AnyContext;
pub use context::TaskContext;
pub use decision::DecisionContext;
pub use phase::Phase;
pub use phase::PhaseResult;
pub use registry::Registry;
pub use state::State;
