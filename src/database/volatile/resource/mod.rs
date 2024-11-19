//! # Volatile Database Resource
//!
//! TODO

use anyhow::Result;
use bitvec::order::Msb0;
use bitvec::slice::BitSlice;
use bitvec::vec::BitVec;

use std::collections::HashMap;
use std::sync::{Arc, Weak};

use crate::database::model::Key;
use crate::database::model::SequenceKey;
use crate::database::model::Value;
use crate::database::volatile::ResourceID;
use crate::database::Map;
use crate::database::Record;
use crate::database::Relation;
use crate::database::Schema;

/* RE-EXPORTS */

pub use manager::Request;
pub use manager::ResourceManager;

/* MODULES */

mod manager;

/* DEFINITIONS */

pub struct Resource {
    manager: Weak<ResourceManager>,
    schema: Schema,
    data: Content,
    id: ResourceID,
}

struct Content {
    indices: HashMap<BitVec<u8, Msb0>, usize>,
    storage: BitVec<u8, Msb0>,
    current: usize,
    size: u64,
}

/* IMPLEMENTATION */

impl Resource {
    pub(in crate::database::volatile) fn new(
        manager: Arc<ResourceManager>,
        schema: Schema,
        id: ResourceID,
    ) -> Self {
        Self {
            manager: Arc::downgrade(&manager),
            schema,
            data: Content {
                indices: HashMap::new(),
                storage: BitVec::new(),
                current: 0,
                size: 0,
            },
            id,
        }
    }

    pub(in crate::database::volatile) fn id(&self) -> ResourceID {
        self.id
    }

    /* UTILITIES */

    fn data_slice(
        &self,
        from: usize,
        to: usize,
    ) -> Option<&BitSlice<u8, Msb0>> {
        if to > self.data.storage.len() {
            None
        } else {
            Some(&self.data.storage[from..to])
        }
    }

    fn index_removed(&self, index: usize) -> bool {
        *self
            .data
            .storage
            .get(index + self.schema.size())
            .expect("Out-of-bounds removal check on storage array.")
    }

    fn mark_removed(&mut self, index: usize) {
        self.data
            .storage
            .set(index + self.schema.size(), true);
    }
}

impl Map for Resource {
    fn insert<R: Record>(&mut self, key: &Key, record: &R) -> Result<()> {
        let key_bits = BitVec::from(key);
        self.data
            .indices
            .insert(key_bits, self.data.current);

        self.data
            .storage
            .extend(record.raw());

        self.data.storage.push(false);
        self.data.current += self.schema.size() + 1;
        self.data.size += 1;
        Ok(())
    }

    fn get(&self, key: &Key) -> Option<&Value> {
        let key_bits = BitVec::from(key);
        if let Some(&location) = self.data.indices.get(&key_bits) {
            if self.index_removed(location) {
                return None;
            }
            self.data_slice(location, location + self.schema.size())
        } else {
            None
        }
    }

    fn remove(&mut self, key: &Key) {
        let key_bits = BitVec::from(key);
        if let Some(&location) = self.data.indices.get(&key_bits) {
            self.mark_removed(location);
            self.data.size -= 1;
        }
    }
}

impl Relation for Resource {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn count(&self) -> u64 {
        self.data.size
    }

    fn bytes(&self) -> u64 {
        (self.data.storage.len() as u64) / 8
    }

    fn id(&self) -> SequenceKey {
        self.id()
    }
}
