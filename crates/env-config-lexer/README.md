# env-config-lexer

Topic: lifetimes in depth (multiple independent lifetimes, outlives bounds,
variance, higher-ranked trait bounds, self-referential structs).

Zero-copy lexer + resolver for a small env/Makefile-style config format
(`NAME = value`, with `${OTHER_NAME}` interpolation). Field layout:
[`FORMAT.md`](FORMAT.md).

## Usage

```rust
use env_config_lexer::{tokenize, build_symbol_table, resolve_all};

let source = "HOST = localhost\nURL = https://${HOST}:8080/api";
let tokens = tokenize(source)?;
let table = build_symbol_table(tokens);
let resolved = resolve_all(&table)?;

assert_eq!(resolved["URL"], "https://localhost:8080/api");
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Pipeline

```
source (&str)
  -> tokenize()           -> Vec<Token>                 (zero-copy)
  -> build_symbol_table() -> HashMap<&str, Vec<Segment>> (zero-copy, grouped by name)
  -> resolve_all()        -> HashMap<&str, String>       (owned, references substituted)
```

`Token`/`Segment` borrow directly from the source text — no allocation.
Resolved values are necessarily owned `String`s: a value built from
multiple non-adjacent pieces (literal text plus a substituted reference)
doesn't exist as a contiguous span anywhere in the source, so there's
nothing to borrow from.

## Self-referential structs

Playground exercises (`playground/lifetimes/`) worked through why a
struct can't directly hold data and a reference into that same data.
This crate's `resolve_all` builds a map of independently-owned `String`s
and only ever hands out short-lived `&str` borrows — a caching attempt
that recurses through the same map while building it was tried and
compiles fine, because Rust's borrow checker (aided by NLL) can prove
each borrow ends before the map is mutated again. No arena/index
workaround was needed here in practice.
