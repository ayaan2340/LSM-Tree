use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::path::PathBuf;
use crate::memtable::Memtable;
use crate::entry::EntryComparator;
use crate::skiplist::rng::SeededRng;

pub struct Db {
    sequence_number: AtomicU64,
    active_memtable: Arc<Memtable>,
    path: PathBuf,
}

pub struct DbConfig {
    pub max_height: usize,
    pub size_threshold: usize,
    pub comparator: EntryComparator,
    pub seed: Option<u64>,
}
