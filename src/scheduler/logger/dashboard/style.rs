//! Styling helpers for dashboard TUI.

use ratatui::style::Color;

use crate::scheduler::TaskState;

/* CONSTANTS */

const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/* IMPLEMENTATIONS */

pub fn spinner(frame: usize) -> &'static str {
    SPINNER[frame % SPINNER.len()]
}

pub fn icon(state: &TaskState) -> &'static str {
    match state {
        TaskState::Suspended(_) => "✓",
        TaskState::Preempting => spinner(0),
        TaskState::Waiting(_) => "⏸",
        TaskState::Running => spinner(0),
        TaskState::Ready => "◯",
        TaskState::Error => "✗",
    }
}

pub fn badge(state: &TaskState) -> (&'static str, Color) {
    match state {
        TaskState::Suspended(_) => ("SUSPENDED", Color::Gray),
        TaskState::Waiting(_) => ("WAITING", Color::Cyan),
        TaskState::Preempting => ("PREEMPT", Color::Yellow),
        TaskState::Running => ("RUNNING", Color::Green),
        TaskState::Ready => ("READY", Color::Blue),
        TaskState::Error => ("ERROR", Color::Red),
    }
}
