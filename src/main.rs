use bytes::Bytes;
use lsm_tree::entry::Entry;
use std::sync::Mutex;


pub fn main() {
    let write_lock: Mutex<u64> = Mutex::new(0);
    let mut vec: Vec<Entry> = Vec::new();
    for i in 1..5 {
        vec.push(Entry::new(
            Bytes::from(i.to_string()),
            Bytes::from((i + 1).to_string()),
            0));
    }
    println!("{:#?}", vec);
}
