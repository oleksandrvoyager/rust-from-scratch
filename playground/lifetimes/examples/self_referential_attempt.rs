// Goal: deliberately attempt a struct holding data and a reference into
// that same data. See the real compiler error, understand why Rust can't
// allow this directly.

struct SelfRef<'a> {
    data: String,
    reference: &'a str, // wants to point at `data`, in the same struct
}

fn main() {
    let s = String::from("hello");
    let r = &s;

    // Uncomment to see it fail: error[E0505] cannot move out of `s`
    // because it is borrowed. `r` still borrows `s` at this point, so `s`
    // (the owned String) can't be moved into `data` alongside it.
    //
    // Even setting that aside, the deeper reason self-referential structs
    // are forbidden: the whole struct can later be moved (returned, pushed
    // into a Vec, ...). If `data` lived inline in the struct rather than on
    // the heap, moving the struct would relocate `data` to a new address
    // while `reference` kept pointing at the old one — a dangling pointer
    // Rust has no way to detect or fix up automatically.
    //
    // let x = SelfRef { data: s, reference: r };

    println!("{} {}", s, r);
}
