use std::sync::{RwLock, RwLockReadGuard};
use crate::iterator::StorageIterator;
use crate::skiplist::{SkipList};
use crate::skiplist::skipnode::SkipEntry;
use crate::entry::{EntryComparator, Value};
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

pub(crate) struct Memtable {
    id: u64,
    inner: RwLock<MemtableInner>,
}

pub(crate) struct MemtableIter<'a> {
    guard: RwLockReadGuard<'a, MemtableInner>,
    idx: usize,
    current: Option<SkipEntry>
}

impl<'a> StorageIterator for MemtableIter<'a> {
    type Error = ();
    fn key(&self) -> &[u8] {
       &self.current.as_ref().expect("key() called on an invalid iterator").entry().key
    }

    fn value(&self) -> &Value {
       &self.current.as_ref().expect("value() called on an invalid iterator").entry().val
    }

    fn is_valid(&self) -> bool {
        self.current.is_some()
    }

    fn next(&mut self) -> Result<(), Self::Error> {
       if let Some(next_idx) = self.guard.skiplist.node_list[self.idx].get_forward()[0] {
            self.idx = next_idx;
            self.current = self.guard.skiplist.node_list[next_idx].get_entry().clone();
       } else {
            self.current = None;
        }
        Ok(())
    }

    // Caller responsibility to check version of entry
    fn seek(&mut self, key: &[u8]) -> Result<(), Self::Error> {
        while self.is_valid() && self.key() < key {
           let _ = self.next(); 
        }
        Ok(())
    }
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

    pub fn iter(&self) -> MemtableIter<'_> {
        let read_guard: RwLockReadGuard<MemtableInner> = self.inner.read().unwrap();
        // First index is the node after the dummy head node
        let first_idx: usize = read_guard.skiplist.node_list[0].get_forward()[0].unwrap_or(0);
        let first_entry: Option<SkipEntry> = read_guard.skiplist.node_list[first_idx].get_entry().clone();
        MemtableIter {
            guard: read_guard,
            idx: first_idx,
            current: first_entry,
        }
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

    fn collect_keys(mt: &Memtable) -> Vec<Vec<u8>> {
        let mut iter = mt.iter();
        let mut keys = vec![];
        while iter.is_valid() {
            keys.push(iter.key().to_vec());
            iter.next().unwrap();
        }
        keys
    }

    #[test]
    fn iter_on_empty_memtable_is_invalid() {
        let mt = make_memtable(0);
        assert!(!mt.iter().is_valid());
    }

    #[test]
    fn iter_single_entry_key_and_value_correct() {
        let mt = make_memtable(0);
        insert(&mt, "key", "val");
        let iter = mt.iter();
        assert!(iter.is_valid());
        assert_eq!(iter.key(), b"key");
        assert_eq!(iter.value().value, Bytes::from("val"));
    }

    #[test]
    fn iter_traverses_in_sorted_order() {
        let mt = make_memtable(0);
        let keys_unsorted = ["delta", "alpha", "charlie", "bravo", "echo"];
        for key in &keys_unsorted {
            insert(&mt, key, "val");
        }
        let collected = collect_keys(&mt);
        let mut expected: Vec<Vec<u8>> = keys_unsorted.iter().map(|k| k.as_bytes().to_vec()).collect();
        expected.sort();
        assert_eq!(collected, expected);
    }

    #[test]
    fn iter_next_past_end_is_invalid() {
        let mt = make_memtable(0);
        insert(&mt, "only", "val");
        let mut iter = mt.iter();
        assert!(iter.is_valid());
        iter.next().unwrap();
        assert!(!iter.is_valid());
    }

    #[test]
    fn seek_lands_on_exact_key() {
        let mt = make_memtable(0);
        for key in &["alpha", "bravo", "charlie"] {
            insert(&mt, key, "val");
        }
        let mut iter = mt.iter();
        iter.seek(b"bravo").unwrap();
        assert!(iter.is_valid());
        assert_eq!(iter.key(), b"bravo");
    }

    #[test]
    fn seek_lands_on_next_greater_when_exact_missing() {
        let mt = make_memtable(0);
        for key in &["alpha", "charlie", "echo"] {
            insert(&mt, key, "val");
        }
        let mut iter = mt.iter();
        iter.seek(b"bravo").unwrap();
        assert!(iter.is_valid());
        assert_eq!(iter.key(), b"charlie");
    }

    #[test]
    fn seek_past_all_keys_is_invalid() {
        let mt = make_memtable(0);
        for key in &["alpha", "bravo"] {
            insert(&mt, key, "val");
        }
        let mut iter = mt.iter();
        iter.seek(b"zzzz").unwrap();
        assert!(!iter.is_valid());
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
