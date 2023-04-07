#[derive(Debug, PartialEq, Eq, Default)]
struct B {
    a: u8,
    b: String,
}

#[derive(Debug, PartialEq, Eq, derive_default::Default)]
struct A {
    a: u8,
    b: B,
}

fn main() {}
