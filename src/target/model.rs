//! # Extraction Target Data Models Module
//!
//! Provides definitions for types used in extraction target interfaces.

use clap::ValueEnum;

/// The default number of bytes used to encode states.
pub const DEFAULT_STATE_BYTES: usize = 8;

/// Unique identifier of a particular state in a target.
pub type State<const B: usize = DEFAULT_STATE_BYTES> = [u8; B];

/// String encoding some specific target's variant.
pub type Variant = String;

/// The name associated with a feature.
pub type FeatureName<'a> = &'a str;

/// The name associated with an extractor.
pub type ExtractorName<'a> = &'a str;

// Specifies the target offerings available through all interfaces.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum TargetModule {
    Crossteaser,
    ZeroBy,
}
