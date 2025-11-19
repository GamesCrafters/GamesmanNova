//! # Developer Applications
//!
//! TODO

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use petgraph::Graph;
use petgraph::graph::NodeIndex;
use rusqlite::Connection;
use strum_macros::Display;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::RwLock;

/* CONSTANTS */

/// Global lock for creating development data directories. Since `cargo test`
/// executes tests in parallel, this helps prevent test flakiness by avoiding
/// race conditions when creating directories (specific files are still
/// susceptible, but they have no specific structure).
pub static DIRECTORY_LOCK: RwLock<()> = RwLock::new(());

/// The name of the global directory at the project root used for generated
/// development data. This directory is not shipped with release builds.
pub const DEV_DIRECTORY: &str = "dev";

/* ENUMERATIONS */

/// Specifies directories for different kinds of data generated for development
/// purposes, which should not be distributed.
#[derive(Display)]
#[strum(serialize_all = "kebab-case")]
pub enum DevelopmentData {
    Visuals,
    Sled,
    SledTest,
}

/// Specifies the level of side effects to generate during testing. This
/// corresponds to the `TEST_SETTING` environment variable.
pub enum TestSetting {
    Correctness,
    Development,
}

/* STRUCTURES */

/// In Nova, many objects (namely game states and scheduler tasks) are organized
/// as graphs. For testing purposes, it is useful to have an abstraction to make
/// graph structures out of these objects for testing in an ergonomic fashion.
#[derive(Default)]
pub struct GraphBuilder<'a, T> {
    pub inserted: HashMap<*const T, NodeIndex>,
    pub graph: Graph<&'a T, ()>,
}

/* IMPLEMENTATIONS */

impl<'a, T> GraphBuilder<'a, T> {
    pub fn edge(mut self, from: &'a T, to: &'a T) -> Self {
        let i = *self
            .inserted
            .entry(from as *const T)
            .or_insert_with(|| self.graph.add_node(from));

        let j = *self
            .inserted
            .entry(to as *const T)
            .or_insert_with(|| self.graph.add_node(to));

        self.graph.update_edge(i, j, ());
        self
    }
}

/* FUNCTIONS */

/// Parses environment variables and establishes an SQLite connection to the
/// global game solution testing database.
pub fn test_database() -> Result<Connection> {
    let db = match test_setting()? {
        TestSetting::Correctness => Connection::open_in_memory()
            .context("Failed to establish connection to in-memory database.")?,
        TestSetting::Development => {
            let path = env::var("TEST_DATABASE")
                .context("DATABASE environment variable not set.")?;

            Connection::open(&path).context(format!(
                "Failed to initialize SQLite connection to {}",
                path
            ))?
        },
    };

    db.execute(
        "PRAGMA synchronous = OFF; \
            PRAGMA journal_mode = MEMORY; \
            PRAGMA temp_store = MEMORY;",
        [],
    )
    .context("Failed to tune SQLite database options.")?;
    Ok(db)
}

/// Returns a Sled database for testing. In correctness mode, uses an in-memory
/// temporary database. In development mode, uses a persistent database in the
/// dev/sled directory, deleting any existing data on initialization.
pub fn test_sled_db(module: &str) -> Result<sled::Db> {
    match test_setting()? {
        TestSetting::Correctness => sled::Config::new()
            .temporary(true)
            .open()
            .context("Failed to open temporary Sled database"),
        TestSetting::Development => {
            let path =
                get_directory(DevelopmentData::Sled, PathBuf::from(module))?;

            if path.exists() {
                fs::remove_dir_all(&path)
                    .context("Failed to remove existing Sled database")?;
            }

            sled::open(&path).context(format!(
                "Failed to open Sled database at {}",
                path.display()
            ))
        },
    }
}

/// Returns the testing side effects setting as obtained from the `TEST_SETTING`
/// environment variable.
pub fn test_setting() -> Result<TestSetting> {
    if let Ok(setting) = env::var("TEST_SETTING") {
        match &setting[..] {
            "correctness" => Ok(TestSetting::Correctness),
            "development" => Ok(TestSetting::Development),
            _ => bail!("TEST_SETTING assignment '{setting}' not recognized."),
        }
    } else {
        Ok(TestSetting::Development)
    }
}

/// Returns a PathBuf corresponding to the correct subdirectory for storing
/// development `data` at a `module`-specific subdirectory, creating it in the
/// process if it does not exist.
pub fn get_directory(
    data: DevelopmentData,
    module: PathBuf,
) -> Result<PathBuf> {
    let root = find_cargo_lock_directory()
        .context("Failed to find project root directory.")?;

    let directory = root
        .join(DEV_DIRECTORY)
        .join(format!("{data}"))
        .join(module);

    let guard = {
        let _lock = DIRECTORY_LOCK.read().unwrap();
        directory.try_exists()?
    };

    if !guard {
        // Does not completely prevent multiple threads from attempting to
        // create the same directory path, but `create_dir_all` is resilient
        // to this regardless. This is only necessary for preventing race
        // conditions within `find_cargo_lock_directory`.
        let _lock = DIRECTORY_LOCK.write().unwrap();
        fs::create_dir_all(&directory)
            .context("Failed to create module subdirectory.")?;
    }

    Ok(directory)
}

/// Creates an SVG visualization of a graph using Graphviz dot.
/// Only generates output in Development test mode.
pub fn visualize_graph(
    graph_dot: &str,
    name: &str,
    module: &str,
) -> Result<()> {
    match test_setting()? {
        TestSetting::Correctness => return Ok(()),
        TestSetting::Development => (),
    }

    let subdir = PathBuf::from(module);
    let mut dir = get_directory(DevelopmentData::Visuals, subdir)?;
    let filename = format!("{}.svg", name).replace(' ', "-");

    dir.push(filename);
    let file = File::create(dir)?;
    let mut dot = Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(file)
        .spawn()
        .context("Failed to execute 'dot' command.")?;

    if let Some(mut stdin) = dot.stdin.take() {
        stdin.write_all(graph_dot.as_bytes())?;
    }

    dot.wait()?;
    Ok(())
}

/* HELPER FUNCTIONS */

/// Searches for a parent directory containing a `Cargo.lock` file.
fn find_cargo_lock_directory() -> Result<PathBuf> {
    let _lock = DIRECTORY_LOCK.read().unwrap();
    let mut cwd = env::current_dir()?;
    loop {
        let cargo_lock = cwd.join("Cargo.lock");
        if cargo_lock.try_exists()? && cargo_lock.is_file() {
            return Ok(cwd);
        }
        if let Some(parent_dir) = cwd.parent() {
            cwd = parent_dir.to_owned();
        } else {
            break;
        }
    }
    bail!("Could not find any parent directory with a Cargo.lock file.")
}
