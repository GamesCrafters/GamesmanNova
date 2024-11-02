//! # Database Data Models Module
//!
//! Provides definitions for types used in database interfaces.

use bitvec::order::Msb0;
use bitvec::slice::BitSlice;

/// A generic number used to differentiate between objects.
pub type SequenceKey = u64;

/// The type of a raw sequence of bits encoding a database value associated
/// with a key, backed by a [`BitSlice`] with [`u8`] big-endian storage.
pub type Value = BitSlice<u8, Msb0>;

/// The type of a database key per an implementation of [`KVStore`].
pub type Key = BitSlice<u8, Msb0>;
