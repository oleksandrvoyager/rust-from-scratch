// Goal: a minimal case that does NOT compile without an explicit outlives
// bound (`'a: 'b`) — find it, add the bound, see it compile.

fn main() {}

fn x<'a, 'b, T>(y: &'a T) -> &'b T
where
    'a: 'b
{
    y
}
