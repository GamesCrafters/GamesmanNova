//! # Volatile Database Transaction
//!
//! TODO

use anyhow::anyhow;
use anyhow::Result;

use std::sync::RwLockReadGuard;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockWriteGuard, Weak},
};

use crate::database::volatile::resource::Resource;
use crate::database::volatile::Request;
use crate::database::volatile::{ResourceID, TransactionID};

/* RE-EXPORTS */

pub use manager::TransactionManager;

/* MODULES */

mod manager;

/* DEFINITIONS */

#[derive(Default)]
pub struct ResourceHandles {
    write: HashMap<ResourceID, WriteLock>,
    read: HashMap<ResourceID, ReadLock>,
}

pub struct Transaction {
    manager: Weak<TransactionManager>,
    handles: ResourceHandles,
    request: Request,
    id: TransactionID,
}

pub struct WriteLock(Arc<RwLock<Resource>>);
pub struct ReadLock(Arc<RwLock<Resource>>);

/* IMPLEMENTATION */

impl WriteLock {
    fn new(lock: Arc<RwLock<Resource>>) -> Self {
        Self(lock)
    }

    fn lock(&self) -> Result<RwLockWriteGuard<Resource>> {
        self.0
            .write()
            .map_err(|_| anyhow!("Resource lock poisoned."))
    }
}

impl ReadLock {
    fn new(lock: Arc<RwLock<Resource>>) -> Self {
        Self(lock)
    }

    fn lock(&self) -> Result<RwLockReadGuard<Resource>> {
        self.0
            .read()
            .map_err(|_| anyhow!("Resource lock poisoned."))
    }
}

impl ResourceHandles {
    pub fn add_reading(&mut self, id: ResourceID, lock: Arc<RwLock<Resource>>) {
        self.read
            .insert(id, ReadLock::new(lock));
    }

    pub fn add_writing(&mut self, id: ResourceID, lock: Arc<RwLock<Resource>>) {
        self.write
            .insert(id, WriteLock::new(lock));
    }

    fn get_reading(&self, resource: ResourceID) -> Result<&ReadLock> {
        if let Some(resource) = self.read.get(&resource) {
            Ok(resource)
        } else {
            Err(anyhow!(
                "Attempted read on unacquired resource {}.",
                resource
            ))
        }
    }

    fn get_writing(&self, resource: ResourceID) -> Result<&WriteLock> {
        if let Some(resource) = self.write.get(&resource) {
            Ok(resource)
        } else {
            Err(anyhow!(
                "Attempted write on unacquired resource {}.",
                resource
            ))
        }
    }
}

impl Transaction {
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

    pub fn create_resource(&self) -> Result<ResourceID> {
        if let Some(manager) = self.manager.upgrade() {
            let id = manager.sequencer.next_resource()?;
            let resource_manager = manager.resource_manager.clone();
            let resource = Resource::new(resource_manager.clone(), id);
            resource_manager.add_resource(resource)?;
            Ok(id)
        } else {
            Err(anyhow!("Transaction manager was dropped."))
        }
    }

    pub fn drop_resource(&self, id: ResourceID) -> Result<()> {
        if let Some(manager) = self.manager.upgrade() {
            manager
                .resource_manager
                .drop_resource(id, self.id())?;

            Ok(())
        } else {
            Err(anyhow!("Transaction manager was dropped."))
        }
    }

    pub fn reading<F, O>(&self, id: ResourceID, func: F) -> Result<O>
    where
        F: FnOnce(&Resource) -> Result<O>,
    {
        let resource = self.handles.get_reading(id)?;
        let guard = resource.lock()?;
        func(&guard)
    }

    pub fn writing<F, O>(&self, id: ResourceID, func: F) -> Result<O>
    where
        F: FnOnce(&mut Resource) -> Result<O>,
    {
        let resource = self.handles.get_writing(id)?;
        let mut guard = resource.lock()?;
        func(&mut guard)
    }

    pub fn id(&self) -> TransactionID {
        self.id
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
