//! # Storage Abstraction
//!
//! Provides pluggable storage backends for game state solutions, decoupling
//! game logic from persistence implementation.

use anyhow::Result;

use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::RwLock;
use std::sync::Arc;

use crate::game::State;

/* TRAITS */

/// Storage abstraction for persisting solution records.
///
/// Implementations must provide get/put operations for key-value storage
/// where keys are game states and values are solution records of type R.
pub trait Storage<R>: Send + Sync {
    /// Retrieves a solution record for the given state, if it exists.
    fn get(&self, state: &State) -> Result<Option<R>>;

    /// Stores a solution record for the given state.
    fn put(&self, state: &State, record: &R) -> Result<()>;

    /// Flushes any buffered writes to persistent storage.
    fn flush(&self) -> Result<()>;
}

/* API STRUCTURES */

/// RocksDB-backed persistent storage.
pub struct RocksDBStorage<R> {
    _phantom: PhantomData<R>,
    db: Arc<rocksdb::DB>,
}

/// In-memory storage for testing (non-persistent).
pub struct InMemoryStorage<R> {
    data: Arc<RwLock<HashMap<Vec<u8>, R>>>,
}

/* IMPLEMENTATIONS */

impl<R> RocksDBStorage<R> {
    pub fn new(db: Arc<rocksdb::DB>) -> Self {
        Self {
            _phantom: PhantomData,
            db,
        }
    }
}

impl<R> Storage<R> for RocksDBStorage<R>
where
    R: From<Vec<u8>> + Into<Vec<u8>> + Clone + Send + Sync,
{
    fn get(&self, state: &State) -> Result<Option<R>> {
        let key = state.as_raw_slice();
        Ok(self.db.get(key)?.map(R::from))
    }

    fn put(&self, state: &State, record: &R) -> Result<()> {
        let key = state.as_raw_slice();
        let value: Vec<u8> = record.clone().into();
        self.db.put(key, value)?;
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}

impl<R> InMemoryStorage<R> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<R> Default for InMemoryStorage<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> Storage<R> for InMemoryStorage<R>
where
    R: Clone + Send + Sync,
{
    fn get(&self, state: &State) -> Result<Option<R>> {
        let key = state.as_raw_slice().to_vec();
        let data = self.data.read().unwrap();
        Ok(data.get(&key).cloned())
    }

    fn put(&self, state: &State, record: &R) -> Result<()> {
        let key = state.as_raw_slice().to_vec();
        self.data.write().unwrap().insert(key, record.clone());
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

/* TESTS */

#[cfg(test)]
mod tests {
    use bitvec::field::BitField;
    use bitvec::order::Msb0;
    use bitvec::vec::BitVec;

    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    struct TestRecord {
        value: i64,
    }

    impl From<Vec<u8>> for TestRecord {
        fn from(bytes: Vec<u8>) -> Self {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&bytes[..8.min(bytes.len())]);
            Self {
                value: i64::from_be_bytes(arr),
            }
        }
    }

    impl From<TestRecord> for Vec<u8> {
        fn from(val: TestRecord) -> Self {
            val.value.to_be_bytes().to_vec()
        }
    }

    #[test]
    fn inmemory_get_put() -> Result<()> {
        let storage = InMemoryStorage::new();
        let mut state: BitVec<u8, Msb0> = BitVec::repeat(false, 64);
        state.store_be(42u64);

        let record = TestRecord { value: 100 };
        storage.put(&state, &record)?;

        let retrieved = storage.get(&state)?;
        assert_eq!(retrieved, Some(record));

        Ok(())
    }

    #[test]
    fn inmemory_get_nonexistent() -> Result<()> {
        let storage: InMemoryStorage<TestRecord> = InMemoryStorage::new();
        let mut state: BitVec<u8, Msb0> = BitVec::repeat(false, 64);
        state.store_be(99u64);

        let retrieved = storage.get(&state)?;
        assert_eq!(retrieved, None);

        Ok(())
    }

    #[test]
    fn inmemory_overwrite() -> Result<()> {
        let storage = InMemoryStorage::new();
        let mut state: BitVec<u8, Msb0> = BitVec::repeat(false, 64);
        state.store_be(10u64);

        storage.put(&state, &TestRecord { value: 50 })?;
        storage.put(&state, &TestRecord { value: 75 })?;

        let retrieved = storage.get(&state)?;
        assert_eq!(retrieved, Some(TestRecord { value: 75 }));

        Ok(())
    }

    #[test]
    fn inmemory_flush_noop() -> Result<()> {
        let storage: InMemoryStorage<TestRecord> = InMemoryStorage::new();
        storage.flush()?;
        Ok(())
    }
}
