use std::ops::Deref;

pub mod arena;
pub mod rc_refcell;

pub trait Cache<K, V> {
    type Ref<'a>: Deref<Target = V>
    where
        Self: 'a,
        V: 'a;

    fn new(capacity: usize) -> Self;
    fn get(&mut self, key: &K) -> Option<Self::Ref<'_>>;
    fn put(&mut self, key: K, value: V);
}
