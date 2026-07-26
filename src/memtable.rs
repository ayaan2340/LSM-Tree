use std::sync::{RwLock};
use crate::skiplist::SkipList;
use crate::skiplist::skipnode::SkipEntry;
struct MemtableInner {
    skiplist: SkipList,
}

struct Memtable {
    id: u64,
    inner: RwLock<MemtableInner>,
}
