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
            node_list: vec!(SkipNode::dummy_height(max_height)),
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

    // Loop for searching through skiplist level for find_predecessors(),
    // Uses EntryComparator to keep sorted order for list including key and version
    fn should_move_forward_entry(&self, node: &SkipNode, new_entry: &SkipEntry, curr_level: usize) -> bool {
        node.get_forward()[curr_level]
            .and_then(|idx| self.node_list[idx].get_entry().as_ref())
            .map(|next_entry|  self.comparator.compare(next_entry.entry(), new_entry.entry()) == Ordering::Less)
            .unwrap_or(false)
    }

    // Returns index into node_list
    pub fn search(&self, key: &Bytes) -> usize {
        let mut curr_level: usize = self.max_height;
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

    // Loop for searching through skiplist level for search(),
    // Uses less than or equal to key comparisons to find last version of a key
    fn should_move_forward(&self, node: &SkipNode, key: &Bytes, curr_level: usize) -> bool {
        node.get_forward()[curr_level]
            .and_then(|idx| self.node_list[idx].get_entry().as_ref())
            .map(|next_entry| next_entry.entry().key.cmp(key) == Ordering::Less)
            .unwrap_or(false)
    }

    // Returns latest entry if matching key found in skiplist
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{make_entry, make_tombstone, make_skiplist};

    fn insert(sl: &mut SkipList, key: &str, val: &str) {
        sl.insert(SkipEntry::new(make_entry(key, val)));
    }

    fn lookup<'a>(sl: &'a SkipList, key: &str) -> Option<&'a SkipEntry> {
        sl.lookup(&Bytes::from(key.to_owned()))
    }

    #[test]
    fn lookup_on_empty_skiplist_returns_none() {
        let sl = make_skiplist();
        assert!(lookup(&sl, "missing").is_none());
    }

    #[test]
    fn insert_single_entry_lookup_finds_it() {
        let mut sl = make_skiplist();
        insert(&mut sl, "key", "val");
        let result = lookup(&sl, "key").expect("expected entry to be found");
        assert_eq!(result.entry().key, Bytes::from("key"));
        assert_eq!(result.entry().val.value, Bytes::from("val"));
    }

    #[test]
    fn lookup_missing_key_returns_none() {
        let mut sl = make_skiplist();
        insert(&mut sl, "key", "val");
        assert!(lookup(&sl, "other").is_none());
    }

    #[test]
    fn insert_multiple_unsorted_all_found() {
        let mut sl = make_skiplist();
        let keys = ["delta", "alpha", "charlie", "bravo", "echo"];
        for key in &keys {
            insert(&mut sl, key, "val");
        }
        for key in &keys {
            assert!(lookup(&sl, key).is_some(), "missing key: {key}");
        }
    }

    #[test]
    fn entry_count_increments_per_insert() {
        let mut sl = make_skiplist();
        assert_eq!(0, sl.entry_count());
        insert(&mut sl, "a", "val");
        assert_eq!(1, sl.entry_count());
        insert(&mut sl, "b", "val");
        assert_eq!(2, sl.entry_count());
    }

    #[test]
    fn byte_size_increases_per_insert() {
        let mut sl = make_skiplist();
        assert_eq!(0, sl.byte_size());
        insert(&mut sl, "a", "val");
        let after_one = sl.byte_size();
        assert!(after_one > 0);
        insert(&mut sl, "bb", "val");
        assert!(sl.byte_size() > after_one);
    }

    #[test]
    fn insert_tombstone_is_found_by_lookup() {
        let mut sl = make_skiplist();
        sl.insert(SkipEntry::new(make_tombstone("gone")));
        let result = lookup(&sl, "gone").expect("tombstone entry should be found");
        assert!(result.entry().is_tombstone());
    }
}
