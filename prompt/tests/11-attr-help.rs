use derive_prompt::*;

#[derive(Debug, Clone, FromPrompt)]
struct A {
    #[help(msg = "Write a u8")]
    a1: u8,
    a2: f32,
    a3: f64,
}

#[derive(Debug, Clone, FromPrompt)]
struct B(#[help(msg = "Write a u8")] u8, f32, f64);

#[derive(Debug, Clone, FromPrompt)]
enum C {
    A {#[help(msg = "Write a u8")] a1: u8, a2: f32, a3:f64 },
    B(u8),
    C
}

#[derive(Debug, Clone, FromPrompt)]
enum D {
    A(#[help(msg = "Write a u8")] u8, f32, f64),
    B(u8),
    C
}

fn main() {}
