//! # Sync Runner
//!
//! Synchronous runner that just blocks on task spawns.

use anyhow::Result;

use std::collections::HashMap;

use crate::traits::scheduler::Executable;
use crate::types::scheduler::TaskID;
use crate::types::scheduler::YieldUpdate;

/* TYPE */

/// Synchronous runner that just blocks on task spawns.
#[derive(Default)]
pub struct SyncRunner {
    pub running: HashMap<TaskID, Box<dyn Executable>>,
    pub results: HashMap<TaskID, Result<YieldUpdate>>,
}
