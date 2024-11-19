//! # Sled Database Engine Integration
//!
//! It's all downhill from here.

use anyhow::anyhow;
use anyhow::Result;

use std::path::Path;

use crate::database::ByteMap;
use crate::database::Persistent;
use crate::database::ProtoRelational;
use crate::database::Relation;
use crate::database::Schema;

/* CONSTANTS */

/// A common name for all instances a [`sled`] database directory.
pub const DIRECTORY_NAME: &str = "sled_db";

/* DEFINITIONS */

/// Wrapper for [`sled::Db`].
pub struct SledDatabase {
    db: sled::Db,
}

/// Wrapper for [`sled::Tree`].
pub struct SledNamespace {
    tree: sled::Tree,
    schema: Schema,
}

/* NAMESPACE IMPLEMENTATIONS */

impl ByteMap for SledNamespace {
    fn insert<K, V>(&mut self, key: K, record: V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        self.tree
            .insert(key, record.as_ref())?;
        Ok(())
    }

    fn get<K>(&self, key: K) -> Result<Option<Vec<u8>>>
    where
        K: AsRef<[u8]>,
    {
        Ok(self
            .tree
            .get(key)?
            .map(|v| v.to_vec()))
    }

    fn remove<K>(&mut self, key: K) -> Result<Option<Vec<u8>>>
    where
        K: AsRef<[u8]>,
    {
        Ok(self
            .tree
            .remove(key)?
            .map(|v| v.to_vec()))
    }
}

impl Relation for SledNamespace {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn count(&self) -> usize {
        self.tree.len()
    }
}

/* DATABASE IMPLEMENTATIONS */

impl ProtoRelational for SledDatabase {
    type Namespace = SledNamespace;
    fn namespace(&self, schema: Schema, name: &str) -> Result<Self::Namespace> {
        Ok(SledNamespace {
            tree: self.db.open_tree(name)?,
            schema,
        })
    }

    fn drop(&self, name: &str) -> Result<bool> {
        self.db
            .drop_tree(name)
            .map_err(|e| anyhow!("Failed to drop namespace '{}': {}", name, e))
    }
}

impl Persistent for SledDatabase {
    fn new(path: &Path) -> Result<Self>
    where
        Self: Sized,
    {
        let db = sled::open(path)?;
        Ok(SledDatabase { db })
    }

    fn flush(&self) -> Result<usize> {
        self.db
            .flush()
            .map_err(|e| anyhow!("Failed to flush database: {}", e))
    }
}
