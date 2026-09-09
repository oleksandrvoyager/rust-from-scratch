// Part 1: covariance — a longer-lived &T can substitute for a shorter-lived
// one, no special handling needed.
fn takes_short(_short: &i32) {}

fn covariance_demo() {
    let long_lived = 5;
    {
        let _short_lived = 10;
        takes_short(&long_lived); // fine: &'long substitutes for &'short
    }
}

// Part 2: invariance — the same substitution through &mut T is unsound,
// so Rust forbids it.
fn assign<T>(input: &mut T, val: T) {
    *input = val;
}

fn invariance_demo() {
    let mut hello: &'static str = "hello";
    {
        let world = String::from("world");
        // Uncomment to see it fail: error[E0597] `world` does not live long
        // enough. If &mut T were covariant like &T, this would silently
        // leave `hello` pointing at freed memory once `world` is dropped.
        // assign(&mut hello, &world);
        let _ = &world;
    }
    println!("{}", hello);
}

fn main() {
    covariance_demo();
    invariance_demo();
}
