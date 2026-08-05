use crate::entry::Value;

pub(crate) trait StorageIterator {
    type Error;
    fn key(&self) -> &[u8];
    fn value(&self) -> &Value;
    fn is_valid(&self) -> bool;
    fn next(&mut self) -> Result<(), Self::Error>;
    fn seek(&mut self, key: &[u8]) -> Result<(), Self::Error>;
}
