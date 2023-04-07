#[derive(Debug, PartialEq, Eq, Default)]
struct A {
    a: u8,
    b: String,
}

#[derive(Debug, PartialEq, Eq, Default)]
struct B {
    a: u8
}

#[derive(Debug, PartialEq, Eq, derive_default::Default)]
enum Letters {
    #[default]
    A(A, u8),
    #[default]
    B(B)
}

fn main() {}
