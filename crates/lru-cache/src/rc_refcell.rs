//! Rc<RefCell<Node>> + Weak back-references.

use crate::Cache;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::{Rc, Weak};

type Link<K, V> = Rc<RefCell<Node<K, V>>>;
type WeakLink<K, V> = Weak<RefCell<Node<K, V>>>;

pub struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, Link<K, V>>,
    head: Option<Link<K, V>>,
    tail: Option<WeakLink<K, V>>,
}

impl<K, V> Drop for LruCache<K, V> {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(node) = current {
            current = node.borrow_mut().next.take();
        }
    }
}

impl<K, V> LruCache<K, V> {
    fn push_front(&mut self, node: &Link<K, V>) {
        if let Some(head) = &self.head {
            head.borrow_mut().prev = Some(Rc::downgrade(node));
        }

        let mut n = node.borrow_mut();
        n.next = self.head.take();
        n.prev = None;
        self.head = Some(node.clone());

        if self.tail.is_none() {
            self.tail = Some(Rc::downgrade(node));
        }
    }

    fn move_to_front(&mut self, node: &Link<K, V>) {
        let (prev, next) = (node.borrow().prev.clone(), node.borrow().next.clone());
        match (prev, next) {
            // Middle: has both neighbors, splice it out and close the gap.
            (Some(prev), Some(next)) => {
                let prev_upgrade = prev.upgrade();
                next.borrow_mut().prev = Some(prev);
                prev_upgrade
                    .expect("a node's prev must still be alive: it's held by the map")
                    .borrow_mut()
                    .next = Some(next);

                self.push_front(node)
            }
            // Tail: its predecessor becomes the new tail.
            (Some(prev), None) => {
                prev.upgrade()
                    .expect("a node's prev must still be alive: it's held by the map")
                    .borrow_mut()
                    .next = None;
                self.tail = Some(prev);
                self.push_front(node)
            }
            // No predecessor: already the head (whether or not it has a successor), nothing to move.
            (None, _) => {}
        }
    }
}

impl<K, V> Cache<K, V> for LruCache<K, V>
where
    K: Hash + Eq + Clone,
{
    type Ref<'a> = Ref<'a, V>
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
            head: None,
            tail: None,
        }
    }

    fn get(&mut self, key: &K) -> Option<Self::Ref<'_>> {
        let node = self.map.get(key)?.clone();
        self.move_to_front(&node);

        // Re-borrow from `self.map` rather than `node`: a `Ref` returned from
        // `node.borrow()` would be tied to this function's local clone, not to `self`,
        // and couldn't be returned. This lookup ties the `Ref`'s lifetime to `self` instead.
        Some(Ref::map(self.map.get(key)?.borrow(), |n| &n.value))
    }

    fn put(&mut self, key: K, value: V) {
        if let Some(node) = self.map.get(&key).cloned() {
            self.move_to_front(&node);
            node.borrow_mut().value = value;

            return;
        }

        let node = Rc::new(RefCell::new(Node::new(key.clone(), value)));
        if self.map.len() >= self.capacity
            && let Some(tail) = self.tail.take().map(|w| {
                w.upgrade()
                    .expect("self.tail must still be alive: it's held by the map")
            })
        {
            self.map.remove(&tail.borrow().key);

            if let Some(prev_tail) = tail.borrow_mut().prev.take() {
                prev_tail
                    .upgrade()
                    .expect("a node's prev must still be alive: it's held by the map")
                    .borrow_mut()
                    .next = None;
                self.tail = Some(prev_tail);
            } else {
                self.head = None;
            }
        }

        self.push_front(&node);
        self.map.insert(key, node);
    }
}

struct Node<K, V> {
    key: K,
    value: V,
    next: Option<Link<K, V>>,
    prev: Option<WeakLink<K, V>>,
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
