pub mod arena;
pub mod rc_refcell;

pub trait Cache<K, V> {
    fn new(capacity: usize) -> Self;
    fn get(&mut self, key: &K) -> Option<&V>;
    fn put(&mut self, key: K, value: V);
}
