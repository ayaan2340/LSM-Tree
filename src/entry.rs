use bytes::Bytes;
use std::cmp::Ordering;
use std::sync::Arc;
use crate::comparator::KeyComparator;

const TOMBSTONE: u8 = 1;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Entry {
    pub key: Bytes,
    pub val: Value,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Value {
    pub value: Bytes,
    pub(crate) metadata: u8,
    pub(crate) version: u64,
}

impl Entry {
    pub fn new(key: Bytes, value: Bytes, metadata: u8) -> Entry {
        Entry {
            key,
            val: Value {
                value,
                metadata,
                version: 0,
            },
        }
    }

    pub fn is_tombstone(&self) -> bool {
        self.val.metadata & TOMBSTONE == 1
    }
}

pub struct EntryComparator {
    comp: Arc<dyn KeyComparator<Bytes>>,
}

impl EntryComparator {
    pub fn new(comp: Arc<dyn KeyComparator<Bytes>>) -> EntryComparator {
        EntryComparator {
            comp,
        }
    }
}

// Key ascending, version descending
impl KeyComparator<Entry> for EntryComparator {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering {
       match self.comp.compare(&first.key, &second.key) {
            Ordering::Equal => second.val.version.cmp(&first.val.version),
            ordering => ordering,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{make_entry, make_versioned_entry, make_tombstone, make_comparator};

    #[test]
    fn entry_creation_sets_fields() {
        let key = Bytes::from(vec![191, 20, 32]);
        let value = Bytes::from("hello");
        let entry = Entry::new(key.clone(), value.clone(), 0);

        assert_eq!(key, entry.key);
        assert_eq!(value, entry.val.value);
        assert_eq!(0, entry.val.metadata);
        assert_eq!(0, entry.val.version);
    }

    #[test]
    fn is_tombstone_returns_false_for_normal_entry() {
        assert!(!make_entry("key", "val").is_tombstone());
    }

    #[test]
    fn is_tombstone_returns_true_for_tombstone_metadata() {
        assert!(make_tombstone("key").is_tombstone());
    }

    #[test]
    fn comparator_orders_lesser_key_first() {
        let comp = make_comparator();
        let a = make_entry("a", "val");
        let b = make_entry("b", "val");
        assert_eq!(Ordering::Less, comp.compare(&a, &b));
        assert_eq!(Ordering::Greater, comp.compare(&b, &a));
    }

    #[test]
    fn comparator_equal_keys_equal_versions_is_equal() {
        let comp = make_comparator();
        assert_eq!(Ordering::Equal, comp.compare(&make_entry("key", "v1"), &make_entry("key", "v2")));
    }

    #[test]
    fn comparator_orders_higher_version_first_for_same_key() {
        let comp = make_comparator();
        let higher = make_versioned_entry("key", "v2", 5);
        let lower = make_versioned_entry("key", "v1", 3);
        assert_eq!(Ordering::Less, comp.compare(&higher, &lower));
        assert_eq!(Ordering::Greater, comp.compare(&lower, &higher));
    }

    #[test]
    fn comparator_key_ordering_takes_precedence_over_version() {
        let comp = make_comparator();
        let a = make_versioned_entry("a", "val", 100);
        let b = make_versioned_entry("b", "val", 0);
        assert_eq!(Ordering::Less, comp.compare(&a, &b));
    }
}
