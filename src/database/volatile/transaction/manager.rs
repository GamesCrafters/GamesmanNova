//! # Volatile Database Transaction Manager
//!
//! TODO

use anyhow::anyhow;
use anyhow::Result;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::database::volatile::transaction::Transaction;
use crate::database::volatile::ResourceManager;
use crate::database::volatile::Sequencer;
use crate::database::volatile::TransactionID;

/* DEFINITIONS */

pub struct TransactionManager {
    transactions: RwLock<HashMap<TransactionID, Arc<Transaction>>>,
    pub resource_manager: Arc<ResourceManager>,
    pub sequencer: Arc<Sequencer>,
}

/* IMPLEMENTATION */

impl TransactionManager {
    pub fn new(
        resource_manager: Arc<ResourceManager>,
        sequencer: Arc<Sequencer>,
    ) -> Arc<Self> {
        let transactions = RwLock::new(HashMap::new());
        let manager = Self {
            resource_manager,
            transactions,
            sequencer,
        };

        Arc::new(manager)
    }

    pub fn add_transaction(&self, transaction: Arc<Transaction>) -> Result<()> {
        let mut txns = self
            .transactions
            .write()
            .map_err(|_| anyhow!("Transaction manager lock poisoned."))?;

        {
            txns.insert(transaction.id(), transaction);
            Ok(())
        }
    }
}
