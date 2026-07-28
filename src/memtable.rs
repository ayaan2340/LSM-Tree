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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::test_utils::{make_entry, make_memtable};

    fn insert(mt: &Memtable, key: &str, val: &str) {
        mt.insert(SkipEntry::new(make_entry(key, val)));
    }

    fn lookup(mt: &Memtable, key: &str) -> Option<SkipEntry> {
        mt.lookup(&Bytes::from(key.to_owned()))
    }

    #[test]
    fn insert_and_lookup_round_trip() {
        let mt = make_memtable(0);
        insert(&mt, "key", "val");
        let result = lookup(&mt, "key").expect("expected entry to be found");
        assert_eq!(result.entry().key, Bytes::from("key"));
        assert_eq!(result.entry().val.value, Bytes::from("val"));
    }

    #[test]
    fn lookup_missing_key_returns_none() {
        let mt = make_memtable(0);
        insert(&mt, "key", "val");
        assert!(lookup(&mt, "other").is_none());
    }

    #[test]
    fn entry_count_and_byte_size_reflect_inserts() {
        let mt = make_memtable(0);
        assert_eq!(0, mt.entry_count());
        assert_eq!(0, mt.byte_size());
        insert(&mt, "key", "val");
        assert_eq!(1, mt.entry_count());
        assert!(mt.byte_size() > 0);
        insert(&mt, "key2", "val2");
        assert_eq!(2, mt.entry_count());
    }

    #[test]
    fn concurrent_reads_and_writes() {
        let mt = Arc::new(make_memtable(0));

        for i in 0..10u32 {
            insert(&mt, &format!("pre{i}"), "val");
        }

        let mut handles = vec![];

        for i in 10..20u32 {
            let mt = Arc::clone(&mt);
            handles.push(std::thread::spawn(move || {
                insert(&mt, &format!("key{i}"), "val");
            }));
        }

        for i in 0..10u32 {
            let mt = Arc::clone(&mt);
            handles.push(std::thread::spawn(move || {
                let _ = lookup(&mt, &format!("pre{i}"));
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(20, mt.entry_count());
    }
}
