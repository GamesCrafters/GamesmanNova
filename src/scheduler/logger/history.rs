//! # History Logger
//!
//! Records full scheduler snapshots and transitions at each tick
//! for post-mortem analysis and testing.

use anyhow::Result;
use derive_builder::Builder;

use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::traits::Logger;

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

impl Logger for HistoryLogger {
    fn report(
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

/* TEST UTILITIES */

#[cfg(test)]
pub mod test_utils {

    use crate::scheduler::SchedulerSnapshot;
    use crate::scheduler::TaskContextSnapshot;
    use crate::scheduler::TaskID;
    use crate::scheduler::TaskState;
    use crate::scheduler::Transition;

    use super::*;

    impl HistoryLogger {
        pub(in crate::scheduler) fn snapshots(&self) -> &[SchedulerSnapshot] {
            &self.snapshots
        }

        pub(in crate::scheduler) fn state(
            &self,
            tick: u64,
            tid: TaskID,
        ) -> Option<TaskState> {
            let snapshot = self
                .snapshots
                .iter()
                .find(|s| s.tick == tick)?;

            let ctx = snapshot.tasks.get(&tid)?;
            Some(ctx.state.clone())
        }

        pub(in crate::scheduler) fn transitions(
            &self,
            tid: TaskID,
        ) -> Vec<Transition> {
            let component = tid.component;
            let filter = |t: &Transition| t.task == component;
            self.snapshots
                .iter()
                .flat_map(|s| s.transitions.iter())
                .filter(|t| filter(t))
                .cloned()
                .collect()
        }

        pub(in crate::scheduler) fn find<F>(
            &self,
            predicate: F,
        ) -> Option<TaskID>
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

        pub(in crate::scheduler) fn filter<F>(
            &self,
            predicate: F,
        ) -> Vec<TaskID>
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

        /// Returns true if tid1 started running before tid2.
        pub(in crate::scheduler) fn before(
            &self,
            tid1: TaskID,
            tid2: TaskID,
        ) -> bool {
            let comp1 = tid1.component;
            let comp2 = tid2.component;
            let relevant = |t: &&Transition| {
                let running = matches!(t.to, TaskState::Running);
                running && (t.task == comp1 || t.task == comp2)
            };

            self.snapshots
                .iter()
                .flat_map(|s| &s.transitions)
                .find(relevant)
                .map(|t| t.task == comp1)
                .unwrap_or(false)
        }

        /// Returns true if task was preempted during execution.
        pub(in crate::scheduler) fn was_preempted(&self, tid: TaskID) -> bool {
            let component = tid.component;
            self.snapshots
                .iter()
                .flat_map(|s| &s.transitions)
                .any(|t| {
                    t.task == component && matches!(t.to, TaskState::Preempting)
                })
        }

        /// Returns true if tasks started running in the given order.
        pub(in crate::scheduler) fn execution_order(
            &self,
            tids: &[TaskID],
        ) -> bool {
            tids.windows(2)
                .all(|window| self.before(window[0], window[1]))
        }

        /// Returns true if tid1 completed before tid2 started running.
        pub(in crate::scheduler) fn completed_before(
            &self,
            tid1: TaskID,
            tid2: TaskID,
        ) -> bool {
            let finished = |s: &TaskState| matches!(s, TaskState::Suspended(_));
            let running = |s: &TaskState| matches!(s, TaskState::Running);
            let finish1 = self.find_state_transition(tid1, finished);
            let start2 = self.find_state_transition(tid2, running);
            self.transition_before(finish1, start2)
        }

        /* HELPERS */

        fn find_state_transition<F>(
            &self,
            tid: TaskID,
            predicate: F,
        ) -> Option<(u64, usize)>
        where
            F: Fn(&TaskState) -> bool,
        {
            let component = tid.component;
            self.snapshots
                .iter()
                .flat_map(|s| {
                    s.transitions
                        .iter()
                        .enumerate()
                        .map(move |(order, t)| (s.tick, order, t))
                })
                .find(|(_, _, t)| t.task == component && predicate(&t.to))
                .map(|(tick, order, _)| (tick, order))
        }

        fn transition_before(
            &self,
            t1: Option<(u64, usize)>,
            t2: Option<(u64, usize)>,
        ) -> bool {
            match (t1, t2) {
                (Some((tick1, order1)), Some((tick2, order2))) => {
                    tick1 < tick2 || (tick1 == tick2 && order1 < order2)
                },
                _ => false,
            }
        }
    }
}
