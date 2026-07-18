use bytes::Bytes;
use std::cmp::Ordering;
use std::sync::Arc;
use crate::comparator::KeyComparator;

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

impl KeyComparator<Entry> for EntryComparator {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering {
        return self.comp.compare(&first.key, &second.key);
    }
}
