use std::cmp::Ordering;
use bytes::Bytes;

pub trait KeyComparator<T> : Send + Sync {
    fn compare(&self, first: &T, second: &T) -> Ordering; 
}

pub struct BytewiseComparator {}

impl KeyComparator<Bytes> for BytewiseComparator{
    fn compare(&self, first: &Bytes, second: &Bytes) -> Ordering {
        first.cmp(second)
    }
}
