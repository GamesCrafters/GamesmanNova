//! # Volatile Database Resource Manager
//!
//! TODO

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::{Arc, RwLock};

use crate::database::volatile::resource::Resource;
use crate::database::volatile::transaction::ResourceHandles;
use crate::database::volatile::transaction::Transaction;
use crate::database::volatile::transaction::TransactionManager;
use crate::database::volatile::ResourceID;
use crate::database::volatile::Sequencer;
use crate::database::volatile::TransactionID;

/* DEFINITIONS */

pub struct ResourceManager {
    pub sequencer: Arc<Sequencer>,
    access: Mutex<AccessControl>,
    signal: Condvar,
}

#[derive(Default)]
pub struct Request {
    pub write: Vec<ResourceID>,
    pub read: Vec<ResourceID>,
}

#[derive(Default)]
pub struct AccessControl {
    pool: HashMap<ResourceID, Arc<RwLock<Resource>>>,
    owners: HashMap<ResourceID, TransactionID>,
    reading: HashMap<ResourceID, u32>,
    writing: HashSet<ResourceID>,
    dropped: HashSet<ResourceID>,
}

/* IMPLEMENTATION */

impl Request {
    pub fn empty() -> Self {
        Self::default()
    }
}

impl AccessControl {
    fn conflict(&self, request: &Request) -> bool {
        request
            .read
            .iter()
            .chain(&request.write)
            .any(|id| self.writing.contains(id))
    }
}

impl ResourceManager {
    pub fn new(sequencer: Arc<Sequencer>) -> Arc<Self> {
        let access = Mutex::new(AccessControl::default());
        let signal = Condvar::new();
        let manager = Self {
            sequencer,
            access,
            signal,
        };

        Arc::new(manager)
    }

    pub fn add_resource(&self, resource: Resource) -> Result<()> {
        let mut resources = self
            .access
            .lock()
            .map_err(|_| anyhow!("Resource access lock poisoned."))?;

        let id = resource.id();
        let lock = RwLock::new(resource);
        resources
            .pool
            .insert(id, Arc::new(lock));

        Ok(())
    }

    pub fn drop_resource(
        &self,
        rid: ResourceID,
        tid: TransactionID,
    ) -> Result<()> {
        let mut resources = self
            .access
            .lock()
            .map_err(|_| anyhow!("Resource access lock poisoned."))?;

        if let Some(&owner) = resources.owners.get(&rid) {
            if owner == tid && !resources.dropped.contains(&rid) {
                resources.dropped.insert(rid);
                resources.pool.remove(&rid);
            }
        }

        Err(anyhow!(
            "Attempted to drop resource without ownership."
        ))
    }

    pub fn initialize_transaction(
        &self,
        request: Request,
        manager: Arc<TransactionManager>,
    ) -> Result<Arc<Transaction>> {
        let mut resources = self.lock()?;
        loop {
            if request
                .write
                .iter()
                .chain(&request.read)
                .any(|id| !resources.pool.contains_key(id))
            {
                bail!("Attempted to acquire non-existent resource.");
            }

            if !resources.conflict(&request) {
                let id = self.sequencer.next_transaction()?;
                self.acquire_resources(&request, id, &mut resources);
                let handles = self.generate_handles(&request, &mut resources);
                let transaction =
                    Transaction::new(manager, handles, request, id);

                return Ok(transaction);
            }

            resources = self
                .signal
                .wait(resources)
                .map_err(|_| anyhow!("Resource access lock poisoned."))?;
        }
    }

    /* UTILITIES */

    pub fn lock(&self) -> Result<MutexGuard<AccessControl>> {
        self.access
            .lock()
            .map_err(|_| anyhow!("Resource access lock poisoned."))
    }

    pub fn signal_waiters(&self) {
        self.signal.notify_all()
    }

    pub fn acquire_resources(
        &self,
        request: &Request,
        transaction: TransactionID,
        resources: &mut MutexGuard<AccessControl>,
    ) {
        request
            .write
            .iter()
            .for_each(|id| {
                resources.writing.insert(*id);
                resources
                    .owners
                    .insert(*id, transaction);
            });

        request.read.iter().for_each(|id| {
            let count = resources
                .reading
                .entry(*id)
                .or_insert(0);

            *count += 1;
            resources
                .owners
                .insert(*id, transaction);
        });
    }

    pub fn release_resources(
        &self,
        request: &Request,
        resources: &mut MutexGuard<AccessControl>,
    ) {
        request
            .write
            .iter()
            .for_each(|id| {
                resources.owners.remove(id);
                resources.writing.remove(id);
            });

        request.read.iter().for_each(|id| {
            resources.owners.remove(id);
            let count = resources
                .reading
                .entry(*id)
                .or_insert(1);

            *count -= 1;
        });
    }

    fn generate_handles(
        &self,
        request: &Request,
        resources: &mut MutexGuard<AccessControl>,
    ) -> ResourceHandles {
        let mut handles = ResourceHandles::default();
        for &id in request.write.iter() {
            let lock = resources
                .pool
                .get(&id)
                .expect("Attempted to fetch non-existent resource lock.");

            handles.add_write(id, lock.clone());
        }

        for &id in request.read.iter() {
            let lock = resources
                .pool
                .get(&id)
                .expect("Attempted to fetch non-existent resource lock.");

            handles.add_read(id, lock.clone());
        }

        handles
    }
}
