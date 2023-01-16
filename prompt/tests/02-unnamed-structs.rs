use derive_prompt::*;

#[derive(FromPrompt)]
pub struct Newtype(u8);

#[derive(FromPrompt)]
pub struct Tuple2(u8, u8);

#[derive(FromPrompt)]
pub struct Tuple3(u8, u16, u32);

fn main() {}
