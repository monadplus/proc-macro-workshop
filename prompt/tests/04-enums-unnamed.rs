use derive_prompt::*;

#[derive(FromPrompt)]
pub struct A1(u8);

#[derive(FromPrompt)]
pub struct B1(u8, u8);

#[derive(FromPrompt)]
pub struct C1(u8, u8, String);

#[derive(FromPrompt)]
pub enum Choice {
    A(A1),
    B(B1, C1),
}

fn main() {}
