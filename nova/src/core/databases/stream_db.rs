use crate::core::databases::DatabaseMode;
use rayon::{ThreadPool, ThreadPoolBuilder};
use crossbeam_channel::unbounded;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::{path::Path, sync::Mutex, thread};
use std::thread::JoinHandle;

/* DEFINITIONS */

/// Sets the block size within database pages.
pub const BLOCK_MASK: usize = 0xffff;

/// Low-abstraction database implementation. As a high level explanation,
/// it allows callers to interface with seemingly contiguous sequences of
/// bytes in memory, and persists updates in a file atomically and durably.
/// This works as follows:
///
/// - Write a byte to a specific page, each having an address space of `usize`.
/// - Each page has a mutual exclusion lock, ensuring thread-safety.
///
/// While each page can be indexed as if it were an array of size `usize`,
/// the implementation ensures that memory is not wasted in unused data
/// structure capacity by subdividing each page into blocks of static size,
/// which are stored in a `BTreeMap`.
///
/// For increased performance, pages cache the last accessed block on the stack
/// such that consecutive reads and writes to the same block do not incur the
/// intrinsic cost of a B-Tree search. For stability reasons, only blocks which
/// are are accessed through methods that take a mutable borrow of the page
/// (namely `get_mut` and `insert`) get cached.
pub struct ByteDB<'a> {
    /// The desired database file name.
    identifier: &'a String,
    /// The desired directory of the database file.
    directory: &'a Path,
    /// File where the database contents can be found.
    file: File,
    /// Decides persistence and read permissions.
    mode: DatabaseMode,
    /// Virtual memory allocation for faster caching.
    memory: Vec<Mutex<Page>>,
    /// Maximum number of entries per log file.
    log_size: u128,
    /// The amount of entries currently logged.
    log_num: u128,
    /// Pool of slave threads.
    threads: ThreadPool,
}

struct Record {
    offset: usize,
    value: u8,
}

enum LogMessage {
    Update(Record),
    Checkpoint,
}

enum WriteMessage {
    Checkpoint,
}

/* IMPLEMENTATION */

impl ByteDB<'_> {
    fn initialize(
        identifier: &String,
        directory: &Path,
        mode: DatabaseMode,
        log_size: u128,
        log_num: u128
    ) -> Self {
        // Initialize a thread pool for logging and writing.
        let mut threads: ThreadPool;
        match ThreadPoolBuilder::new().build() {
            Ok(pool) => {
                threads = pool;
            }
            Err(e) => {
                eprintln!("Error creating database thread pool: {:?}", e);
                todo!("Handle errors from thread builder gracefully.");
            }
        }

        // Create final database file.
        let file = Self::create_database_file(directory, identifier);

        // Initialize cache data structure in virtual memory.
        let memory: Vec<Mutex<Page>> = Vec::new();

        ByteDB {
            identifier,
            directory,
            file,
            mode,
            memory,
            log_size,
            log_num,
            threads,
        }
    }

    fn update(&mut self, record: Record, page: usize) {
        let target_lock = self.memory.get(page).unwrap_or(&Mutex::new(Page::new()));
        if let Ok(mut page) = target_lock.lock() {

            // TODO: Send update to logging worker thread

            page.insert(record.offset, record.value);
        } else {
            panic!("A database page in memory was poisoned by a thread.");
        }
    }

    fn create_database_file(directory: &Path, identifier: &String) -> File {
        let full_path = directory.join(Path::new(identifier));
        let database = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(full_path);
        match database {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error creating/opening database file: {:?}", e);
                todo!("Handle errors from database creation or opening.");
            }
        }
    }
}

/* SERIALIZER WORKER */



/* PAGE DATA STRUCTURE  */

/// A sparse, addressable sequence of bytes that initializes to all-zeros.
struct Page {
    data: BTreeMap<usize, [u8; BLOCK_MASK + 1]>,
}

impl Page {
    fn new() -> Self {
        Page {
            data: BTreeMap::new(),
        }
    }

    fn get(&self, index: usize) -> Option<&u8> {
        let upper = index & !BLOCK_MASK;
        let lower = index & BLOCK_MASK;
        if let Some(block) = self.data.get(&upper) {
            Some(&block[lower])
        } else {
            None
        }
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut u8> {
        let upper = index & !BLOCK_MASK;
        let lower = index & BLOCK_MASK;
        if let Some(block) = self.data.get_mut(&upper) {
            Some(&mut block[lower])
        } else {
            None
        }
    }

    fn insert(&mut self, index: usize, value: u8) {
        let upper = index & !BLOCK_MASK;
        let lower = index & BLOCK_MASK;
        match self.data.get_mut(&upper) {
            Some(mut block) => {
                block[lower] = value;
            }
            None => {
                let mut new_block = [0; BLOCK_MASK + 1];
                new_block[lower] = value;
                self.data.insert(upper, new_block);
            }
        }
    }
}
