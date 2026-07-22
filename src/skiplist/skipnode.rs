use crate::entry::Entry;

pub struct SkipEntry {
    entry: Entry,
}

impl SkipEntry {
    pub fn new(entry: Entry) -> SkipEntry {
        SkipEntry {
            entry,
        }
    }

}

pub(crate) struct SkipNode {
    entry: Option<SkipEntry>,
    forward: Vec<Option<usize>>,
}

impl SkipNode {
    pub fn new(entry: SkipEntry) -> SkipNode {
        SkipNode {
            entry: Some(entry),
            forward: Vec::new(),
        }
    }

    pub fn dummy() -> SkipNode {
        SkipNode {
            entry: None,
            forward: Vec::new(),
        }
    }

    pub(crate) fn get_entry(&self) -> &Option<SkipEntry> {
        &self.entry
    }

    pub(crate) fn get_forward(&self) -> &Vec<Option<usize>> {
        &self.forward
    }

    pub(crate) fn get_forward_mut(&mut self) -> &mut Vec<Option<usize>> {
        &mut self.forward
    }
}
