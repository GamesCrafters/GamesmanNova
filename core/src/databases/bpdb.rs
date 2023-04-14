//! # Bit Perfect Database Module
//!
//! This module contains the implementation for a bit-perfect database.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

use super::Database;
use crate::{State, Value};

pub struct BPDatabase;

impl Database for BPDatabase {
    fn new(id: String, read: bool, write: bool) -> Self {
        todo!()
    }

    fn insert(&mut self, state: State, value: Option<Value>) {}

    fn get(&self, state: State) -> &Option<Value> {
        todo!()
    }

    fn delete(&mut self, state: State) {}
}
