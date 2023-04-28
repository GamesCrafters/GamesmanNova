use crate::core::databases::DatabaseMode;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::{path::Path, sync::Mutex, thread};

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

enum LoggerMessage {
    Update(Record, Arc<Mutex<File>>),
    Checkpoint,
}

/* DATABASE IMPLEMENTATION */

impl ByteDB<'_> {
    fn initialize(
        identifier: &String,
        directory: &Path,
        mode: DatabaseMode,
        log_size: u128,
        log_num: u128,
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
            //
            // TODO: Send update to logging worker thread
            //
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

/// Wrapper for a worker thread whose only purpose is to persist the operations
/// made on a `ByteDB` to disk atomically and durably using write-ahead logging
/// and file checkpoints.
struct ByteStreamSerializer<'a> {
    /// Name of the database. Used to name log and database files.
    identifier: &'a String,
    /// Target directory to store log and database files.
    directory: &'a Path,
    /// Write-ahead log currently being written to.
    log_file: Arc<Mutex<File>>,
    /// Number of new log files that have been produced in total.
    log_count: u64,
    /// Handle of the worker thread handling logging.
    handle: JoinHandle<()>,
    /// Sender for the logging thread's communication channel.
    sender: Sender<LoggerMessage>,
}

impl ByteStreamSerializer<'_> {
    fn new(identifier: &String, directory: &Path) -> Self {
        let log_count = 0;
        let (sender, receiver) = mpsc::channel();
        let log_file = Self::generate_log(directory, identifier, log_count);
        let log_file = Arc::new(Mutex::new(log_file));
        let handle = Self::spawn_listener(receiver);
        ByteStreamSerializer {
            identifier,
            directory,
            log_file,
            log_count,
            handle,
            sender,
        }
    }

    fn order_update(&self, record: Record) {
        let log_reference = Arc::clone(&self.log_file);
        let message = LoggerMessage::Update(record, log_reference);
        if let Err(e) = self.sender.send(message) {
            eprintln!("Error sending update order to logger: {:?}", e);
            todo!("Handle logger sender errors gracefully.");
        }
    }

    fn order_checkpoint(&self) {
        let message = LoggerMessage::Checkpoint;
        if let Err(e) = self.sender.send(message) {
            eprintln!("Error sending checkpoint order to logger: {:?}", e);
            todo!("Handle logger sender errors gracefully.");
        }
    }

    /* Associated methods... */

    fn spawn_listener(receiver: Receiver<LoggerMessage>) -> JoinHandle<()> {
        thread::spawn(move || match receiver.recv() {
            Ok(message) => match message {
                LoggerMessage::Update(record, file_reference) => {
                    let log = unlock_file(file_reference);
                    Self::perform_log_update(record, log);
                }
                LoggerMessage::Checkpoint => {}
            },
            Err(e) => {
                eprintln!("Logger failed to receive a message: {:?}", e);
                todo!("Handle failures when logger receiver fails.");
            }
        })
    }

    fn perform_checkpoint() {
        todo!()
    }

    fn perform_log_update(record: Record, log: &mut File) {
        let mut data: [u8; 9] = [0; 9];
        let index: [u8; 8] = bytemuck::cast(record.offset);
        data[..8].copy_from_slice(&index);
        data[8] = record.value;
        if let Err(e) = log.write_all(&data) {
            eprintln!("Error writing update to log buffer:  {:?}", e);
            if let Err(e) = log.flush() {
                eprintln!("Error flushing log contents to disk: {:?}", e);
            }
            todo!("Handle failures when writing to logfile.");
        }
    }

    fn generate_log(directory: &Path, identifier: &String, log_count: u64) -> File {
        let mut new_name = identifier.clone();
        new_name.push_str(".log");
        new_name.push_str(&log_count.to_string());
        let new_filepath = directory.join(new_name);
        OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(new_filepath)
            .expect("Failed to create new log file.")
    }
}

/* DATABASE PAGE */

/// Hybrid data structure consisting of a `BTreeMap` top layer with byte
/// arrays of static size as values and integers as keys. Made as alternative
/// to a really long vector of bytes (where inserting a single element to a
/// very high index, for example, wastes a lot of capacity).
///
/// TODO: Implement a cache to avoid searching the B-Tree when doing sequential
///       accesses (i.e. cache a reference to the last accessed block). This
///       will require knowledge of `Arc`.
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

/* HELPER FUNCTIONS */

fn unlock_file<'a>(reference: Arc<Mutex<File>>) -> &'a mut File {
    match reference.lock() {
        Ok(log) => &mut (*log),
        Err(e) => {
            eprintln!("Error locking log file: {:?}", e);
            todo!("Handle log file locking errors gracefully.");
        }
    }
}
