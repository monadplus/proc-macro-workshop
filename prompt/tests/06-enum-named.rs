use derive_prompt::*;

#[derive(FromPrompt)]
pub struct A1(u8);

#[derive(FromPrompt)]
pub enum Choice {
    A(A1),
    B { b1: u8, b2: A1 },
}

fn main() {}
