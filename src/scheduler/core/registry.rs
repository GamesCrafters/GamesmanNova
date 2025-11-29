//! # Task Registry
//!
//! HashMap wrapper with cycle detection and dependency operations.

use anyhow::Result;
use anyhow::bail;

use std::collections::HashMap;
use std::collections::HashSet;

use crate::scheduler::TaskID;
use crate::scheduler::core::context::AnyContext;

/* STRUCTURES */

/// Task registry with cycle detection.
pub struct Registry {
    buffer: HashMap<TaskID, AnyContext>,
}

/* IMPLEMENTATIONS */

impl Registry {
    pub fn new() -> Self {
        Self {
            buffer: HashMap::new(),
        }
    }

    pub fn get(&self, id: &TaskID) -> Option<&AnyContext> {
        self.buffer.get(id)
    }

    pub fn get_mut(&mut self, id: &TaskID) -> Option<&mut AnyContext> {
        self.buffer.get_mut(id)
    }

    pub fn insert(&mut self, id: TaskID, ctx: AnyContext) {
        self.buffer.insert(id, ctx);
    }

    pub fn remove(&mut self, id: &TaskID) -> Option<AnyContext> {
        self.buffer.remove(id)
    }

    pub fn contains(&self, id: &TaskID) -> bool {
        self.buffer.contains_key(id)
    }

    pub fn keys(&self) -> impl Iterator<Item = &TaskID> {
        self.buffer.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &AnyContext> {
        self.buffer.values()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&TaskID, &AnyContext)> {
        self.buffer.iter()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(cycle) = self.find_cycle() {
            bail!("Dependency cycle: {}", format_cycle(&cycle));
        }
        Ok(())
    }

    pub fn find_cycle(&self) -> Option<Vec<TaskID>> {
        let mut seen = HashSet::new();
        self.keys()
            .find_map(|id| {
                (!seen.contains(id)).then(|| {
                    let mut stack = Vec::new();
                    self.find_cycle_from(*id, &mut seen, &mut stack)
                })
            })
            .flatten()
    }

    fn find_cycle_from(
        &self,
        start: TaskID,
        seen: &mut HashSet<TaskID>,
        stack: &mut Vec<TaskID>,
    ) -> Option<Vec<TaskID>> {
        stack.push(start);
        seen.insert(start);

        let deps = self.waiting_dependencies(&start)?;
        let cycle = self.check_dependencies(deps, seen, stack);

        stack.pop();
        cycle
    }

    fn waiting_dependencies(&self, id: &TaskID) -> Option<&HashSet<TaskID>> {
        self.get(id)
            .and_then(|ctx| match ctx {
                AnyContext::Waiting(waiting) => Some(waiting.dependencies()),
                _ => None,
            })
    }

    fn check_dependencies(
        &self,
        deps: &HashSet<TaskID>,
        seen: &mut HashSet<TaskID>,
        stack: &[TaskID],
    ) -> Option<Vec<TaskID>> {
        deps.iter()
            .find_map(|dep| self.check_dependency(*dep, seen, stack))
    }

    fn check_dependency(
        &self,
        dep: TaskID,
        seen: &mut HashSet<TaskID>,
        stack: &[TaskID],
    ) -> Option<Vec<TaskID>> {
        if !seen.contains(&dep) {
            return self.find_cycle_from(dep, seen, &mut stack.to_vec());
        }

        if !stack.contains(&dep) {
            return None;
        }

        let cycle_start = stack
            .iter()
            .position(|id| *id == dep)?;
        let mut cycle = stack[cycle_start..].to_vec();
        cycle.push(dep);
        Some(cycle)
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/* UTILITIES */

fn format_cycle(cycle: &[TaskID]) -> String {
    cycle
        .iter()
        .map(|id| format!("{}", id))
        .collect::<Vec<_>>()
        .join(" -> ")
}
