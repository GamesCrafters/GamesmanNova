//! # Volatile Database
//!
//! This module provides a trivial database implementation backed by a volatile
//! in-memory data structure arrangement.

use anyhow::bail;
use anyhow::Result;
use bitvec::slice::BitSlice;

use std::marker::PhantomData;
use std::sync::Arc;

use crate::database::model::SequenceKey;
use crate::database::record::dir;
use crate::database::util::KeySequencer;
use crate::database::volatile::resource::Request;
use crate::database::volatile::resource::ResourceManager;
use crate::database::volatile::transaction::Transaction;
use crate::database::volatile::transaction::TransactionManager;
use crate::database::Map;
use crate::database::Schema;

/* RE-EXPORTS */

pub use resource::Resource;
pub use transaction::WorkingSet;

/* MODULES */

mod transaction;
mod resource;

/* DEFINITIONS */

type TransactionID = SequenceKey;
type ResourceID = SequenceKey;

pub struct Database {
    transaction_manager: Arc<TransactionManager>,
    resource_manager: Arc<ResourceManager>,
    directory: Option<ResourceID>,
    sequencer: Arc<Sequencer>,
}

pub struct TransactionBuilder<F, O>
where
    F: FnOnce(WorkingSet) -> Result<O>,
{
    transaction_manager: Arc<TransactionManager>,
    resource_manager: Arc<ResourceManager>,
    write_requests: Vec<ResourceID>,
    read_requests: Vec<ResourceID>,
    function: Option<F>,
    _out: PhantomData<O>,
}

#[derive(Default)]
struct Sequencer {
    transaction: KeySequencer,
    resource: KeySequencer,
}

/* IMPLEMENTATION */

impl Sequencer {
    pub fn next_transaction(&self) -> Result<TransactionID> {
        self.transaction.next()
    }

    pub fn next_resource(&self) -> Result<ResourceID> {
        self.transaction.next()
    }
}

impl Database {
    pub fn new() -> Result<Self> {
        let directory = None;
        let sequencer = Arc::new(Sequencer::default());
        let resource_manager = ResourceManager::new(sequencer.clone());
        let transaction_manager = TransactionManager::new(
            resource_manager.clone(),
            sequencer.clone(),
        );

        let mut db = Self {
            transaction_manager,
            resource_manager,
            directory,
            sequencer,
        };

        let directory = db
            .start_transaction(Request::empty())?
            .create_resource(dir::schema()?)?;

        db.directory = Some(directory);
        Ok(db)
    }

    pub fn build_transaction<F, O>(&self) -> TransactionBuilder<F, O>
    where
        F: FnOnce(WorkingSet) -> Result<O>,
    {
        TransactionBuilder {
            transaction_manager: self.transaction_manager.clone(),
            resource_manager: self.resource_manager.clone(),
            write_requests: Vec::new(),
            read_requests: Vec::new(),
            function: None,
            _out: PhantomData,
        }
    }

    pub fn start_transaction(
        &self,
        request: Request,
    ) -> Result<Arc<Transaction>> {
        let txn = self
            .resource_manager
            .initialize_transaction(
                request,
                self.transaction_manager.clone(),
            )?;

        self.transaction_manager
            .add_transaction(txn.clone())?;

        Ok(txn)
    }

    pub fn create_resource(&self, schema: Schema, name: &str) -> Result<()> {
        let directory = self
            .directory
            .expect("Database directory table found uninitialized.");

        let txn = self.start_transaction(Request {
            write: vec![self.directory.unwrap()],
            read: vec![],
        })?;

        let mut directory = txn.write(directory)?;
        let dir_key = &BitSlice::from_slice(name.as_bytes());
        if let Some(_) = directory.get(dir_key) {
            bail!("Resource '{name}' already exists.")
        }

        let id = txn.create_resource(schema.clone())?;
        let mut record = dir::RecordBuffer::try_from(schema)?;
        record.set_offset(id);

        directory.insert(dir_key, &record);
        Ok(id)
    }

    pub fn drop_resource(&self, name: &str) -> Result<()> {
        let directory = self
            .directory
            .expect("Database directory table found uninitialized.");

        let txn = self.start_transaction(Request {
            write: vec![self.directory.unwrap()],
            read: vec![],
        })?;

        let dir_key = &BitSlice::from_slice(name.as_bytes());
        let mut directory = txn.write(directory)?;
        if let Some(value) = directory.get(dir_key) {
            let record = dir::RecordBuffer::new(value)?;
            let id = record.get_offset();
            txn.drop_resource(id)?;
            directory.remove(dir_key);
        }

        Ok(())
    }
}

impl<F, O> TransactionBuilder<F, O>
where
    F: FnOnce(WorkingSet) -> Result<O>,
{
    pub fn writing(mut self, id: ResourceID) -> Self {
        self.read_requests.push(id);
        self
    }

    pub fn reading(mut self, id: ResourceID) -> Self {
        self.write_requests.push(id);
        self
    }

    pub fn action(mut self, function: F) -> Self {
        self.function = Some(function);
        self
    }

    pub fn execute(self) -> Result<O> {
        if self.read_requests.is_empty() && self.write_requests.is_empty() {
            bail!("No resource acquisition requests provided.")
        }

        let function = if let Some(func) = self.function {
            func
        } else {
            bail!("No actionable provided to transaction builder.")
        };

        let request = Request {
            write: self.write_requests,
            read: self.read_requests,
        };

        let txn = self
            .resource_manager
            .initialize_transaction(
                request,
                self.transaction_manager.clone(),
            )?;

        self.transaction_manager
            .add_transaction(txn.clone())?;

        let working_set = txn.resources()?;
        function(working_set)
    }
}
