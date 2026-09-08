# Roadmap

Mini-projects in order, each targeting 1-2 Rust topics in depth. Where a topic
supports multiple substantially different designs, both are implemented and
compared.

| # | Project | Topic | Status |
|---|---------|-------|--------|
| 1 | [`lru-cache`](crates/lru-cache) | Ownership, borrowing, lifetimes | Done |
| 2 | [`binary-protocol-parser`](crates/binary-protocol-parser) | Memory layout (alignment, endianness, zero-copy) | Done |
| 3 | Lexer/tokenizer + symbol table | Lifetimes in depth | Planned |
| 4 | Lazy log/CSV engine | Iterators, custom adapters | Planned |
| 5 | Rules/middleware engine | Traits & generics (exhaustive) | Planned |
| 6 | Config loader | Error handling, serde | Planned |
| 7 | Task runner | Concurrency (Send/Sync, Arc/Mutex) | Planned |
| 8 | Ring buffer | Unsafe Rust | Planned |
| 9 | Test DSL + derive macro | Macros | Planned |
| 10 | Concurrent HTTP downloader | Async basics | Planned |
| 11 | TCP chat/proxy | Tokio in depth | Planned |
| 12 | Rate-limited event aggregator | Streams & pinning | Planned |
| 13 | Chat server retrofit | Graceful shutdown & cancellation | Planned |
| 14 | gRPC service | tonic + prost | Planned |
| — | REST API capstone | Full workspace, axum, sqlx, tracing | Planned |

Full task breakdown lives in [GitHub Issues](https://github.com/oleksandrvoyager/rust-from-scratch/issues) and the [project board](https://github.com/users/oleksandrvoyager/projects/1).
