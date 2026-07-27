use std::sync::{RwLock};
use crate::skiplist::SkipList;
use crate::skiplist::skipnode::SkipEntry;
use crate::entry::EntryComparator;
use crate::skiplist::rng::SeededRng;
use bytes::Bytes;

pub(crate) struct MemtableInner {
    pub skiplist: SkipList,
}

impl MemtableInner {
    pub fn new(skiplist: SkipList) -> MemtableInner {
        MemtableInner {
            skiplist,
        }
    }
}

pub struct Memtable {
    id: u64,
    inner: RwLock<MemtableInner>,
}

impl Memtable {
    pub fn new(id: u64, max_height: usize, comparator: EntryComparator, rng: SeededRng) -> Memtable {
        Memtable {
            id,
            inner: RwLock::new(
                MemtableInner::new(SkipList::new(max_height, comparator, rng)),
            ),
        }
    }

    pub fn insert(&self, entry: SkipEntry) {
        let mut write_guard = self.inner.write().unwrap();
        write_guard.skiplist.insert(entry);
    }

    pub fn lookup(&self, key: &Bytes) -> Option<SkipEntry> {
        let read_guard = self.inner.read().unwrap();
        read_guard.skiplist.lookup(key).cloned()
    }

    pub fn entry_count(&self) -> usize {
        let read_guard = self.inner.read().unwrap();
        read_guard.skiplist.entry_count()
    }

    pub fn byte_size(&self) -> usize {
        let read_guard = self.inner.read().unwrap();
        read_guard.skiplist.byte_size()
    }
}
