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


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::test_utils::make_db;

    #[test]
    fn put_and_get_basic() {
        let db = make_db(usize::MAX);
        db.put(Bytes::from("key"), Bytes::from("value"));
        assert_eq!(db.get(&Bytes::from("key")), Some(Bytes::from("value")));
    }

    #[test]
    fn get_missing_key_returns_none() {
        let db = make_db(usize::MAX);
        assert_eq!(db.get(&Bytes::from("missing")), None);
    }

    #[test]
    fn put_same_key_twice_returns_latest_value() {
        let db = make_db(usize::MAX);
        db.put(Bytes::from("key"), Bytes::from("v1"));
        db.put(Bytes::from("key"), Bytes::from("v2"));
        assert_eq!(db.get(&Bytes::from("key")), Some(Bytes::from("v2")));
    }

    #[test]
    fn delete_makes_key_invisible() {
        let db = make_db(usize::MAX);
        db.put(Bytes::from("key"), Bytes::from("value"));
        db.delete(Bytes::from("key"));
        assert_eq!(db.get(&Bytes::from("key")), None);
    }

    #[test]
    fn delete_nonexistent_key_is_noop() {
        let db = make_db(usize::MAX);
        db.delete(Bytes::from("ghost"));
        assert_eq!(db.get(&Bytes::from("ghost")), None);
    }

    #[test]
    fn put_after_delete_restores_key() {
        let db = make_db(usize::MAX);
        db.put(Bytes::from("key"), Bytes::from("v1"));
        db.delete(Bytes::from("key"));
        db.put(Bytes::from("key"), Bytes::from("v2"));
        assert_eq!(db.get(&Bytes::from("key")), Some(Bytes::from("v2")));
    }

    #[test]
    fn all_keys_findable_after_promotion() {
        let db = make_db(128);
        for i in 0..50u32 {
            db.put(Bytes::from(format!("key{i}")), Bytes::from(format!("val{i}")));
        }
        for i in 0..50u32 {
            assert_eq!(
                db.get(&Bytes::from(format!("key{i}"))),
                Some(Bytes::from(format!("val{i}")))
            );
        }
    }

    #[test]
    fn delete_survives_promotion() {
        let db = make_db(128);
        db.put(Bytes::from("target"), Bytes::from("value"));
        db.delete(Bytes::from("target"));
        for i in 0..50u32 {
            db.put(Bytes::from(format!("filler{i}")), Bytes::from(format!("val{i}")));
        }
        assert_eq!(db.get(&Bytes::from("target")), None);
    }

    #[test]
    fn concurrent_puts_all_visible() {
        let db = Arc::new(make_db(usize::MAX));
        let mut handles = vec![];

        for i in 0..20u32 {
            let db = Arc::clone(&db);
            handles.push(std::thread::spawn(move || {
                db.put(Bytes::from(format!("key{i}")), Bytes::from(format!("val{i}")));
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        for i in 0..20u32 {
            assert!(
                db.get(&Bytes::from(format!("key{i}"))).is_some(),
                "key{i} missing after concurrent puts"
            );
        }
    }

    #[test]
    fn concurrent_puts_and_gets_no_panic() {
        let db = Arc::new(make_db(usize::MAX));

        for i in 0..10u32 {
            db.put(Bytes::from(format!("pre{i}")), Bytes::from(format!("val{i}")));
        }

        let mut handles = vec![];

        for i in 10..20u32 {
            let db = Arc::clone(&db);
            handles.push(std::thread::spawn(move || {
                db.put(Bytes::from(format!("key{i}")), Bytes::from(format!("val{i}")));
            }));
        }

        for i in 0..10u32 {
            let db = Arc::clone(&db);
            handles.push(std::thread::spawn(move || {
                let _ = db.get(&Bytes::from(format!("pre{i}")));
            }));
        }

        for h in handles {
            h.join().unwrap();
        }
    }

    #[test]
    fn concurrent_puts_with_promotion() {
        let db = Arc::new(make_db(128));
        let mut handles = vec![];

        for i in 0..50u32 {
            let db = Arc::clone(&db);
            handles.push(std::thread::spawn(move || {
                db.put(Bytes::from(format!("key{i}")), Bytes::from(format!("val{i}")));
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        for i in 0..50u32 {
            assert!(
                db.get(&Bytes::from(format!("key{i}"))).is_some(),
                "key{i} missing after concurrent puts with promotion"
            );
        }
    }
}
