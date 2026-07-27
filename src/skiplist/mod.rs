pub mod rng;
pub mod skipnode;

use self::skipnode::{SkipEntry, SkipNode};
use crate::entry::{EntryComparator, Entry};
use crate::comparator::KeyComparator;
use self::rng::SeededRng;
use bytes::Bytes;
use std::cmp::Ordering;
use std::mem::size_of;

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
    
    pub fn entry_count(&self) -> usize {
        self.entry_count
    }
    
    pub fn byte_size(&self) -> usize {
        self.byte_size
    }

    pub fn insert(&mut self, entry: SkipEntry) {
        let key_size = entry.entry().key.len();
        let value_size = entry.entry().val.value.len();

        // Previous nodes on all levels
        let predecessors: Vec<usize> = self.find_predecessors(&entry);
        let new_height: usize = self.random_height();

        let mut new_node: SkipNode = SkipNode::new(entry);
        let new_idx: usize = self.node_list.len();

        // Insert into list on all levels
        let forward: &mut Vec<Option<usize>> = new_node.get_forward_mut();
        for i in 0..new_height {
            let prev_node = &mut self.node_list[predecessors[i]];
            forward.push(prev_node.get_forward()[i]);
            prev_node.get_forward_mut()[i] = Some(new_idx);
        }

        self.node_list.push(new_node);

        // Update metadata
        self.entry_count += 1;


        let overhead = size_of::<Entry>();
        self.byte_size += key_size + value_size + overhead;
    }
    
    fn random_height(&mut self) -> usize {
        let mut height = 1;
        while height < self.max_height && self.rng.coinflip() {
            height += 1
        }
        
        height 
    }

    fn find_predecessors(&self, entry: &SkipEntry) -> Vec<usize> {
        let mut pred: Vec<usize> = vec![0; self.max_height];
        let mut curr_idx: usize = 0;
        for i in (0..self.max_height).rev() {
            let mut curr_node: &SkipNode = &self.node_list[curr_idx];

            while self.should_move_forward_entry(curr_node, entry, i) {
                curr_idx = curr_node.get_forward()[i].unwrap();
                curr_node = &self.node_list[curr_idx];
            }
            
            pred[i] = curr_idx;
        }
        
        pred
    }

    fn should_move_forward_entry(&self, node: &SkipNode, new_entry: &SkipEntry, curr_level: usize) -> bool {
        let next_exists: bool = node.get_forward()[curr_level].is_some(); 
        match node.get_entry() {
            None if next_exists => true,
            Some(entry) if next_exists => self.comparator.compare(entry.entry(), new_entry.entry()) == Ordering::Less,
            _ => false,
        }
    }
    // Returns index into node_list
    pub fn search(&self, key: &Bytes) -> usize {
        let mut curr_level: usize = self.max_height - 1;
        let mut curr_idx: usize = 0;
        while curr_level > 0 {
            curr_level -= 1;
            let mut curr_node: &SkipNode = &self.node_list[curr_idx];

            while self.should_move_forward(curr_node, key, curr_level) {
                curr_idx = curr_node.get_forward()[curr_level].unwrap();
                curr_node = &self.node_list[curr_idx];
            }
        }
        
        curr_idx
    }

    fn should_move_forward(&self, node: &SkipNode, key: &Bytes, curr_level: usize) -> bool {
        let next_exists: bool = node.get_forward()[curr_level].is_some(); 
        match node.get_entry() {
            None if next_exists => true,
            Some(entry) if next_exists => entry.entry().key.cmp(key) == Ordering::Less,
            _ => false,
        }
    }

    // Returns entry if matching key found in skiplist
    pub fn lookup(&self, key: &Bytes) -> Option<&SkipEntry> {
        let node: &SkipNode = &self.node_list[self.search(key)];
        if let Some(entry) = node.get_entry() {
            let e = entry.entry();
            match e.key.cmp(key) {
                Ordering::Equal => Some(entry),
                _ => None,
            }
        } else { 
            None
        }

    }
}
