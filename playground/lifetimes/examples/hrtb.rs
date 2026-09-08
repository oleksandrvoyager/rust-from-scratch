// Goal: a function accepting a closure/callback that must work for any
// lifetime, not one fixed at the call site — `for<'a> Fn(&'a str) -> ...`.

fn main() {}
