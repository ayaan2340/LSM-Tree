use crate::entry::Entry;

pub struct SkipEntry {
    pub entry: Entry,
}

impl SkipEntry {
    pub fn new(entry: Entry) -> SkipEntry {
        SkipEntry {
            entry,
        }
    }
}

pub struct SkipNode {
    entry: Option<SkipEntry>,
    forward: Vec<Option<usize>>,
}

impl SkipNode {
    pub fn new(entry: SkipEntry) -> SkipNode {
        SkipNode {
            entry,
            forward: Vec::new(),
        }
    }

    pub fn dummy() -> SkipNode {
        SkipNode {
            entry: (),
            forward: Vec::new(),
        }
    }
}
