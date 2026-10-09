use std::sync::Arc;
use std::path::PathBuf;
use bytes::Bytes;
use crate::comparator::BytewiseComparator;
use crate::db::{Db, DbConfig};
use crate::entry::{Entry, EntryComparator, Value};
use crate::memtable::Memtable;
use crate::skiplist::SkipList;
use crate::skiplist::rng::SeededRng;

const TOMBSTONE: u8 = 1;
const DEFAULT_MAX_HEIGHT: usize = 12;
const DEFAULT_SEED: u64 = 42;

pub(crate) fn make_entry(key: &str, val: &str) -> Entry {
    Entry::new(Bytes::from(key.to_owned()), Bytes::from(val.to_owned()), 0)
}

pub(crate) fn make_tombstone(key: &str) -> Entry {
    Entry::new(Bytes::from(key.to_owned()), Bytes::new(), TOMBSTONE)
}

pub(crate) fn make_versioned_entry(key: &str, val: &str, sequence_number: u64) -> Entry {
    Entry {
        key: Bytes::from(key.to_owned()),
        val: Value { value: Bytes::from(val.to_owned()), metadata: 0, sequence_number },
    }
}

pub(crate) fn make_comparator() -> EntryComparator {
    EntryComparator::new(Arc::new(BytewiseComparator {}))
}

pub(crate) fn make_skiplist() -> SkipList {
    SkipList::new(DEFAULT_MAX_HEIGHT, make_comparator(), SeededRng::new(DEFAULT_SEED))
}

pub(crate) fn make_memtable(id: u64) -> Memtable {
    Memtable::new(id, DEFAULT_MAX_HEIGHT, make_comparator(), SeededRng::new(DEFAULT_SEED))
}

pub(crate) fn make_db_config(size_threshold: usize) -> DbConfig {
    DbConfig {
        max_height: DEFAULT_MAX_HEIGHT,
        size_threshold,
        comparator: make_comparator(),
        seed: Some(DEFAULT_SEED),
    }
}

pub(crate) fn make_db(size_threshold: usize) -> Db {
    Db::open(PathBuf::from("/tmp/lsm_test"), make_db_config(size_threshold))
}
