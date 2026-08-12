use crate::memtable::Memtable;
use crate::skiplist::rng::SeededRng;
use crate::entry::{Entry, Value};
use crate::skiplist::skipnode::SkipEntry;
use crate::db::DbConfig;
use std::sync::Arc;
use std::collections::VecDeque;
use bytes::Bytes;

pub(crate) struct Memtables {
    active: Arc<Memtable>,
    immutable_list: VecDeque<Arc<Memtable>>,
    config: DbConfig,
    next_id: u64,
}
pub(crate) struct MemtablesIter {
    index: usize,
    memtables: Vec<Arc<Memtable>>,
}

impl MemtablesIter {
    pub fn new(memtables: Vec<Arc<Memtable>>) -> MemtablesIter {
        MemtablesIter {
            index: 0,
            memtables,
        }
    }
}

impl Iterator for MemtablesIter {
    type Item = Arc<Memtable>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.memtables.get(self.index - 1).cloned()
   }
}

impl Memtables {
    pub fn open(config: DbConfig) -> Memtables {
        let rng: SeededRng = match config.seed {
            Some(seed) => SeededRng::new(seed),
            None => SeededRng::from_entropy(),
        };
        Memtables {
            active: Arc::new(Memtable::new(0, config.max_height, config.comparator.clone(), rng)),
            immutable_list: VecDeque::new(),
            config,
            next_id: 1,
        }
    }

    pub fn new_table(&mut self) {
        let rng: SeededRng = match self.config.seed {
            Some(seed) => SeededRng::new(seed),
            None => SeededRng::from_entropy(),
        };
        self.immutable_list.push_front(self.active.clone());
        self.active = Arc::new(Memtable::new(self.next_id, self.config.max_height, self.config.comparator.clone(), rng));
        self.next_id += 1;
    }

    pub fn insert(&self, entry: Entry) {
        self.active.insert(SkipEntry::new(entry));
    }

    pub fn get(&self, key: &Bytes) -> Option<Value> {
        for table in self.iter() {
            if let Some(entry) = table.lookup(key) {
                return Some(entry.entry().val.clone());
            }
        } 
        None
    }

    pub fn should_promote(&self) -> bool {
        self.active.byte_size() > self.config.size_threshold
    }

    fn iter(&self) -> MemtablesIter {
        let mut memtables_list: Vec<Arc<Memtable>> = Vec::new();
        memtables_list.push(self.active.clone());
        memtables_list.extend(self.immutable_list.iter().cloned());
        MemtablesIter {
            index: 0,
            memtables: memtables_list,
        }
    }
}
