use std::path::Path;

pub mod engine;
pub mod page;

pub enum DatabaseMode<'a> {
    Virtual,
    Persistent(&'a Path),
}
