//! # Tabulate Scheduler Task
//!
//! Database persistence task for storing solved game state values. Writes
//! computed solutions to SQLite for downstream analysis.

use std::collections::VecDeque;
use std::sync::Arc;

use anyhow::Result;

use crate::database::storage::Storage;
use crate::game::Component;
use crate::game::State;
use crate::game::traits::Variable;
use crate::scheduler::TaskCategory;
use crate::scheduler::TaskID;
use crate::scheduler::TaskIDBuilder;
use crate::scheduler::TaskOutcome;
use crate::scheduler::TaskOutcomes;
use crate::scheduler::traits::Executable;
use crate::scheduler::traits::YieldIntention;
use crate::scheduler::traits::YieldUpdate;

/* STRUCTURES */

pub struct TabulateTask<G, R> {
    component: Component,
    _frontier: VecDeque<State>,
    _storage: Arc<dyn Storage<R>>,
    _game: G,
}

/* IMPLEMENTATIONS */

impl<G, R> TabulateTask<G, R>
where
    G: Variable + Clone + Send + 'static,
    R: Default + Into<Vec<u8>> + Clone + Send + Sync + 'static,
{
    pub fn new(
        component: Component,
        game: G,
        frontier: VecDeque<State>,
        storage: Arc<dyn Storage<R>>,
    ) -> Result<Self> {
        Ok(Self {
            component,
            _frontier: frontier,
            _storage: storage,
            _game: game,
        })
    }
}

impl<G, R> Executable for TabulateTask<G, R>
where
    G: Variable + Clone + Send + 'static,
    R: Default + Into<Vec<u8>> + Clone + Send + Sync + 'static,
{
    fn tick(&mut self, _deps: TaskOutcomes) -> Option<YieldUpdate> {
        let outcome = TaskOutcome::Success(0);
        Some(YieldUpdate {
            intention: YieldIntention::Suspended(outcome),
            discovered: Vec::new(),
        })
    }

    fn size(&self) -> Option<u64> {
        None
    }

    fn progress(&self) -> Option<u64> {
        None
    }

    fn id(&self) -> TaskID {
        TaskIDBuilder::default()
            .category(TaskCategory::Store)
            .component(self.component)
            .build()
            .expect("TaskID builder should not fail with all fields provided")
    }
}
