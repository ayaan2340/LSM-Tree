pub mod rng;
pub mod skipnode;

use self::skipnode::{SkipEntry, SkipNode};
use crate::entry::EntryComparator;
use self::rng::SeededRng;
use bytes::Bytes;
use std::cmp::Ordering;

pub(crate) struct SkipList {
    node_list: Vec<SkipNode>,
    max_height: usize,
    comparator: EntryComparator,
    entry_count: usize,
    byte_size: usize,
    rng: SeededRng,
}

impl SkipList {
    pub fn new(max_height: usize, comparator: EntryComparator, rng: SeededRng) -> SkipList {
        SkipList {
            node_list: vec!(SkipNode::dummy()),
            max_height,
            comparator,
            entry_count: 0,
            byte_size: 0,
            rng,
        }
    }

    pub fn insert(&mut self, entry: SkipEntry) -> Result<(), String> {
        
    
        // Search for new location and generate new height
        // Find previous and next nodes and connect them, then add current node to node_list
        Ok(())
    }

    pub fn search(&self, key: &Bytes) -> &SkipNode {

    }

    pub fn lookup(&self, key: &Bytes) -> Option<&SkipEntry> {
        let node: &SkipNode = self.search(key);
        if let Some(entry) = node.get_entry() {
            let e = entry.entry();
            match e.key.cmp(key) {
                Ordering::Equal => Some(entry),
                _ => None,
            }
        } else { return None }

    }
}
