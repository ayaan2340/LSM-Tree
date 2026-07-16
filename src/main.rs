mod entry;
use bytes::Bytes;
use crate::entry::Entry;

pub fn main() {
    let mut vec: Vec<Entry> = Vec::new();
    for i in 1..5 {
        vec.push(Entry::new(
            Bytes::from(i.to_string()),
            Bytes::from((i + 1).to_string()),
            0));
    }

    println!("{:#?}", vec);
}
