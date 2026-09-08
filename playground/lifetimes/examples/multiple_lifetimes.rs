// Goal: a struct that genuinely needs two independent lifetime parameters —
// find a case where collapsing them into one shared lifetime would be wrong
// or wouldn't compile.

fn main() {
    let x = "hello y".to_string();
    let foo_x = {
        let y = "hello x".to_string();
        let foo = Foo { x: &x, y: &y };
        foo.x
    };

    println!("{}", foo_x);
}

#[derive(Debug)]
struct Foo<'a, 'b> {
    x: &'a str,
    y: &'b str,
}
