//! # Backward Scheduler Task
//!
//! Retrograde analysis task for solving game state graphs. Propagates values
//! backward from terminal states to compute optimal play.

use std::any::Any;
use std::collections::VecDeque;
use std::sync::Arc;

use anyhow::Result;

use crate::database::storage::Storage;
use crate::game::Component;
use crate::game::State;
use crate::game::traits::Implicit;
use crate::game::traits::Transpose;
use crate::scheduler::TaskCategory;
use crate::scheduler::TaskID;
use crate::scheduler::TaskIDBuilder;
use crate::scheduler::TaskOutcome;
use crate::scheduler::TaskOutcomes;
use crate::scheduler::YieldIntention;
use crate::scheduler::YieldUpdate;
use crate::scheduler::YieldUpdateBuilder;
use crate::scheduler::traits::Executable;

/* STRUCTURES */

pub struct BackwardTask<G, R> {
    component: Component,
    frontier: VecDeque<State>,
    storage: Arc<dyn Storage<R>>,
    game: G,
}

/* IMPLEMENTATIONS */

impl<G, R> BackwardTask<G, R>
where
    G: Implicit + Transpose + Clone + Send + 'static,
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
            frontier,
            storage,
            game,
        })
    }
}

impl<G, R> Executable for BackwardTask<G, R>
where
    G: Implicit + Transpose + Clone + Send + 'static,
    R: Default + Into<Vec<u8>> + Clone + Send + Sync + 'static,
{
    fn tick(&mut self, _deps: TaskOutcomes) -> Option<YieldUpdate> {
        let outcome = TaskOutcome::Success(0);
        Some(
            YieldUpdateBuilder::default()
                .intention(YieldIntention::Suspended(outcome))
                .discovered(Vec::new())
                .build()
                .expect("Failed to build suspended yield"),
        )
    }

    fn size(&self) -> Option<u64> {
        None
    }

    fn progress(&self) -> Option<u64> {
        None
    }

    fn id(&self) -> TaskID {
        TaskIDBuilder::default()
            .category(TaskCategory::Solve)
            .component(self.component)
            .build()
            .expect("TaskID builder should not fail with all fields provided")
    }
}

/* DOWNCAST UTILITIES */

pub(super) trait ExecutableExt {
    fn as_any(&self) -> &dyn Any;
}

impl<G, R> ExecutableExt for BackwardTask<G, R>
where
    G: Implicit + Transpose + Clone + Send + 'static,
    R: Default + Into<Vec<u8>> + Clone + Send + Sync + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ExecutableExt for dyn Executable {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}
