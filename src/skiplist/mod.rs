use crate::skipnode::{SkipEntry, SkipNode};
use crate::entry::EntryComparator;
use crate::rng::SeededRng;
use bytes::Bytes;

pub struct SkipList {
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

    pub fn search(&self, key: Bytes) {

    }

    pub fn lookup(&self, key: Bytes) -> Option<SkipEntry> {
    
    }
}
