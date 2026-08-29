//! Vec arena + indices instead of pointers.

use crate::Cache;
use std::collections::HashMap;
use std::hash::Hash;

/// LRU cache backed by a `Vec` arena, with nodes linked via `usize`
/// indices instead of pointers. Eviction reuses the freed slot in place.
pub struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, usize>,
    vec: Vec<Option<Node<K, V>>>,
    head: Option<usize>,
    tail: Option<usize>,
}

impl<K, V> LruCache<K, V>
where
    K: Hash + Eq + Clone,
{
    fn push_front(&mut self, node_index: usize) {
        if let Some(head_index) = self.head {
            self.vec[head_index]
                .as_mut()
                .expect("self.head must point to a live node")
                .prev = Some(node_index);
        }

        self.vec[node_index]
            .as_mut()
            .expect("node_index must point to a live node")
            .next = self.head.take();

        self.vec[node_index]
            .as_mut()
            .expect("node_index must point to a live node")
            .prev = None;

        self.head = Some(node_index);

        if self.tail.is_none() {
            self.tail = Some(node_index);
        }
    }

    fn move_to_front(&mut self, index: usize) {
        let node = self.vec[index]
            .as_ref()
            .expect("index must point to a live node");

        match (node.prev, node.next) {
            // Middle: has both neighbors, splice it out and close the gap.
            (Some(prev), Some(next)) => {
                self.vec[prev]
                    .as_mut()
                    .expect("prev index must point to a live node")
                    .next = Some(next);

                self.vec[next]
                    .as_mut()
                    .expect("next index must point to a live node")
                    .prev = Some(prev);

                self.push_front(index);
            }
            // Tail: its predecessor becomes the new tail.
            (Some(prev), None) => {
                self.vec[prev]
                    .as_mut()
                    .expect("prev index must point to a live node")
                    .next = None;

                self.tail = Some(prev);

                self.push_front(index);
            }
            // No predecessor: already the head (whether or not it has a successor), nothing to move.
            (None, _) => {}
        }
    }

    fn evict_tail(&mut self, node: Node<K, V>, tail_index: usize) -> usize {
        let tail_node = &mut self.vec[tail_index];
        let prev_index = tail_node.as_ref().and_then(|node| node.prev);

        self.map.remove(
            &tail_node
                .as_ref()
                .expect("tail index must point to a live node")
                .key,
        );

        self.vec[tail_index] = Some(node);

        if let Some(prev_index) = prev_index {
            let prev_tail = &mut self.vec[prev_index];
            prev_tail
                .as_mut()
                .expect("tail index must point to a live node")
                .next = None;
            self.tail = Some(prev_index);
        } else {
            self.head = None;
        }

        tail_index
    }
}

impl<K, V> Cache<K, V> for LruCache<K, V>
where
    K: Eq + Hash + Clone,
{
    type Ref<'a>
        = &'a V
    where
        Self: 'a,
        V: 'a;

    /// # Panics
    ///
    /// Panics if `capacity` is `0`.
    fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be at least 1");
        Self {
            capacity,
            map: HashMap::new(),
            vec: vec![],
            head: None,
            tail: None,
        }
    }

    fn get(&mut self, key: &K) -> Option<Self::Ref<'_>> {
        let index = *self.map.get(key)?;
        self.move_to_front(index);

        Some(
            &self.vec[index]
                .as_ref()
                .expect("index must point to a live node")
                .value,
        )
    }

    fn put(&mut self, key: K, value: V) {
        if let Some(&index) = self.map.get(&key) {
            self.move_to_front(index);
            self.vec[index]
                .as_mut()
                .expect("index must point to a live node")
                .value = value;

            return;
        }

        let node = Node::new(key.clone(), value);
        let new_index = if self.vec.len() >= self.capacity
            && let Some(tail_index) = self.tail.take()
        {
            self.evict_tail(node, tail_index)
        } else {
            self.vec.push(Some(node));
            self.vec.len() - 1
        };

        self.push_front(new_index);
        self.map.insert(key, new_index);
    }
}

struct Node<K, V> {
    key: K,
    value: V,
    next: Option<usize>,
    prev: Option<usize>,
}

impl<K, V> Node<K, V> {
    fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            next: None,
            prev: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_on_empty_cache_returns_none() {
        let mut c: LruCache<i32, i32> = LruCache::new(2);
        assert!(c.get(&1).is_none());
    }

    #[test]
    fn put_then_get_returns_value() {
        let mut c: LruCache<i32, i32> = LruCache::new(3);
        c.put(1, 10);
        assert_eq!(*c.get(&1).unwrap(), 10);
    }

    #[test]
    fn put_overwrites_existing_key() {
        let mut c: LruCache<i32, i32> = LruCache::new(3);
        c.put(1, 10);
        c.put(1, 99);
        assert_eq!(*c.get(&1).unwrap(), 99);
    }

    #[test]
    fn fills_below_capacity_without_eviction() {
        let mut c: LruCache<i32, i32> = LruCache::new(3);
        c.put(1, 10);
        c.put(2, 20);
        assert_eq!(*c.get(&1).unwrap(), 10);
        assert_eq!(*c.get(&2).unwrap(), 20);
    }

    #[test]
    fn eviction_removes_least_recently_used() {
        let mut c: LruCache<i32, i32> = LruCache::new(2);
        c.put(1, 10);
        c.put(2, 20);
        c.put(3, 30); // evicts 1

        assert!(c.get(&1).is_none());
        assert_eq!(*c.get(&2).unwrap(), 20);
        assert_eq!(*c.get(&3).unwrap(), 30);
    }

    #[test]
    fn get_refreshes_recency_and_protects_from_eviction() {
        let mut c: LruCache<i32, i32> = LruCache::new(2);
        c.put(1, 10);
        c.put(2, 20);
        c.get(&1); // 1 is now most recently used, 2 becomes LRU
        c.put(3, 30); // evicts 2, not 1

        assert!(c.get(&2).is_none());
        assert_eq!(*c.get(&1).unwrap(), 10);
        assert_eq!(*c.get(&3).unwrap(), 30);
    }

    #[test]
    fn put_on_existing_key_also_refreshes_recency() {
        let mut c: LruCache<i32, i32> = LruCache::new(2);
        c.put(1, 10);
        c.put(2, 20);
        c.put(1, 11); // touches 1, 2 becomes LRU
        c.put(3, 30); // evicts 2, not 1

        assert!(c.get(&2).is_none());
        assert_eq!(*c.get(&1).unwrap(), 11);
        assert_eq!(*c.get(&3).unwrap(), 30);
    }

    #[test]
    fn repeated_eviction_reuses_freed_slots() {
        let mut c: LruCache<i32, i32> = LruCache::new(2);
        for i in 0..10 {
            c.put(i, i * 100);
        }

        assert!(c.get(&7).is_none());
        assert_eq!(*c.get(&8).unwrap(), 800);
        assert_eq!(*c.get(&9).unwrap(), 900);
    }

    #[test]
    fn evicted_key_can_be_reinserted() {
        let mut c: LruCache<i32, i32> = LruCache::new(2);
        c.put(1, 10);
        c.put(2, 20);
        c.put(3, 30); // evicts 1
        c.put(1, 111); // re-insert 1, evicts 2 (now LRU)

        assert!(c.get(&2).is_none());
        assert_eq!(*c.get(&1).unwrap(), 111);
        assert_eq!(*c.get(&3).unwrap(), 30);
    }

    #[test]
    fn capacity_of_one_evicts_every_put() {
        let mut c: LruCache<i32, i32> = LruCache::new(1);
        c.put(1, 10);
        c.put(2, 20);

        assert!(c.get(&1).is_none());
        assert_eq!(*c.get(&2).unwrap(), 20);
    }

    #[test]
    #[should_panic(expected = "capacity must be at least 1")]
    fn zero_capacity_panics() {
        let _: LruCache<i32, i32> = LruCache::new(0);
    }
}
