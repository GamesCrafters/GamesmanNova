//! # Scheduler Runner Types
//!
//! TODO

use anyhow::Result;
use derive_builder::Builder;

use std::collections::HashMap;

use crate::traits::scheduler::Executable;
use crate::types::scheduler::TaskID;
use crate::types::scheduler::YieldUpdate;

/* RUNNER STRUCTURES */

/// Synchronous runner that just blocks on task spawns.
#[derive(Default)]
pub struct SyncRunner {
    pub running: HashMap<TaskID, Box<dyn Executable>>,
    pub results: HashMap<TaskID, Result<YieldUpdate>>,
}

/// TODO
#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct ThreadPoolRunner {}
