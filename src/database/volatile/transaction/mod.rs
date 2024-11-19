//! # Volatile Database Transaction
//!
//! TODO

use anyhow::anyhow;
use anyhow::Result;

use std::collections::HashMap;
use std::sync::RwLockReadGuard;
use std::sync::{Arc, RwLock, RwLockWriteGuard, Weak};

use crate::database::volatile::resource::Resource;
use crate::database::volatile::Request;
use crate::database::volatile::{ResourceID, TransactionID};
use crate::database::Schema;

/* RE-EXPORTS */

pub use manager::TransactionManager;

/* MODULES */

mod manager;

/* DEFINITIONS */

pub struct Transaction {
    manager: Weak<TransactionManager>,
    handles: ResourceHandles,
    request: Request,
    id: TransactionID,
}

#[derive(Default)]
pub struct ResourceHandles {
    write: HashMap<String, Arc<RwLock<Resource>>>,
    read: HashMap<String, Arc<RwLock<Resource>>>,
}

pub struct WorkingSet<'a> {
    write: HashMap<String, RwLockWriteGuard<'a, Resource>>,
    read: HashMap<String, RwLockReadGuard<'a, Resource>>,
}

/* IMPLEMENTATION */

impl WorkingSet<'_> {
    pub fn get_reading(&mut self, name: &str) -> RwLockReadGuard<Resource> {
        self.read.remove(name).unwrap()
    }

    pub fn get_writing(&mut self, name: &str) -> RwLockWriteGuard<Resource> {
        self.write.remove(name).unwrap()
    }
}

impl ResourceHandles {
    pub fn add_read(&mut self, name: String, lock: Arc<RwLock<Resource>>) {
        self.read.insert(name, lock);
    }

    pub fn add_write(&mut self, name: String, lock: Arc<RwLock<Resource>>) {
        self.write.insert(name, lock);
    }

    fn read(&self, name: &str) -> Result<RwLockReadGuard<Resource>> {
        if let Some(ref resource) = self.read.get(name) {
            Ok(resource
                .read()
                .map_err(|_| anyhow!("Read on poisoned resource lock."))?)
        } else {
            Err(anyhow!(
                "Attempted read on unacquired resource {}.",
                name
            ))
        }
    }

    fn write(&self, name: &str) -> Result<RwLockWriteGuard<Resource>> {
        if let Some(ref resource) = self.write.get(name) {
            Ok(resource
                .write()
                .map_err(|_| anyhow!("Write on poisoned resource lock."))?)
        } else {
            Err(anyhow!(
                "Attempted write on unacquired resource {}.",
                name
            ))
        }
    }

    pub fn lock_all(&self) -> Result<WorkingSet> {
        let read = self
            .read
            .iter()
            .map(|(name, l)| {
                l.read()
                    .map(|l| (name.clone(), l))
                    .map_err(|_| anyhow!("Read on poisoned resource lock."))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let write = self
            .write
            .iter()
            .map(|(name, l)| {
                l.write()
                    .map(|l| (name.clone(), l))
                    .map_err(|_| anyhow!("Write on poisoned resource lock."))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        Ok(WorkingSet { write, read })
    }
}

impl Transaction {
    pub fn read(&self, name: &str) -> Result<RwLockReadGuard<Resource>> {
        self.handles.read(name)
    }

    pub fn write(&self, name: &str) -> Result<RwLockWriteGuard<Resource>> {
        self.handles.write(name)
    }

    pub fn resources(&self) -> Result<WorkingSet> {
        self.handles.lock_all()
    }

    pub fn id(&self) -> TransactionID {
        self.id
    }

    /* PROTECTED */

    pub(in crate::database::volatile) fn new(
        manager: Arc<TransactionManager>,
        handles: ResourceHandles,
        request: Request,
        id: TransactionID,
    ) -> Arc<Self> {
        let manager = Arc::downgrade(&manager);
        let transaction = Self {
            manager,
            handles,
            request,
            id,
        };

        Arc::new(transaction)
    }

    pub(in crate::database::volatile) fn create_resource(
        &self,
        schema: Schema,
    ) -> Result<ResourceID> {
        if let Some(manager) = self.manager.upgrade() {
            let id = manager.sequencer.next_resource()?;
            let resource_manager = manager.resource_manager.clone();
            let resource = Resource::new(resource_manager.clone(), schema, id);
            resource_manager.add_resource(resource)?;
            Ok(id)
        } else {
            Err(anyhow!("Transaction manager was dropped."))
        }
    }

    pub(in crate::database::volatile) fn drop_resource(
        &self,
        id: ResourceID,
    ) -> Result<()> {
        if let Some(manager) = self.manager.upgrade() {
            manager
                .resource_manager
                .drop_resource(id, self.id())?;

            Ok(())
        } else {
            Err(anyhow!("Transaction manager was dropped."))
        }
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if let Some(manager) = self.manager.upgrade() {
            let resource_manager = &manager.resource_manager;
            let mut resources = resource_manager.lock().unwrap();
            resource_manager.release_resources(&self.request, &mut resources);
            resource_manager.signal_waiters();
        }
    }
}
