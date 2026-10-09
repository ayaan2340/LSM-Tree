use std::io::BufWriter;
use std::path::PathBuf;
use bytes::Bytes;
use std::fs::File;
use crate::entry::Entry;

pub struct SSTableWriter {
    file: BufWriter<File>,
    index: Vec<(Bytes, u64)>,
    offset: u64,
}

impl SSTableWriter {
    pub fn new(path: PathBuf) -> std::io::Result<SSTableWriter> {
        let file = File::create(path)?;
        Ok(SSTableWriter {
            file: BufWriter::new(file),
            index: vec![],
            offset: 0,
        })
    }

    pub fn write_entry(&mut self, entry: Entry) {
        self.index.push((entry.key.clone(), self.offset));

    }
}
