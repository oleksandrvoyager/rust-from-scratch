//! A fixed-capacity LRU cache, implemented two different ways behind a
//! shared [`Cache`] trait. See [`arena`] and [`rc_refcell`] for the designs.

use std::ops::Deref;

pub mod arena;
pub mod rc_refcell;

/// A fixed-capacity cache that evicts the least recently used entry once full.
pub trait Cache<K, V> {
    /// Reference to a stored value, returned by [`get`](Cache::get) and tied
    /// to `self`'s borrow. Each design uses its own natural reference type.
    type Ref<'a>: Deref<Target = V>
    where
        Self: 'a,
        V: 'a;

    /// Creates an empty cache holding at most `capacity` entries.
    fn new(capacity: usize) -> Self;

    /// Returns the value for `key` and marks it most recently used, or
    /// `None` if `key` isn't present.
    fn get(&mut self, key: &K) -> Option<Self::Ref<'_>>;

    /// Inserts or updates the value for `key` and marks it most recently
    /// used. If the cache is at capacity, evicts the least recently used
    /// entry first.
    fn put(&mut self, key: K, value: V);
}
