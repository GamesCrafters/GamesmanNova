//! Formatting utilities for dashboard display.

use crate::game::Component;
use crate::scheduler::SchedulerSnapshot;
use crate::scheduler::TaskCategory;
use crate::scheduler::TaskID;

/* FORMATTING */

pub fn format_task_id(category: &TaskCategory, component: Component) -> String {
    format!("{:?}({})", category, component)
}

pub fn format_task_ids(tasks: &[(TaskCategory, Component)]) -> String {
    tasks
        .iter()
        .map(|(cat, comp)| format_task_id(cat, *comp))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn format_deps(
    dep_ids: &[TaskID],
    snapshot: &SchedulerSnapshot,
    max_deps_display: usize,
) -> String {
    if dep_ids.is_empty() {
        return String::new();
    }

    let dep_info: Vec<(TaskCategory, Component)> = dep_ids
        .iter()
        .filter_map(|id| {
            snapshot
                .tasks
                .get(id)
                .map(|ctx| (ctx.category, ctx.component))
        })
        .collect();

    if dep_info.len() <= max_deps_display {
        format!("Waits on: {}", format_task_ids(&dep_info))
    } else {
        let shown = &dep_info[..max_deps_display];
        let remaining = dep_info.len() - max_deps_display;
        format!(
            "Waits on: {}, ... (+{} more)",
            format_task_ids(shown),
            remaining
        )
    }
}

pub fn format_duration(seconds: f64) -> String {
    if seconds < 60.0 {
        format!("{:.0}s", seconds)
    } else if seconds < 3600.0 {
        let mins = (seconds / 60.0) as u64;
        let secs = (seconds % 60.0) as u64;
        if secs > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}m", mins)
        }
    } else if seconds < 86400.0 {
        let hours = (seconds / 3600.0) as u64;
        let mins = ((seconds % 3600.0) / 60.0) as u64;
        if mins > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}h", hours)
        }
    } else {
        let days = (seconds / 86400.0) as u64;
        let hours = ((seconds % 86400.0) / 3600.0) as u64;
        if hours > 0 {
            format!("{}d {}h", days, hours)
        } else {
            format!("{}d", days)
        }
    }
}
