//! # Bit Perfect Database Module
//!
//! This module contains the implementation for a bit-perfect database.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

type Offset = usize;

use super::DatabaseMode;
use crossbeam_channel::{Receiver, Sender};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, ErrorKind, Read, Write};
use std::path::Path;
use std::sync::{Mutex};
use std::{thread, thread::JoinHandle};

// 50 million byte updates per checkpoint
const MAX_LOG_SIZE: u128 = 50_000_000;

// Multiple of available features
const PAGE_BYTE_SIZE: usize = 64;

// Represents a key-value pair
#[derive(Clone)]
struct Record(Offset, u8);

pub struct StreamDB<'a> {
    db_mode: DatabaseMode,
    memory: Vec<Mutex<[u8; PAGE_BYTE_SIZE]>>,
    update_count: u128,
    logger: Option<Logger<'a>>,
}

impl StreamDB<'_> {
    fn initialize(name: String, mode: DatabaseMode, path: &Path) -> Self {
        match mode {
            DatabaseMode::ReadOnly | DatabaseMode::Virtual => StreamDB {
                db_mode: mode,
                memory: Vec::new(),
                update_count: 0,
                logger: None,
            },
            DatabaseMode::Default => StreamDB {
                db_mode: mode,
                memory: Vec::new(),
                update_count: 0,
                logger: Some(Logger::initialize(path, &name)),
            },
        }
    }

    fn update(&mut self, value: u8, offset: usize) {
        if let Some(logger) = &self.logger {
            if self.update_count % MAX_LOG_SIZE == 0 && self.update_count > 0 {
                if let Err(e) = logger.sender.send(LoggerMessage::DoCheckpoint) {
                    eprintln!("Error sending checkpoint message to logger thread: {:?}", e);
                    todo!("Handle checkpoint message failure for logger thread.");
                }
            }
            let update_message = LoggerMessage::Update(Record(offset, value));
            if let Err(e) = logger.sender.send(update_message) {
                eprintln!("Error sending record update to logger thread: {:?}", e);
                todo!("Handle update message failure for logger thread.");
            }
        }
        let page_index: usize = offset / PAGE_BYTE_SIZE;
        let page_offset: usize = offset % PAGE_BYTE_SIZE;
        if let Some(mutex) = self.memory.get(page_index) {
            let page = mutex.lock().expect("Memory page poisoned!");
            page[page_offset] = value;
        } else {
            let mut new_page = [0; PAGE_BYTE_SIZE];
            new_page[page_offset] = value;
            self.memory.insert(page_index, Mutex::new(new_page));
        }
        self.update_count += 1;
    }
}

impl Drop for StreamDB<'_> {
    fn drop(&mut self) {
        if let Some(logger) = self.logger.take() {
            if let Err(e) = logger.sender.send(LoggerMessage::DoCheckpoint) {
                eprintln!("Error sending DoCheckpoint message: {:?}", e);
            }
            if let Some(handle) = logger.handle {
                if let Err(e) = handle.join() {
                    eprintln!("Error joining logger thread: {:?}", e);
                }
            }
        }
    }
}

/* LOGGING */

#[derive(Clone)]
enum LoggerMessage {
    DoCheckpoint,
    Update(Record)
}

struct Logger<'a> {
    log_count: u128,
    log_file: File,
    writer: Writer<'a>,
    sender: Sender<LoggerMessage>,
    handle: Option<JoinHandle<()>>,
}

impl Logger<'_> {
    fn initialize(db_dir: &Path, db_name: &String) -> Self {
        let log_file = {
            let mut new_name = db_name.clone();
            new_name.push_str(".log1");
            let new_filepath = db_dir.join(new_name);
            match OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(new_filepath) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to create new log file: {:?}", e);
                    todo!("Handle logfile generation failure.");
                },
            }
        };
        let writer = Writer::new(db_name, db_dir);
        let (sender, receiver) = crossbeam_channel::unbounded();
        let mut logger = Logger {
            log_count: 1,
            log_file,
            writer,
            sender,
            handle: None,
        };
        logger.handle = Some(logger.listen(receiver));
        logger
    }

    fn listen(&mut self, receiver: Receiver<LoggerMessage>) -> JoinHandle<()> {
        thread::spawn(move || {
            for received in receiver {
                match received {
                    LoggerMessage::DoCheckpoint => self.checkpoint(),
                    LoggerMessage::Update(record) => self.update(record)
                };
            }
        })
    }

    fn update(&mut self, record: Record) {
        let mut data: [u8; 9] = [0; 9];
        let index: [u8; 8] = bytemuck::cast(record.0);
        data[..8].copy_from_slice(&index);
        data[8] = record.1;
        if let Err(e) = self.log_file.write_all(&data) {
            eprintln!("Error writing update to log buffer:  {:?}", e);
            if let Err(e) = self.log_file.flush() {
                eprintln!("Error flushing log contents to disk: {:?}", e);
            }
            todo!("Handle failures when writing to logfile.");
        }
    }

    fn checkpoint(&mut self) {
        let mut old_log: File = self.generate_log();
        std::mem::swap(&mut self.log_file, &mut old_log);
        if let Err(e) = self.writer.sender.send(WriterMessage::DoCheckpoint(old_log)) {
            eprintln!("Error sending checkpoint message to writer thread: {:?}", e);
            todo!("Handle failed checkpoint message sends to writer thread.");
        }
    }

    fn generate_log(&mut self) -> File {
        self.log_count += 1;
        let mut new_name = self.writer.db_name.clone();
        new_name.push_str(".log");
        new_name.push_str(&self.log_count.to_string());
        let new_filepath = self.writer.db_dir.join(new_name);
        match OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(new_filepath) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Failed to create new log file: {:?}", e);
                todo!("Handle logfile generation failure.");
            },
        }
    }
}

/* CHECKPOINTS */

enum WriterMessage {
    DoCheckpoint(File),
}

struct Writer<'a> {
    db_name: &'a String,
    db_dir: &'a Path,
    sender: Sender<WriterMessage>,
    handle: Option<JoinHandle<()>>,
}

impl Writer<'_> {
    fn new(db_name: &String, db_dir: &Path) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let mut writer = Writer {
            db_name,
            db_dir,
            sender,
            handle: None,
        };
        writer.handle = Some(writer.listen(receiver));
        writer
    }

    fn listen(&mut self, receiver: Receiver<WriterMessage>) -> JoinHandle<()> {
        thread::spawn(move || {
            for received in receiver {
                match received {
                    WriterMessage::DoCheckpoint(old) => self.checkpoint(old)
                }
            }
        })
    }

    fn checkpoint(&mut self, old_log: File) {
        let mut buf = BufReader::new(old_log);
        let mut bytes = [0; 4096];
        let mut index: [u8; 8] = [0; 8];
        let mut count = 0;
        let mut value: u8;
        loop {
            match buf.read(&mut bytes) {
                Ok(0) => break,
                Ok(n) => {
                    for i in 0..n {
                        if count == 8 {
                            count = 0;
                            index = [0; 8];
                            value = bytes[i];
                            let offset: u64 = bytemuck::cast(index);
                            self.write(value, offset);
                        } else {
                            index[count] = bytes[i];
                            count += 1;
                        }
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => panic!("{:?}", e),
            };
        }
    }

    fn write(&self, value: u8, offset: u64) {
        todo!("Write value to database file at offset.");
    }

}

fn generate_log(dir_path: &Path, db_name: &String, log_count: u128) -> File {
    let mut new_name = db_name.clone();
    new_name.push_str(".log");
    new_name.push_str(&log_count.to_string());
    let new_filepath = dir_path.join(new_name);
    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(new_filepath)
        .expect("Failed to create new log file.")
}
