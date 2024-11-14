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
    write: HashMap<ResourceID, Arc<RwLock<Resource>>>,
    read: HashMap<ResourceID, Arc<RwLock<Resource>>>,
}

pub struct WorkingSet<'a> {
    write: HashMap<ResourceID, RwLockWriteGuard<'a, Resource>>,
    read: HashMap<ResourceID, RwLockReadGuard<'a, Resource>>,
}

/* IMPLEMENTATION */

impl WorkingSet<'_> {
    pub fn get_reading(&mut self, id: ResourceID) -> RwLockReadGuard<Resource> {
        self.read.remove(&id).unwrap()
    }

    pub fn get_writing(
        &mut self,
        id: ResourceID,
    ) -> RwLockWriteGuard<Resource> {
        self.write.remove(&id).unwrap()
    }
}

impl ResourceHandles {
    pub fn add_read(&mut self, id: ResourceID, lock: Arc<RwLock<Resource>>) {
        self.read.insert(id, lock);
    }

    pub fn add_write(&mut self, id: ResourceID, lock: Arc<RwLock<Resource>>) {
        self.write.insert(id, lock);
    }

    fn read(&self, resource: ResourceID) -> Result<RwLockReadGuard<Resource>> {
        if let Some(ref resource) = self.read.get(&resource) {
            Ok(resource
                .read()
                .map_err(|_| anyhow!("Read on poisoned resource lock."))?)
        } else {
            Err(anyhow!(
                "Attempted read on unacquired resource {}.",
                resource
            ))
        }
    }

    fn write(
        &self,
        resource: ResourceID,
    ) -> Result<RwLockWriteGuard<Resource>> {
        if let Some(ref resource) = self.write.get(&resource) {
            Ok(resource
                .write()
                .map_err(|_| anyhow!("Write on poisoned resource lock."))?)
        } else {
            Err(anyhow!(
                "Attempted write on unacquired resource {}.",
                resource
            ))
        }
    }

    pub fn lock_all(&self) -> Result<WorkingSet> {
        let read = self
            .read
            .iter()
            .map(|(&id, l)| {
                l.read()
                    .map(|l| (id, l))
                    .map_err(|_| anyhow!("Read on poisoned resource lock."))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let write = self
            .write
            .iter()
            .map(|(&id, l)| {
                l.write()
                    .map(|l| (id, l))
                    .map_err(|_| anyhow!("Write on poisoned resource lock."))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        Ok(WorkingSet { write, read })
    }
}

impl Transaction {
    pub fn read(&self, id: ResourceID) -> Result<RwLockReadGuard<Resource>> {
        self.handles.read(id)
    }

    pub fn write(&self, id: ResourceID) -> Result<RwLockWriteGuard<Resource>> {
        self.handles.write(id)
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
