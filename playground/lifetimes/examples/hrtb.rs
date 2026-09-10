// Broken version (kept for comparison, not compiled): F is only required
// to work for ONE specific lifetime 'a, fixed by call_with_two_broken's
// caller before the function body even runs — before `a`/`b` exist.
//
// fn call_with_two_broken<'a, F>(f: F)
// where
//     F: Fn(&'a str) -> &'a str,
// {
//     let a = String::from("first");
//     println!("{}", f(&a)); // error: `a` does not live long enough
//     {
//         let b = String::from("second, different scope");
//         println!("{}", f(&b)); // error: `b` does not live long enough
//     }
// }

// Fixed version: for<'a> means F must work for ANY lifetime, chosen fresh
// at each individual call to f, not pinned once by the caller of
// call_with_two itself.
fn call_with_two<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    let a = String::from("first");
    println!("{}", f(&a));
    {
        let b = String::from("second, different scope");
        println!("{}", f(&b));
    }
}

fn identity(s: &str) -> &str {
    s
}

fn main() {
    call_with_two(identity);
}
