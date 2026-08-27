use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{RwLock};
use std::path::PathBuf;
use crate::memtables::Memtables;
use crate::entry::{Entry, EntryComparator};
use bytes::Bytes;

pub struct Db {
    sequence_number: AtomicU64,
    memtables: RwLock<Memtables>,
    path: PathBuf,
    config: DbConfig,
}

#[derive(Clone)]
pub struct DbConfig {
    pub(crate) max_height: usize,
    pub(crate) size_threshold: usize,
    pub(crate) comparator: EntryComparator,
    pub(crate) seed: Option<u64>,
}

impl Db {
    pub fn open(path: PathBuf, config: DbConfig) -> Db {
        Db {
            sequence_number: AtomicU64::new(0),
            memtables: RwLock::new(Memtables::open(config.clone())),
            path,
            config,
        }
    }

    pub fn get(&self, key: &Bytes) -> Option<Bytes> {
        let memtables = self.memtables.read().unwrap();
        match memtables.get(key) {
            Some(val) => Some(val.value),
            None => None,
        }
    }

    pub fn put(&self, key: Bytes, value: Bytes) {
        let new_entry: Entry = Entry::new_with_sequence(key, value, 0, self.sequence_number.fetch_add(1, Ordering::SeqCst));
        {
            let memtables = self.memtables.read().unwrap();
            memtables.insert(new_entry);
        }

        if self.memtables.read().unwrap().should_promote() {
            let mut memtables = self.memtables.write().unwrap();
            memtables.try_promote();
        }
    }

    pub fn delete(&self, key: Bytes) {
        let mut new_entry: Entry = Entry::new_with_sequence(key, Bytes::new(), 0, self.sequence_number.fetch_add(1, Ordering::SeqCst));
        new_entry.set_tombstone();
        {
            let memtables = self.memtables.read().unwrap();
            memtables.insert(new_entry);
        }

        if self.memtables.read().unwrap().should_promote() {
            let mut memtables = self.memtables.write().unwrap();
            memtables.try_promote();
        }
    }
}
