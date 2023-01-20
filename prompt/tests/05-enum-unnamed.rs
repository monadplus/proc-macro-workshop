use derive_prompt::*;

#[derive(FromPrompt)]
pub struct A1(u8);

#[derive(FromPrompt)]
pub struct B1(u8, u8);

#[derive(FromPrompt)]
pub enum Choice {
    A(A1),
    B(u8, B1),
}

fn main() {}
