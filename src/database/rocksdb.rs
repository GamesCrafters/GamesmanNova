//! # RocksDB Database Applications
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use rocksdb::BlockBasedOptions;
use rocksdb::Cache;
use rocksdb::DB;
use rocksdb::Options;

use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use crate::frontend::IOMode;

/* CONSTANTS */

/// Environment variable with RocksDB path.
const ROCKSDB_DATABASE: &str = "ROCKSDB_DATABASE";

/// In bytes. Recall 1GB ~= 1_000_000_000B.
const CACHE_CAPACITY: usize = 10_000_000_000;

/// Write buffer size in bytes (256MB).
const WRITE_BUFFER_SIZE: usize = 256 * 1024 * 1024;

/// Maximum number of write buffers.
const MAX_WRITE_BUFFERS: i32 = 4;

/* FUNCTIONS */

pub fn init_rocksdb(mode: IOMode, name: &str) -> Result<Arc<DB>> {
    let base_path: PathBuf = env::var(ROCKSDB_DATABASE)
        .with_context(|| {
            format!("{ROCKSDB_DATABASE} environment variable must be set")
        })?
        .into();

    let path = base_path.join(name);
    if matches!(mode, IOMode::Overwrite) {
        let _ = DB::destroy(&Options::default(), &path);
    }

    let mut opts = Options::default();
    opts.create_if_missing(true);

    let cache = Cache::new_lru_cache(CACHE_CAPACITY);
    let mut block_opts = BlockBasedOptions::default();
    block_opts.set_block_cache(&cache);
    opts.set_block_based_table_factory(&block_opts);

    opts.set_manual_wal_flush(true);

    opts.set_enable_pipelined_write(true);
    opts.set_allow_concurrent_memtable_write(true);

    opts.set_write_buffer_size(WRITE_BUFFER_SIZE);
    opts.set_max_write_buffer_number(MAX_WRITE_BUFFERS);
    opts.set_enable_write_thread_adaptive_yield(true);

    opts.increase_parallelism(num_cpus::get() as i32);
    opts.set_max_background_jobs(6);

    opts.set_compression_type(rocksdb::DBCompressionType::Lz4);

    let db =
        DB::open(&opts, &path).context("Failed to open RocksDB database")?;
    Ok(Arc::new(db))
}
