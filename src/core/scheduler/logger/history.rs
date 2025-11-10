//! # History Logger
//!
//! TODO

use anyhow::Result;
use derive_builder::Builder;

use crate::core::scheduler::SchedulerSnapshot;
use crate::core::scheduler::TaskContextSnapshot;
use crate::core::scheduler::TaskID;
use crate::core::scheduler::TaskState;
use crate::core::scheduler::Transition;
use crate::traits::scheduler::Logger;

/* STRUCTURES */

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct HistoryLogger {
    #[builder(default = "1")]
    frequency: usize,

    #[builder(default = "false")]
    lazy: bool,

    #[builder(default)]
    #[builder(setter(skip))]
    changes: usize,

    #[builder(default)]
    #[builder(setter(skip))]
    snapshots: Vec<SchedulerSnapshot>,
}

/* IMPLEMENTATIONS */

impl HistoryLogger {
    pub fn snapshots(&self) -> &[SchedulerSnapshot] {
        &self.snapshots
    }

    pub fn state(&self, tick: u64, tid: TaskID) -> Option<TaskState> {
        let snapshot = self
            .snapshots
            .iter()
            .find(|s| s.tick == tick)?;

        let ctx = snapshot.tasks.get(&tid)?;
        Some(ctx.state.clone())
    }

    pub fn transitions(&self, tid: TaskID) -> Vec<Transition> {
        let filter = |t: &Transition| t.task == tid;
        self.snapshots
            .iter()
            .flat_map(|s| s.transitions.iter())
            .filter(|t| filter(t))
            .cloned()
            .collect()
    }

    pub fn find<F>(&self, predicate: F) -> Option<TaskID>
    where
        F: Fn(&TaskContextSnapshot) -> bool,
    {
        let latest = self.snapshots.last()?;
        latest
            .tasks
            .iter()
            .find(|(_, ctx)| predicate(ctx))
            .map(|(tid, _)| *tid)
    }

    pub fn filter<F>(&self, predicate: F) -> Vec<TaskID>
    where
        F: Fn(&TaskContextSnapshot) -> bool,
    {
        let Some(latest) = self.snapshots.last() else {
            return Vec::new();
        };

        latest
            .tasks
            .iter()
            .filter(|(_, ctx)| predicate(ctx))
            .map(|(tid, _)| *tid)
            .collect()
    }

    pub fn before(&self, tid1: TaskID, tid2: TaskID) -> bool {
        let t1 = self.running_tick(tid1, &self.snapshots);
        let t2 = self.running_tick(tid2, &self.snapshots);
        matches!((t1, t2), (Some(tick1), Some(tick2)) if tick1 < tick2)
    }

    /* HELPERS */

    fn running_tick(
        &self,
        tid: TaskID,
        snapshots: &[SchedulerSnapshot],
    ) -> Option<u64> {
        snapshots
            .iter()
            .flat_map(|s| {
                s.transitions
                    .iter()
                    .map(move |t| (s.tick, t))
            })
            .find(|(_, t)| t.task == tid && matches!(t.to, TaskState::Running))
            .map(|(tick, _)| tick)
    }
}

impl Logger for HistoryLogger {
    fn observe(
        &mut self,
        snapshot: &SchedulerSnapshot,
        changed: bool,
    ) -> Result<()> {
        let record = match (self.lazy, changed) {
            (true, false) => false,
            (false, _) => snapshot
                .tick
                .is_multiple_of(self.frequency as u64),
            (true, true) => {
                self.changes += 1;
                self.changes
                    .is_multiple_of(self.frequency)
            },
        };

        if record {
            self.snapshots
                .push(snapshot.clone());
        }

        Ok(())
    }
}
