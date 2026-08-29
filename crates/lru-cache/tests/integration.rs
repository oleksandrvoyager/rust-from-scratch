// Integration tests for both implementations via the shared Cache trait.

use lru_cache::Cache;
use lru_cache::{arena, rc_refcell};

fn basic_put_get<C: Cache<i32, i32>>() {
    let mut c = C::new(3);
    assert!(c.get(&1).is_none());

    c.put(1, 10);
    assert_eq!(*c.get(&1).unwrap(), 10);

    c.put(1, 99);
    assert_eq!(*c.get(&1).unwrap(), 99);
}

fn eviction_and_recency<C: Cache<i32, i32>>() {
    let mut c = C::new(2);
    c.put(1, 10);
    c.put(2, 20);
    c.get(&1); // 1 is now most recently used, 2 becomes LRU
    c.put(3, 30); // evicts 2, not 1

    assert!(c.get(&2).is_none());
    assert_eq!(*c.get(&1).unwrap(), 10);
    assert_eq!(*c.get(&3).unwrap(), 30);
}

fn repeated_eviction_reuses_capacity<C: Cache<i32, i32>>() {
    let mut c = C::new(2);
    for i in 0..20 {
        c.put(i, i * 100);
    }

    assert!(c.get(&17).is_none());
    assert_eq!(*c.get(&18).unwrap(), 1800);
    assert_eq!(*c.get(&19).unwrap(), 1900);
}

#[test]
fn arena_basic_put_get() {
    basic_put_get::<arena::LruCache<i32, i32>>();
}

#[test]
fn arena_eviction_and_recency() {
    eviction_and_recency::<arena::LruCache<i32, i32>>();
}

#[test]
fn arena_repeated_eviction_reuses_capacity() {
    repeated_eviction_reuses_capacity::<arena::LruCache<i32, i32>>();
}

#[test]
fn rc_refcell_basic_put_get() {
    basic_put_get::<rc_refcell::LruCache<i32, i32>>();
}

#[test]
fn rc_refcell_eviction_and_recency() {
    eviction_and_recency::<rc_refcell::LruCache<i32, i32>>();
}

#[test]
fn rc_refcell_repeated_eviction_reuses_capacity() {
    repeated_eviction_reuses_capacity::<rc_refcell::LruCache<i32, i32>>();
}
