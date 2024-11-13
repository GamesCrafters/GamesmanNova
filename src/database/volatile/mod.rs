//! # Volatile Database
//!
//! This module provides a trivial database implementation backed by a volatile
//! in-memory data structure arrangement.

use anyhow::bail;
use anyhow::Result;

use std::marker::PhantomData;
use std::sync::Arc;

use crate::database::model::SequenceKey;
use crate::database::util::KeySequencer;
use crate::database::volatile::resource::Request;
use crate::database::volatile::resource::ResourceManager;
use crate::database::volatile::transaction::Transaction;
use crate::database::volatile::transaction::TransactionManager;
use crate::database::KVStore;

/* RE-EXPORTS */

pub use resource::Resource;
pub use transaction::WorkingSet;

use super::Schema;

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

    pub fn next_resource(&self) -> Result<TransactionID> {
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
            .create_resource(Self::directory_schema())?;

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

    pub fn create_resource(&self, schema: Schema) -> Result<ResourceID> {
        let directory = self
            .directory
            .expect("Database directory table found uninitialized.");

        let txn = self.start_transaction(Request {
            write: vec![self.directory.unwrap()],
            read: vec![],
        })?;

        let id = txn.create_resource(schema)?;
        let mut directory = txn.write(directory)?;
        todo!()
        // directory.insert(id.into(), schema.into());
        // Ok(id)
    }

    pub fn drop_resource(&self, id: ResourceID) -> Result<()> {
        let directory = self
            .directory
            .expect("Database directory table found uninitialized.");

        let txn = self.start_transaction(Request {
            write: vec![self.directory.unwrap()],
            read: vec![],
        })?;

        txn.drop_resource(id)?;
        let mut directory = txn.write(directory)?;
        todo!()
        // directory.remove(id.into());
        // Ok(id)
    }

    /* PRIVATE */

    fn directory_schema() -> Schema {
        todo!()
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
