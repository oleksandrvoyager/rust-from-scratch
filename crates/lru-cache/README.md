# lru-cache

Topic: ownership, borrowing, lifetimes (advanced).

Two independent designs implementing a shared `Cache` trait:

- `rc_refcell` — `Rc<RefCell<Node>>` + `Weak` for back-references.
- `arena` — `Vec` arena, nodes linked via indices.
