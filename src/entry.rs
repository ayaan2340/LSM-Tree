use bytes::Bytes;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Entry {
    pub key: Bytes,
    pub val: Value,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Value {
    pub value: Bytes,
    pub(crate) metadata: u8,
}

impl Entry {
    pub fn new(key: Bytes, value: Bytes, metadata: u8) -> Entry {
        Entry {
            key,
            val: Value {
                value,
                metadata, },
        }
    }    
}
