use bytes::Bytes;
use lsm_tree::entry::Entry;

pub fn main() {
    let mut vec: Vec<Entry> = Vec::new();
    for i in 1..=3 {
        vec.push(Entry::new(
            Bytes::from(i.to_string()),
            Bytes::from((i + 1).to_string()),
            0,
        ));
    }
    println!("{:#?}", vec);
}
