#[derive(Debug, PartialEq, Eq, Default)]
struct A {
    a: u8,
    b: String,
}

#[derive(Debug, PartialEq, Eq)]
struct B {
    a: u8
}

#[derive(Debug, PartialEq, Eq, derive_default::Default)]
enum Letters {
    #[default]
    A { a: A, b: u8 },
    B(B)
}

fn main() {
    let expected = Letters::A { a: A::default(), b: u8::default() };
    assert_eq!(Letters::default(), expected);
}
