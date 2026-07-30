use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::path::PathBuf;
use crate::memtable::Memtable;

struct Db {
    sequence_number: AtomicU64,
    active_memtable: Arc<Memtable>,
    path: PathBuf,
}
