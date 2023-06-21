use std::fs::{File, OpenOptions, remove_file};
use std::io::Write;
use std::os::unix::prelude::FileExt;
use std::path::Path;
use std::sync::{mpsc, Mutex, mpsc::{Receiver, Sender}};
use std::{thread, thread::JoinHandle};

use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use super::DatabaseMode;
use super::page::Page;

/// The largest amount of log updates that a single log file can contain. Each
/// update contains 9 bytes -- 8 bytes for an unsigned 64-bit integer index,
/// and one byte for that index's corresponding value in the database.
pub const MAX_LOG_SIZE: u64 = 100_000;

/// Instructions that can be sent to logger thread representing different
/// actions, such as adding an update to the append-only log, or terminating
/// the write session by persisting whatever is in the current log (as opposed
/// to waiting to reach the maximum log size).
#[derive(Clone)]
enum LoggerMessage {
    Update {
        index: usize,
        value: u8
    },
    Terminate,
}

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
pub struct Engine<'a> {
    /// Sender for `mpsc` logger thread channel.
    logger_sender: Sender<LoggerMessage>,
    /// Logger thread handle.
    logger_handle: Option<JoinHandle<()>>,
    /// Unique database name (unenforced, bad things will happen if repeated).
    identifier: &'a String,
    /// In-memory cache of database contents.
    memory: Vec<Mutex<Page>>,
    /// Specifier for persistence options.
    mode: &'a DatabaseMode<'a>,
    /// Directory path where logs and databases will be created.
    path: &'a Path,
}

impl Engine<'_> {
    /// Returns a new database engine implementation.
    pub fn new<'a>(identifier: &'a String, mode: &'a DatabaseMode<'a>, path: &'a Path) -> Engine<'a> {
        let mut memory = Vec::new();
        let (logger_sender, logger_receiver) = mpsc::channel();
        let mut logger_handle = if let DatabaseMode::Persistent(_) = mode {
            Some(spawn_logger(logger_receiver, path, identifier))
        } else {
            None
        };
        Engine {
            logger_sender,
            logger_handle,
            identifier,
            memory,
            mode,
            path,
        }
    }

    /// Signals the database to update the record at `index` with `value`.
    /// Different `page`s can be accessed by different threads with a safety
    /// guarantee, as each one is under a `Mutex` lock. Attempting to access
    /// the same page from different threads results in blocking behavior,
    /// and writing to the same index in different pages from different threads
    /// results in non-deterministic persistence if enabled.
    pub fn update(&mut self, index: usize, value: u8, page: usize) {
        if let Some(page) = self.memory.get(page) {
            if let Ok(mut data) = page.lock() {
                data.insert(index, value);
            } else {
                panic!("A database page in memory was poisoned by a thread panic.");
            }
        } else {
            let mut new_page = Page::new();
            new_page.insert(index, value);
            self.memory.insert(page, Mutex::new(new_page));
        }
        if let DatabaseMode::Persistent(path) = self.mode {
            self.logger_sender.send(
                LoggerMessage::Update {
                    index,
                    value
                }
            );
        }
    }

    /// Returns an optional reference to the in-memory byte at `index` in the
    /// indicated `page`. Only guarantees to return `None` if no database page
    /// with the passed identifier has been created.
    pub fn get(&self, index: usize, page: usize) -> Option<&u8> {
        if let Some(page) = self.memory.get(page) {
            if let Ok(mut data) = page.lock() {
                data.get(index)
            } else {
                panic!("A database page in memory was poisoned by a thread panic.");
            }
        } else {
            None
        }
    }

    /// Returns an optional mutable reference to the in-memory byte at `index`
    /// in the indicated `page`. Only guarantees to return `None` if no
    /// database page with the passed identifier has been created.
    pub fn get_mut(&mut self, index: usize, page: usize) -> Option<&mut u8> {
        if let Some(page) = self.memory.get(page) {
            if let Ok(mut data) = page.lock() {
                data.get_mut(index)
            } else {
                panic!("A database page in memory was poisoned by a thread panic.");
            }
        } else {
            None
        }
    }
}

impl Drop for Engine<'_> {
    fn drop(&mut self) {
        if let Some(handle) = self.logger_handle {
            self.logger_sender.send(LoggerMessage::Terminate);
            handle.join();
        }
    }
}

fn spawn_logger<'a>(receiver: Receiver<LoggerMessage>,
                    path: &'a Path,
                    identifier: &'a String) -> JoinHandle<()>
{
    let mut current_log = generate_log_file(path, identifier, 0);
    let mut database = generate_database_file(path, identifier);
    let mut logger = Logger {
        log_update_count: 0,
        log_file_count: 0,
        current_log,
        identifier,
        database,
        path
    };

    thread::spawn(move || match receiver.recv() {
        Ok(message) => {
            match message {
                LoggerMessage::Terminate => {
                    // TODO: Dispatch asynchronous from main and logger threads
                    logger.write_log_to_database(logger.current_log);
                    let current_log_path = get_log_path(
                        logger.path,
                        logger.identifier,
                        logger.log_file_count
                    );
                    // TODO: Wait for DB write to finish before deleting
                    remove_file(current_log_path);
                },
                LoggerMessage::Update { index, value } => {
                    if logger.log_update_count == MAX_LOG_SIZE {
                        let old_log = logger.indiana_jones_log();
                        // TODO: Dispatch asynchronous from main and logger threads
                        logger.write_log_to_database(old_log);
                    }
                    logger.write_update(index, value);
                }
            }
        },
        Err(error) => {
            eprintln!("Logger failed to receive a message: {:?}", error);
            todo!("Handle failures when logger receiver fails.");
        }
    })
}

/* LOGGER THREAD CONSTRUCTS */

struct Logger<'a> {
    log_update_count: u64,
    log_file_count: u64,
    current_log: File,
    identifier: &'a String,
    database: File,
    path: &'a Path,
}

impl Logger<'_> {
    fn indiana_jones_log(&mut self) -> File {
        self.log_file_count += 1;
        self.log_update_count = 0;
        let mut new_log = generate_log_file(
            self.path,
            self.identifier,
            self.log_file_count
        );
        std::mem::replace(&mut self.current_log, new_log)
    }

    fn write_log_to_database(&mut self, from_log: File)
    {
        (0..self.log_update_count)
            .into_par_iter()
            .for_each(|i| {
                let record_byte_offset = i * 9;
                let mut record_buffer = [0; 9];
                from_log.read_at(&mut record_buffer, record_byte_offset);
                let index_bytes: [u8; 8] = record_buffer[0..8]
                    .try_into()
                    .expect("Failed to read first 8 bytes of a record.");
                let index = bytemuck::cast::<[u8; 8], u64>(index_bytes);
                self.database.write_at(&[record_buffer[8]], index);
        });
    }

    fn write_update(&mut self, index: usize, value: u8) {
        let mut data: [u8; 9] = [0; 9];
        let index = bytemuck::cast::<usize, [u8; 8]>(index);
        data[..8].copy_from_slice(&index);
        data[8] = value;
        if let Err(e) = self.current_log.write_all(&data) {
            eprintln!("Error writing update to log buffer:  {:?}", e);
            if let Err(e) = self.current_log.flush() {
                eprintln!("Error flushing log contents to disk: {:?}", e);
            }
            todo!("Handle failures when writing to logfile.");
        }
        self.log_update_count += 1;
    }
}

/* LOGGER HELPER FUNCTIONS */

fn generate_log_file(path: &Path,
                     identifier: &String,
                     log_file_count: u64) -> File
{
    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(
            get_log_path(path, identifier, log_file_count)
        )
        .expect("Failed to create new log file.")
}

fn generate_database_file(path: &Path, identifier: &String) -> File {
    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(
            get_database_path(path, identifier)
        )
        .expect("Failed to create new database file.")
}

fn get_log_path<'a>(path: &'a Path,
                    identifier: &'a String,
                    log_file_count: u64) -> &'a Path
{
    let mut new_name = identifier.clone();
    new_name.push_str(".log");
    new_name.push_str(&log_file_count.to_string());
    let new_filepath = path.join(new_name);
    Path::new(&new_filepath)
}

fn get_database_path<'a>(path: &'a Path, identifier: &'a String) -> &'a Path {
    let mut new_name = identifier.clone();
    new_name.push_str(".db");
    let new_filepath = path.join(new_name);
    Path::new(&new_filepath)
}

/* TESTS */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_cache_read_and_write() {
        let mode = &DatabaseMode::Virtual;
        let path = Path::new(&"".to_string());
        let mut e = Engine::new(&"".to_string(), mode, path);

        e.update(0, 23, 1);
        assert_eq!(Some(23), e.get(0, 1).copied());
        assert_eq!(None, e.get(0, 0).copied());

        e.update(5, 5, 0);
        assert_eq!(Some(0), e.get_mut(0, 0).copied());
        assert_eq!(None, e.get_mut(30, 3).copied());
    }
}
