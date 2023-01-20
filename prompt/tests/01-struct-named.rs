use derive_prompt::*;

#[derive(Debug, FromPrompt)]
pub struct Command {
    executable: String,
    iterations: u64,
    precision: f64,
    shouting: bool,
    ids: Vec<u8>,
}

fn main() {}
