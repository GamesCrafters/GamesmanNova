//! # Bit Perfect Database Module
//!
//! This module contains the implementation for a bit-perfect database.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/14/2023 (maxfierro@berkeley.edu)

type Index = State;

use super::DatabaseMode;
use crate::core::State;
use crossbeam_channel::{Receiver, Sender};
use std::fs::{File, OpenOptions};
use std::io::{Write, BufReader, Read, ErrorKind};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{thread, thread::JoinHandle};

// Half a GiB
const MAX_LOG_SIZE: u32 = 500_000_000;

// Multiple of available features
const PAGE_BYTE_SIZE: usize = 64;

pub struct StreamDB<'a> {
    db_name: String,
    db_mode: DatabaseMode,
    dir_path: &'a Path,

    memory: Vec<Mutex<[u8; PAGE_BYTE_SIZE]>>,
    update_count: u128,

    logger: Option<Logger<'a>>,
    logger_handle: Option<JoinHandle<()>>,
    logger_sender: Option<Sender<LoggerMessage>>,
}

#[derive(Clone)]
struct Record {
    index: Index,
    value: u8,
}

impl StreamDB<'_> {
    fn initialize(name: String, mode: DatabaseMode, path: &Path) -> Self {
        match mode {
            DatabaseMode::ReadOnly | DatabaseMode::Virtual => StreamDB {
                db_name: name,
                db_mode: mode,
                dir_path: path,
                memory: Vec::new(),
                update_count: 0,
                logger: None,
                logger_handle: None,
                logger_sender: None,
            },
            DatabaseMode::Default => {
                let log_file = generate_log(path, &name, 0);
                let logger = Logger::initialize(log_file, path, &name);
                let (logger_sender, logger_receiver) = crossbeam_channel::unbounded();
                let logger_handle = logger.listen(logger_receiver);
                StreamDB {
                    db_name: name,
                    db_mode: mode,
                    dir_path: path,
                    memory: Vec::new(),
                    update_count: 0,
                    logger: Some(logger),
                    logger_handle: Some(logger_handle),
                    logger_sender: Some(logger_sender),
                }
            }
        }
    }
}

/* LOGGING */

#[derive(Clone)]
enum LoggerMessage {
    DoCheckpoint,
    Update(Record),
}

struct Logger<'a> {
    log_file: Arc<File>,
    dir_path: &'a Path,
    log_count: u128,

    writer: Writer<'a>,
    writer_handle: JoinHandle<()>,
    writer_sender: Sender<WriterMessage>,
}

impl Logger<'_> {
    fn initialize(log_file: File, dir_path: &Path, db_name: &String) -> Self {
        let log_file = Arc::new(log_file);
        let writer = Writer::new(Arc::clone(&log_file), db_name);
        let (writer_sender, writer_receiver) = crossbeam_channel::unbounded();
        let writer_handle = writer.listen(writer_receiver);
        Logger {
            log_file,
            dir_path,
            log_count: 1,
            writer,
            writer_handle,
            writer_sender,
        }
    }

    fn listen(&self, rx: Receiver<LoggerMessage>) -> JoinHandle<()> {
        thread::spawn(move || {
            for received in rx {
                match received {
                    LoggerMessage::DoCheckpoint => self.checkpoint(),
                    LoggerMessage::Update(record) => self.update(record),
                };
            }
        })
    }

    fn update(&self, record: Record) {
        let mut data: [u8; 9];
        let index: [u8; 8] = bytemuck::cast(record.index);
        data[..8].copy_from_slice(&index);
        data[8] = record.value;
        if let Err(_) = self.log_file.write_all(&data) {
            self.log_file.flush();
            panic!("Failed to write update to log");
        }
    }

    fn checkpoint(&self) {
        self.writer_sender.send(WriterMessage::DoCheckpoint);
        self.log_file = Arc::new(generate_log(
            self.dir_path,
            self.writer.db_name,
            self.log_count,
        ));
        self.log_count += 1;
        self.writer_sender
            .send(WriterMessage::UpdateLog(self.log_file));
    }
}

/* CHECKPOINTS */

enum WriterMessage {
    DoCheckpoint,
    UpdateLog(Arc<File>),
}

struct Writer<'a> {
    log_file: Arc<File>,
    db_name: &'a String,
}

impl Writer<'_> {
    fn new(log_file: Arc<File>, db_name: &String) -> Self {
        Writer { log_file, db_name }
    }

    fn listen(&self, rx: Receiver<WriterMessage>) -> JoinHandle<()> {
        thread::spawn(move || {
            for received in rx {
                match received {
                    WriterMessage::DoCheckpoint => self.checkpoint(),
                    WriterMessage::UpdateLog(file) => self.update_log(file),
                }
            }
        })
    }

    fn checkpoint(&self) {
        let mut buf = BufReader::new(*self.log_file);
        let mut bytes = [0; 512];
        let mut curr_index: [u8; 8];
        let mut curr_value: u8;
        loop {
            match buf.read(&mut bytes) {
                Ok(0) => break,
                Ok(n) => {
                    for i in 0..n {
                        
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => panic!("{:?}", e),
            };
        }
        
    }

    fn update_log(&mut self, new_log: Arc<File>) {
        self.log_file = new_log;
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
