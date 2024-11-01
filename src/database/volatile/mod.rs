//! # Volatile Database
//!
//! This module provides a trivial database implementation backed by a volatile
//! in-memory data structure arrangement.

use anyhow::Result;

use std::sync::Arc;

use crate::database::util::KeySequencer;
use resource::ResourceManager;
use transaction::TransactionManager;

/* RE-EXPORTS */

pub use resource::Request;
pub use resource::Resource;
pub use transaction::Transaction;

/* MODULES */

mod transaction;
mod resource;

/* DEFINITIONS */

type SequenceKey = u64;
type TransactionID = SequenceKey;
type ResourceID = SequenceKey;

pub struct Database {
    transaction_manager: Arc<TransactionManager>,
    resource_manager: Arc<ResourceManager>,
    sequencer: Arc<Sequencer>,
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
    fn new() -> Self {
        let sequencer = Arc::new(Sequencer::default());
        let resource_manager = ResourceManager::new(sequencer.clone());
        let transaction_manager = TransactionManager::new(
            resource_manager.clone(),
            sequencer.clone(),
        );

        Self {
            transaction_manager,
            resource_manager,
            sequencer,
        }
    }

    fn create_transaction(&self, request: Request) -> Result<Arc<Transaction>> {
        let transaction = self
            .resource_manager
            .initialize_transaction(
                request,
                self.transaction_manager.clone(),
            )?;

        {
            self.transaction_manager
                .add_transaction(transaction.clone());
            Ok(transaction)
        }
    }
}
