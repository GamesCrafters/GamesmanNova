//! # Command Line Module
//!
//! This module offers UNIX-like CLI tooling in order to facilitate scripting
//! and ergonomic use of GamesmanNova. This uses the
//! [clap](https://docs.rs/clap/latest/clap/) crate to provide standard
//! behavior, which is outlined in [this](https://clig.dev/) great guide.

use clap::{Args, Parser, Subcommand};
use serde_json::Value;

use crate::interface::{InfoFormat, TargetAttribute};
use crate::target::model::TargetModule;

/* COMMAND LINE INTERFACE */

/// TODO
#[derive(Parser)]
#[command(author, version, about, long_about = None, propagate_version = true)]
pub struct Cli {
    /* REQUIRED COMMANDS */
    /// Available subcommands for the main 'nova' command.
    #[command(subcommand)]
    pub command: Commands,

    /* DEFAULTS PROVIDED */
    /// Send no output to STDOUT during successful execution.
    #[arg(short, long, group = "output")]
    pub quiet: bool,
}

/// Subcommand choices, specified as `nova <subcommand>`.
#[derive(Subcommand)]
pub enum Commands {
    /// Extract a set of features from a target.
    Extract(ExtractArgs),

    /// Frame a collection of features in a data format.
    Frame(FrameArgs),

    /// Provides information about the system's offerings.
    Info(InfoArgs),
}

/* ARGUMENT AND OPTION DEFINITIONS */

/// TODO
#[derive(Args)]
pub struct ExtractArgs {
    /* REQUIRED ARGUMENTS */
    /// Target name.
    pub target: TargetModule,

    /// Set of features to extract from target.
    #[arg(short, long, value_delimiter = ',', num_args(1..))]
    pub features: Vec<String>,

    /* OPTIONAL ARGUMENTS */
    #[arg(short, long)]
    pub config: Option<Value>,

    /// Skips prompts for confirming destructive operations.
    #[arg(short, long)]
    pub yes: bool,
}

/// TODO
#[derive(Args)]
pub struct FrameArgs {
    /* REQUIRED ARGUMENTS */
    /// Target name.
    pub target: TargetModule,

    /// Set of features to collect.
    #[arg(short, long, num_args(1..))]
    pub features: Vec<String>,

    /* OPTIONAL ARGUMENTS */
    #[arg(short, long)]
    pub config: Option<Value>,

    /// Skips prompts for confirming destructive operations.
    #[arg(short, long)]
    pub yes: bool,
}

/// TODO
#[derive(Args)]
pub struct InfoArgs {
    /// Specify the target to provide information about.
    pub target: TargetModule,

    /// Specify which of the target's attributes to provide information about.
    #[arg(short, long, value_delimiter = ',', num_args(1..))]
    pub attributes: Vec<TargetAttribute>,

    /* OPTIONAL ARGUMENTS */
    /// Format in which to send output to STDOUT.
    #[arg(short, long, default_value_t = InfoFormat::Legible)]
    pub output: InfoFormat,
}
