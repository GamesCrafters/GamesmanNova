//! # Tabulate Scheduler Task
//!
//! Database persistence task for storing solved game state values. Writes
//! computed solutions to SQLite for downstream analysis.

use std::any::Any;
use std::collections::VecDeque;
use std::marker::PhantomData;

use anyhow::Result;

use crate::game::Component;
use crate::game::DEFAULT_STATE_BYTES;
use crate::game::PlayerCount;
use crate::game::State;
use crate::game::traits::Variable;
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

pub struct TabulateTask<
    G,
    const N: PlayerCount,
    const B: usize = DEFAULT_STATE_BYTES,
> {
    component: Component,
    _players: PhantomData<[(); N]>,
    frontier: VecDeque<State<B>>,
    db: sled::Db,
    game: G,
}

/* IMPLEMENTATIONS */

impl<G, const N: PlayerCount, const B: usize> TabulateTask<G, N, B>
where
    G: Variable,
{
    pub fn new(
        component: Component,
        game: G,
        frontier: VecDeque<State<B>>,
        db: sled::Db,
    ) -> Result<Self> {
        Ok(Self {
            _players: PhantomData,
            component,
            frontier,
            game,
            db,
        })
    }
}

impl<G, const N: PlayerCount, const B: usize> Executable
    for TabulateTask<G, N, B>
where
    G: Variable + Send + 'static,
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
            .category(TaskCategory::Store)
            .component(self.component)
            .build()
            .expect("TaskID builder should not fail with all fields provided")
    }
}

/* DOWNCAST UTILITIES */

pub(super) trait ExecutableExt {
    fn as_any(&self) -> &dyn Any;
}

impl<G, const N: PlayerCount, const B: usize> ExecutableExt
    for TabulateTask<G, N, B>
where
    G: Variable + 'static,
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
