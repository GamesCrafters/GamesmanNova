//! # History Logger
//!
//! Logger that captures complete scheduler state history for post-execution analysis.

use anyhow::Result;
use std::sync::Arc;
use std::sync::Mutex;

use crate::core::scheduler::SchedulerSnapshot;
use crate::core::scheduler::TaskContextSnapshot;
use crate::core::scheduler::TaskID;
use crate::core::scheduler::TaskState;
use crate::core::scheduler::Transition;
use crate::traits::scheduler::Logger;

pub struct HistoryLogger {
    snapshots: Arc<Mutex<Vec<SchedulerSnapshot>>>,
}

impl HistoryLogger {
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn snapshots(&self) -> Vec<SchedulerSnapshot> {
        self.snapshots
            .lock()
            .unwrap()
            .clone()
    }

    pub fn state(&self, tick: u64, tid: TaskID) -> Option<TaskState> {
        let snapshots = self
            .snapshots
            .lock()
            .unwrap();

        let snapshot = snapshots
            .iter()
            .find(|s| s.tick == tick)?;

        let ctx = snapshot
            .tasks
            .get(&tid)?;

        Some(ctx.progress.clone())
    }

    pub fn transitions(&self, tid: TaskID) -> Vec<Transition> {
        let snapshots = self
            .snapshots
            .lock()
            .unwrap();

        let filter = |t: &Transition| t.task == tid;
        snapshots
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
        let snapshots = self
            .snapshots
            .lock()
            .unwrap();

        let latest = snapshots.last()?;
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
        let snapshots = self
            .snapshots
            .lock()
            .unwrap();

        let Some(latest) = snapshots.last() else {
            return Vec::new();
        };

        latest
            .tasks
            .iter()
            .filter(|(_, ctx)| predicate(ctx))
            .map(|(tid, _)| *tid)
            .collect()
    }

    pub fn describe(&self, about: &str) -> Option<TaskID> {
        self.find(|ctx| ctx.about == about)
    }

    pub fn before(&self, tid1: TaskID, tid2: TaskID) -> bool {
        let snapshots = self
            .snapshots
            .lock()
            .unwrap();

        let find_first_running = |tid: TaskID| {
            for snapshot in snapshots.iter() {
                for transition in &snapshot.transitions {
                    if transition.task == tid
                        && matches!(transition.to, TaskState::Running)
                    {
                        return Some(snapshot.tick);
                    }
                }
            }
            None
        };

        let t1 = find_first_running(tid1);
        let t2 = find_first_running(tid2);

        match (t1, t2) {
            (Some(tick1), Some(tick2)) => tick1 < tick2,
            _ => false,
        }
    }
}

impl Clone for HistoryLogger {
    fn clone(&self) -> Self {
        Self {
            snapshots: Arc::clone(&self.snapshots),
        }
    }
}

impl Logger for HistoryLogger {
    fn observe(&mut self, snapshot: &SchedulerSnapshot, _changed: bool) -> Result<()> {
        self.snapshots
            .lock()
            .unwrap()
            .push(snapshot.clone());

        Ok(())
    }
}

impl Default for HistoryLogger {
    fn default() -> Self {
        Self::new()
    }
}
