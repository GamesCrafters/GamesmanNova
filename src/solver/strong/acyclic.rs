//! # Strong Acyclic Solving Module
//!
//! This module implements strong acyclic solving routines.
//!
//! #### Authorship
//!
//! - Max Fierro, 12/3/2023 (maxfierro@berkeley.edu)

use anyhow::{Context, Result};
use bitvec::prelude::*;

use crate::database::{volatile, Record};
use crate::database::{Attribute, Datatype, SchemaBuilder};
use crate::database::{KVStore, Tabular};
use crate::game::{Acyclic, Bounded, DTransition, STransition, Solvable};
use crate::interface::IOMode;
use crate::model::{PlayerCount, Remoteness, State, Turn, Utility};
use crate::solver::{util, MAX_TRANSITIONS};

/* CONSTANTS */

/// The exact number of bits that are used to encode remoteness.
const REMOTENESS_SIZE: usize = 16;

/// The maximum number of bits that can be used to encode a record.
const BUFFER_SIZE: usize = 128;

/// The exact number of bits that are used to encode utility for one player.
const UTILITY_SIZE: usize = 8;

/* SOLVERS */

pub fn dynamic_solver<const N: usize, G>(game: &G, mode: IOMode) -> Result<()>
where
    G: Acyclic<N> + DTransition<State> + Bounded<State> + Solvable<N>,
{
    let mut db = volatile_database(game)
        .context("Failed to initialize volatile database.")?;
    dynamic_backward_induction(&mut db, game)
        .context("Failed solving algorithm execution.")?;
    Ok(())
}

pub fn static_solver<const N: usize, G>(game: &G, mode: IOMode) -> Result<()>
where
    G: Acyclic<N>
        + STransition<State, MAX_TRANSITIONS>
        + Bounded<State>
        + Solvable<N>,
{
    let mut db = volatile_database(game)
        .context("Failed to initialize volatile database.")?;
    static_backward_induction(&mut db, game)
        .context("Failed solving algorithm execution.")?;
    Ok(())
}

/* DATABASE INITIALIZATION */

/// Initializes a volatile database, creating a table schema according to the
/// solver record layout, initializing a table with that schema, and switching
/// to that table before returning the database handle.
fn volatile_database<const N: usize, G>(game: &G) -> Result<volatile::Database>
where
    G: Solvable<N>,
{
    let db = volatile::Database::initialize();
    let mut schema = SchemaBuilder::new();

    for i in 0..game.players() {
        let name = &format!("P{} utility", i);
        let data = Datatype::SINT;
        let size = UTILITY_SIZE;
        schema = schema
            .add(Attribute::new(name, data, size))
            .context("Failed to add utility attribute to database schema.")?;
    }

    let name = "State remoteness";
    let data = Datatype::UINT;
    let size = REMOTENESS_SIZE;
    schema = schema
        .add(Attribute::new(name, data, size))
        .context("Failed to add remoteness attribute to database schema.")?;

    let id = game.id();
    let schema = schema.build();
    db.create_table(&id, schema)
        .context("Failed to create database table for solution set.")?;
    db.select_table(&id)
        .context("Failed to select solution set database table.")?;

    Ok(db)
}

/* SOLVING ALGORITHMS */

/// Performs an iterative depth-first traversal of the game tree, assigning to
/// each game `state` a remoteness and utility values for each player within
/// `db`. This uses heap-allocated memory for keeping a stack of positions to
/// facilitate DFS, as well as for communicating state transitions.
fn dynamic_backward_induction<const N: PlayerCount, D, G>(
    db: &mut D,
    game: &G,
) -> Result<()>
where
    D: KVStore<RecordBuffer>,
    G: Acyclic<N> + DTransition<State> + Bounded<State> + Solvable<N>,
{
    let mut stack = Vec::new();
    stack.push(game.start());
    while let Some(curr) = stack.pop() {
        let children = game.prograde(curr);
        let mut buf = RecordBuffer::new(game.players())
            .context("Failed to create placeholder record.")?;
        if db.get(curr).is_none() {
            db.put(curr, &buf);
            if game.end(curr) {
                buf = RecordBuffer::new(game.players())
                    .context("Failed to create record for end state.")?;
                buf.set_utility(game.utility(curr))
                    .context("Failed to copy utility values to record.")?;
                buf.set_remoteness(0)
                    .context("Failed to set remoteness for end state.")?;
                db.put(curr, &buf);
            } else {
                stack.push(curr);
                stack.extend(
                    children
                        .iter()
                        .filter(|&x| db.get(*x).is_none()),
                );
            }
        } else {
            let mut optimal = buf;
            let mut max_val = Utility::MIN;
            let mut min_rem = Remoteness::MAX;
            for state in children {
                let buf = RecordBuffer::from(db.get(state).unwrap())
                    .context("Failed to create record for middle state.")?;
                let val = buf
                    .get_utility(game.turn(state))
                    .context("Failed to get utility from record.")?;
                let rem = buf.get_remoteness();
                if val > max_val || (val == max_val && rem < min_rem) {
                    max_val = val;
                    min_rem = rem;
                    optimal = buf;
                }
            }
            optimal
                .set_remoteness(min_rem + 1)
                .context("Failed to set remoteness for solved record.")?;
            db.put(curr, &optimal);
        }
    }
    Ok(())
}

/// Performs an iterative depth-first traversal of the `game` tree, assigning to
/// each `game` state a remoteness and utility values for each player within
/// `db`. This uses heap-allocated memory for keeping a stack of positions to
/// facilitate DFS, and stack memory for communicating state transitions.
fn static_backward_induction<const N: PlayerCount, D, G>(
    db: &mut D,
    game: &G,
) -> Result<()>
where
    D: KVStore<RecordBuffer>,
    G: Acyclic<N>
        + STransition<State, MAX_TRANSITIONS>
        + Bounded<State>
        + Solvable<N>,
{
    let mut stack = Vec::new();
    stack.push(game.start());
    while let Some(curr) = stack.pop() {
        let children = game.prograde(curr);
        let mut buf = RecordBuffer::new(game.players())
            .context("Failed to create placeholder record.")?;
        if db.get(curr).is_none() {
            db.put(curr, &buf);
            if game.end(curr) {
                buf = RecordBuffer::new(game.players())
                    .context("Failed to create record for end state.")?;
                buf.set_utility(game.utility(curr))
                    .context("Failed to copy utility values to record.")?;
                buf.set_remoteness(0)
                    .context("Failed to set remoteness for end state.")?;
                db.put(curr, &buf);
            } else {
                stack.push(curr);
                stack.extend(
                    children
                        .iter()
                        .filter_map(|&x| x)
                        .filter(|&x| db.get(x).is_none()),
                );
            }
        } else {
            let mut cur = 0;
            let mut optimal = buf;
            let mut max_val = Utility::MIN;
            let mut min_rem = Remoteness::MAX;
            while cur < MAX_TRANSITIONS {
                cur += 1;
                if let Some(state) = children[cur] {
                    let buf = RecordBuffer::from(db.get(state).unwrap())
                        .context("Failed to create record for middle state.")?;
                    let val = buf
                        .get_utility(game.turn(state))
                        .context("Failed to get utility from record.")?;
                    let rem = buf.get_remoteness();
                    if val > max_val || (val == max_val && rem < min_rem) {
                        max_val = val;
                        min_rem = rem;
                        optimal = buf;
                    }
                }
            }
            optimal
                .set_remoteness(min_rem + 1)
                .context("Failed to set remoteness for solved record.")?;
            db.put(curr, &optimal);
        }
    }
    Ok(())
}

/* RECORD IMPLEMENTATION */

/// Solver-specific record entry, meant to communicate the remoteness and each
/// player's utility at a corresponding game state. The layout is as follows:
///
/// ```none
/// [UTILITY_SIZE bits: P0 utility]
/// ...
/// [UTILITY_SIZE bits: P(N-1) utility]
/// [REMOTENESS_SIZE bits: Remoteness]
/// [0b0 until BUFFER_SIZE]
/// ```
///
/// The number of players `N` is limited by `BUFFER_SIZE`, because a statically
/// sized buffer is used for intermediary storage. The utility and remoteness
/// values are encoded in big-endian, with utility being a signed two's
/// complement integer and remoteness an unsigned integer.
struct RecordBuffer {
    buf: BitArr!(for BUFFER_SIZE, in u8, Msb0),
    players: PlayerCount,
}

impl Record for RecordBuffer {
    #[inline(always)]
    fn raw(&self) -> &BitSlice<u8, Msb0> {
        &self.buf[..Self::bit_size(self.players)]
    }
}

impl RecordBuffer {
    /// Returns a new instance of a bit-packed record buffer that is able to
    /// store utility values for `players`. Fails if `players` is too high for
    /// the underlying buffer's capacity.
    #[inline(always)]
    fn new(players: PlayerCount) -> Result<Self> {
        if Self::bit_size(players) > BUFFER_SIZE {
            todo!()
        } else {
            Ok(Self {
                buf: bitarr!(u8, Msb0; 0; BUFFER_SIZE),
                players,
            })
        }
    }

    /// Return a new instance with `bits` as the underlying buffer. Fails in the
    /// event that the size of `bits` is incoherent with the record.
    #[inline(always)]
    fn from(bits: &BitSlice<u8, Msb0>) -> Result<Self> {
        let len = bits.len();
        if len > BUFFER_SIZE {
            todo!()
        } else if len < Self::minimum_bit_size() {
            todo!()
        } else {
            let players = Self::player_count(len);
            let mut buf = bitarr!(u8, Msb0; 0; BUFFER_SIZE);
            buf[..len].copy_from_bitslice(bits);
            Ok(Self { players, buf })
        }
    }

    /* GET METHODS */

    /// Parse and return the utility value corresponding to `player`. Fails if
    /// the `player` index passed in is incoherent with player count.
    #[inline(always)]
    fn get_utility(&self, player: Turn) -> Result<Utility> {
        if player >= self.players {
            todo!()
        } else {
            let start = Self::utility_index(self.players);
            let end = start + UTILITY_SIZE;
            Ok(self.buf[start..end].load_be::<Utility>())
        }
    }

    /// Parse and return the remoteness value in the record encoding. Failure
    /// here indicates corrupted state.
    #[inline(always)]
    fn get_remoteness(&self) -> Remoteness {
        let start = Self::remoteness_index(self.players);
        let end = start + REMOTENESS_SIZE;
        self.buf[start..end].load_be::<Remoteness>()
    }

    /* SET METHODS */

    /// Set this entry to have the utility values in `v` for each player. Fails
    /// if any of the utility values are too high to fit in the space dedicated
    /// for each player's utility, or if there is a mismatch between player
    /// count and the number of utility values passed in.
    #[inline(always)]
    fn set_utility<const N: usize>(&mut self, v: [Utility; N]) -> Result<()> {
        if N >= self.players {
            todo!()
        } else {
            let player = 0;
            while player < self.players {
                let utility = v[player];
                if util::min_sbits(utility) > UTILITY_SIZE {
                    todo!()
                }

                let start = Self::utility_index(player);
                let end = start + UTILITY_SIZE;
                self.buf[start..end].store_be(utility);
            }
            Ok(())
        }
    }

    /// Set this entry to have `value` remoteness. Fails if `value` is too high
    /// to fit in the space dedicated for remoteness within the record.
    #[inline(always)]
    fn set_remoteness(&mut self, value: Remoteness) -> Result<()> {
        if util::min_ubits(value) > REMOTENESS_SIZE {
            todo!()
        } else {
            let start = Self::remoteness_index(self.players);
            let end = start + REMOTENESS_SIZE;
            self.buf[start..end].store_be(value);
            Ok(())
        }
    }

    /* LAYOUT HELPER METHODS */

    /// Return the number of bits that would be needed to store a record
    /// containing utility information for `players` as well as remoteness.
    #[inline(always)]
    const fn bit_size(players: usize) -> usize {
        players * UTILITY_SIZE + REMOTENESS_SIZE
    }

    /// Return the minimum number of bits needed for a valid record buffer.
    #[inline(always)]
    const fn minimum_bit_size() -> usize {
        REMOTENESS_SIZE
    }

    /// Return the bit index of the remoteness entry start in the record buffer.
    #[inline(always)]
    const fn remoteness_index(players: usize) -> usize {
        players * UTILITY_SIZE
    }

    /// Return the bit index of the 'i'th player's utility entry start.
    #[inline(always)]
    const fn utility_index(player: Turn) -> usize {
        player * UTILITY_SIZE
    }

    /// Return the maximum number of utility entries supported by a dense record
    /// (one that maximizes bit usage) with `length`. Ignores unused bits.
    #[inline(always)]
    const fn player_count(length: usize) -> usize {
        length - REMOTENESS_SIZE / UTILITY_SIZE
    }
}
