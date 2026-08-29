# lru-cache

Topic: ownership, borrowing, lifetimes (advanced).

LRU cache built from scratch, two designs behind a shared `Cache<K, V>` trait (a GAT lets each design return its own natural reference type from `get`).

## Designs

### `rc_refcell`

`Rc<RefCell<Node<K, V>>>` doubly-linked list, `Weak` back-references to avoid reference cycles. `get` returns `Ref<'_, V>`.

- Every node is its own heap allocation; every access pays a `RefCell` runtime borrow check.
- No `unsafe`.

### `arena`

`Vec<Option<Node<K, V>>>`, nodes linked via `usize` indices instead of pointers. `get` returns `&V`.

- One backing allocation; eviction reuses the freed slot in place.
- No runtime borrow checks — the borrow checker only ever sees plain `usize` values.
- The `Option` around each node has no reachable `None` today (eviction always overwrites in place); kept as a slot for a future `remove`.

## Usage

```rust
use lru_cache::{Cache, arena::LruCache};

let mut cache: LruCache<&str, i32> = LruCache::new(2);
cache.put("a", 1);
cache.put("b", 2);
cache.put("c", 3); // evicts "a"

assert!(cache.get(&"a").is_none());
assert_eq!(*cache.get(&"b").unwrap(), 2);
```

Swap `arena::LruCache` for `rc_refcell::LruCache` for the other design — both implement the same `Cache<K, V>` trait.

## Benchmarks

`cargo bench` compares both designs across sizes 100 to 100,000 on three scenarios: `fill`, `get_hits` (all hits, each refreshing recency), and `put_with_eviction`. `arena` is consistently ~1.5-1.6x faster at every size — no per-node allocation, no `RefCell` checks — and the gap holds steady as size grows (both scale linearly). HTML report with graphs: `target/criterion/report/index.html`.

## Tests

`cargo test` runs unit tests per design (`src/rc_refcell.rs`, `src/arena.rs`) plus integration tests exercising both through the shared trait (`tests/integration.rs`).
