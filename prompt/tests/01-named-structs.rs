use derive_prompt::*;

#[derive(FromPrompt)]
pub struct Command {
    executable: String,
    iterations: u64,
    precision: f64,
}

fn main() {}
