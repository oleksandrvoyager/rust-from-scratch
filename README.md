# rust-from-scratch

Mini-projects built from scratch, each targeting 1-2 Rust topics in depth. Where a topic supports multiple substantially different designs, both are implemented and compared.

## Projects

1. [`lru-cache`](crates/lru-cache) — ownership, borrowing, lifetimes (two designs: `Rc<RefCell<>>` vs arena/indices)
2. [`binary-protocol-parser`](crates/binary-protocol-parser) — memory layout (alignment, endianness, zero-copy lifetimes)
