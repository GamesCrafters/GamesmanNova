//! # Volatile Database Resource
//!
//! TODO

use std::sync::{Arc, Weak};

use crate::database::volatile::ResourceID;

/* RE-EXPORTS */

pub use manager::Request;
pub use manager::ResourceManager;

/* MODULES */

mod manager;

/* DEFINITIONS */

pub struct Resource {
    manager: Weak<ResourceManager>,
    id: ResourceID,
}

/* IMPLEMENTATION */

impl Resource {
    pub(in crate::database::volatile) fn new(
        manager: Arc<ResourceManager>,
        id: ResourceID,
    ) -> Self {
        let manager = Arc::downgrade(&manager);
        Self { manager, id }
    }

    pub fn id(&self) -> ResourceID {
        self.id
    }
}
