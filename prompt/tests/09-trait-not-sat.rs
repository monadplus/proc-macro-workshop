use derive_prompt::*;

#[derive(Debug)]
struct B;

#[derive(Debug, FromPrompt)]
struct A {
    a1: u8,
    a2: B,
}

#[derive(Debug, FromPrompt)]
enum C {
    A(u8),
    B(B)
}

fn main() {}
